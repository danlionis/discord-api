use std::{error::Error, sync::Arc, time::Duration};

use discord::{
    model::gateway::Event,
    proto::{Connection, State},
    rest::{Client, CreateMessageParams},
};
use futures::{
    future::{poll_fn, Either},
    prelude::*,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_tungstenite as ws;
use ws::{MaybeTlsStream, WebSocketStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = std::env::var("TOKEN").expect("missing token");

    env_logger::init();

    // create Discord Rest Client
    let rest = Arc::new(Client::new(token.to_string()));

    // connect to websocket
    let gateway_info = rest.get_gateway_bot().await?;
    let mut url = gateway_info.url;
    url.push_str("/?v=9");
    let url = url.as_str();
    let (mut socket, _) = ws::connect_async(url).await?;

    // initialize connection and receive first hello packet
    let mut conn = Connection::new(token);
    let hello = socket.next().await.unwrap()?;
    let hello = hello.to_text()?;
    conn.recv_json(hello)?;

    // create heartbeat interval
    let mut interval = tokio::time::interval(Duration::from_millis(conn.heartbeat_interval()));

    loop {
        match conn.state() {
            State::Reconnect | State::InvalidSession(true) => {
                socket = reconnect_socket(socket, url).await?;
                conn.resume();
            }
            State::InvalidSession(false) => {
                socket = reconnect_socket(socket, url).await?;
                conn.reconnect();
            }
            _ => {}
        }

        // select between the interval and the websocket
        let select =
            futures::future::select(poll_fn(|mut cx| interval.poll_tick(&mut cx)), socket.next());

        match select.await {
            Either::Left(_) => {
                conn.queue_heartbeat();
            }
            Either::Right((Some(Ok(msg)), _)) => {
                conn.recv_json(msg.to_text()?)?;

                for event in conn.events() {
                    let r = Arc::clone(&rest);
                    handle_event(event, r);
                }
            }
            _ => {
                log::error!("an error occured, closing connection and reconnecting");
                socket = reconnect_socket(socket, url).await?;
                conn.reconnect();
            }
        }

        // iterate through all packets generated and send them to the gateway
        for s in conn.send_iter_json() {
            socket.send(ws::tungstenite::Message::Text(s)).await?;
        }
    }
}

async fn reconnect_socket<S>(
    mut socket: WebSocketStream<S>,
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ws::tungstenite::Error>
where
    S: AsyncWrite + AsyncRead + Unpin,
{
    let _ = socket.close(None).await;
    let (socket, _) = ws::connect_async(url).await?;
    Ok(socket)
}

fn handle_event(event: Event, rest: Arc<Client>) {
    if let Event::MessageCreate(msg) = event {
        if msg.content.contains("!ping") {
            tokio::spawn(async move {
                let params = CreateMessageParams::default().content("Pong");
                rest.create_message(msg.channel_id, params).await.unwrap();
            });
        }
    }
}
