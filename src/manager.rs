//! Managed connections to the discord gateway
//!
//! Managed connections handle I/O operations in addition to the gateway protocol.
//!
//! # Example
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! # let token = "";
//! let mut manager = discord::manager::connect(token).await?;
//!
//! while let Ok(event) = manager.recv().await {
//!     println!("received event: {}", event.kind());
//! }
//! # Ok(())
//! # }
//! ```

use crate::{model::gateway::Event, proto::Connection, rest::client::Client};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{error::Error, fmt::Debug, ops::Deref, sync::Arc, time::Duration};
use tokio::{net::TcpStream, time::Interval};
use tokio_tungstenite::{self as ws, WebSocketStream};
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
pub async fn connect<S>(token: S) -> Result<Manager, ws::tungstenite::Error>
where
    S: Into<String>,
{
    let token: String = token.into();
    let rest = Client::new(token.clone());
    let mut conn = Connection::new(token.clone());

    let url = {
        let mut gateway_info = rest.get_gateway_bot().await.unwrap();
        gateway_info.url.push_str("/?v=9");
        gateway_info.url
    };

    let (mut socket, _) = ws::connect_async(&url).await.unwrap();

    // init connection
    let hello = socket.next().await.unwrap()?;
    let hello = hello.to_text()?;
    conn.recv_json(hello).unwrap();

    let interval = tokio::time::interval(Duration::from_millis(conn.heartbeat_interval()));

    Ok(Manager {
        conn,
        socket,
        rest: Arc::new(rest),
        token,
        url,
        interval,
    })
}

/// Managed connection to the discord gateway
///
/// This manager uses the [tokio_tungstenite](https://docs.rs/tokio-tungstenite) crate for
/// websockets and the included [`Client`](Client) as REST client.
///
/// # Example
/// ```no_run
/// # use std::{error::Error, sync::Arc};
/// # use discord::model::gateway::Event;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let mut manager = discord::manager::connect("YOUR_TOKEN").await?;
///
///     while let Ok(event) = manager.recv().await {
///         if let Event::MessageCreate(msg) = event {
///             let rest = Arc::clone(manager.rest());
///
///             // spawn new task to not block recv loop
///             tokio::spawn(async move {
///                 // react with 😀
///                 let _ = rest
///                     .create_reaction(msg.channel_id, msg.id, "%F0%9F%98%80".to_owned())
///                     .await;
///             });
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub struct Manager {
    conn: Connection,
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    rest: Arc<Client>,
    token: String,
    url: String,
    interval: Interval,
}

impl Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("conn", &self.conn)
            // .field("socket", &self.socket)
            .field("rest", &self.rest)
            .field("token", &self.token)
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
    pub async fn recv(&mut self) -> Result<Event, Box<dyn Error>> {
        loop {
            if let Some(event) = self.conn.events_ref_mut().pop_front() {
                log::trace!("passing event to receiver: {:?}", event);
                return Ok(event);
            }

            if let Some(code) = self.conn.failed() {
                return Err(code.into());
            }

            if self.conn.should_reconnect() {
                self.reconnect_socket().await?;
            }

            tokio::select! {
                _ = self.interval.tick() => {
                    self.conn.queue_heartbeat();
                }
                ws_msg = self.socket.next() => {
                    match ws_msg {
                        Some(Ok(msg)) => {
                            self.handle_ws_message(msg).await?;
                        }
                        Some(Err(e)) => {
                            log::warn!("an error occured, reconnecting... : {}", e);
                            self.reconnect_socket().await?;
                        }
                        None => {
                            log::warn!("stream closed, reconnecting...");
                            self.reconnect_socket().await?;
                        }
                    }
                }
            }

            // iterate through all packets generated and send them to the gateway
            for s in self.conn.send_iter_json() {
                log::debug!("sending: {}", s);
                self.socket
                    .feed(Message::Text(s))
                    .await
                    .expect("could not send");
            }
            self.socket.flush().await?;
        }
    }

    async fn handle_ws_message(
        &mut self,
        msg: ws::tungstenite::Message,
    ) -> Result<(), Box<dyn Error>> {
        match msg {
            Message::Close(Some(CloseFrame { code, reason })) => {
                log::debug!("conn closed: code= {} reason= {}", code, reason);
                self.conn.recv_close_code(code);
            }
            Message::Text(msg) => {
                self.conn.recv_json(&msg)?;
            }
            msg => {
                log::info!("ignoring unexpected message: {:?}", msg);
            }
        }
        Ok(())
    }

    async fn reconnect_socket(&mut self) -> Result<(), ws::tungstenite::Error> {
        log::info!("reconnecting socket");
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
