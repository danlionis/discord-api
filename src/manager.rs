//! Managed connections to the discord gateway
//!
//! Managed connections handle I/O operations in addition to the gateway protocol.
//!
//! # Example
//!
//! ```no_run
//! # use discord::{proto::*, model::gateway::Intents};
//! # async fn run() -> Result<(), discord::Error> {
//! # let token = "";
//! let config = Config::new(token, Intents::all());
//! let mut manager = discord::manager::connect(config).await?;
//!
//! while let Ok(event) = manager.recv().await {
//!     println!("received event: {:?}", event.kind());
//! }
//! # Ok(())
//! # }
//! ```

use crate::{
    proto::{Config, GatewayContext},
    Error, API_VERSION,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{fmt::Debug, ops::Deref, sync::Arc, time::Duration};
use tokio::{net::TcpStream, time::Interval};
use tokio_tungstenite::{self as ws, WebSocketStream};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use ws::{
    tungstenite::{protocol::CloseFrame, Message},
    MaybeTlsStream,
};

/// Connect to the discord gateway.
///
/// It is expected from the client that it starts heartbeating right after connecting.
/// The manager sends heartbeats automatically as long as it is awaiting an event via the
/// [`recv()`] method.
///
/// Should too much time pass between two calls of [`recv()`] it could happen
/// that the gateway will close the connection.
/// The manager will try to resume the session during the next call to [`recv()`].
/// If the session timed out in the connection will be reset.
///
/// It is recommended to receive events in a loop and spawn a new thread/task for any
/// event processing so that the manager will not be blocked from sending heartbeats.
///
/// # Example
/// See [module docs][self]
///
/// [`recv()`]: Manager::recv
pub async fn connect(config: Config) -> Result<Manager, Error> {
    let token = config.token.clone();
    let rest = Client::new(token.clone());
    let mut ctx = GatewayContext::new(config.clone());

    let info = {
        let mut info = rest.gateway().authed().exec().await?.model().await.unwrap();
        info.url.push_str("/?v=");
        info.url.push_str(&API_VERSION.to_string());
        info
    };

    let (mut socket, _) = ws::connect_async(&info.url).await.unwrap();

    // init connection
    let hello = socket.next().await.unwrap()?;
    let hello = hello.to_text()?;
    ctx.recv_json(hello).unwrap();

    let interval = tokio::time::interval(Duration::from_millis(ctx.heartbeat_interval()));

    Ok(Manager {
        ctx,
        socket,
        rest: Arc::new(rest),
        config,
        url: info.url,
        interval,
    })
}

/// Managed connection to the discord gateway
///
/// This manager uses the [tokio_tungstenite](https://docs.rs/tokio-tungstenite) crate for
/// websockets and the `twilight_http` [`Client`](Client) REST client.
pub struct Manager {
    ctx: GatewayContext,
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    rest: Arc<Client>,
    config: Config,
    url: String,
    interval: Interval,
}

impl Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("conn", &self.ctx)
            // .field("socket", &self.socket)
            .field("rest", &self.rest)
            .field("token", &self.config.token)
            .field("url", &self.url)
            .field("interval", &self.interval)
            .finish()
    }
}

impl Manager {
    /// get a reference to the internal rest client
    pub fn rest(&self) -> &Arc<Client> {
        &self.rest
    }

    /// Receive an event from the gateway
    pub async fn recv(&mut self) -> Result<Event, Error> {
        loop {
            if let Some(event) = self.ctx.event() {
                log::trace!("passing event to receiver: {:?}", event);
                return Ok(event);
            }

            if let Some(code) = self.ctx.failed() {
                return Err(code.into());
            }

            if self.ctx.should_reconnect() {
                self.reconnect_socket().await?;
            }

            tokio::select! {
                _ = self.interval.tick() => {
                    self.ctx.queue_heartbeat();
                }
                ws_msg = self.socket.next() => {
                    match ws_msg {
                        Some(Ok(msg)) => {
                            self.handle_ws_message(msg).await?;
                        }
                        Some(Err(e)) => {
                            log::info!("an error occured while receiving a message: {}", e);
                            self.reconnect_socket().await?;
                        }
                        None => {
                            log::info!("websocket stream closed...");
                            self.reconnect_socket().await?;
                        }
                    }
                }
            }

            // iterate through all packets generated and send them to the gateway
            for s in self.ctx.send_iter_json() {
                log::debug!("sending: {}", s);
                self.socket
                    .feed(Message::Text(s))
                    .await
                    .expect("could not send");
            }
            self.socket.flush().await?;
        }
    }

    async fn handle_ws_message(&mut self, msg: ws::tungstenite::Message) -> Result<(), Error> {
        match msg {
            Message::Close(Some(CloseFrame { code, reason })) => {
                log::debug!("conn closed: code= {} reason= {}", code, reason);
                self.ctx.recv_close_code(code);
            }
            Message::Text(msg) => {
                self.ctx.recv_json(&msg)?;
            }
            msg => {
                log::info!("ignoring unexpected message: {:?}", msg);
            }
        }
        Ok(())
    }

    async fn reconnect_socket(&mut self) -> Result<(), ws::tungstenite::Error> {
        log::debug!("reconnecting socket");
        let _ = self.socket.close(None).await;
        let (socket, _) = ws::connect_async(&self.url).await?;
        self.socket = socket;
        Ok(())
    }
}

impl Deref for Manager {
    type Target = Arc<Client>;
    fn deref(&self) -> &<Self as Deref>::Target {
        self.rest()
    }
}
