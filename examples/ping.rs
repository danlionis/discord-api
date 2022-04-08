use std::{error::Error, time::Duration};

use discord::{
    model::gateway::Event,
    proto::{Connection, State},
    rest::Rest,
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
    let rest = Rest::new(&token);

    // connect to websocket
    let url = "wss://gateway.discord.gg/?v=9";
    let (mut socket, _) = ws::connect_async(url).await?;

    // initialize connection and receive first hello packet
    let mut conn = Connection::new(token);
    let hello = socket.next().await.unwrap()?;
    let hello = hello.to_text()?;
    conn.recv_json_str(hello)?;

    // create heartbeat interval
    let mut interval =
        tokio::time::interval(Duration::from_millis(conn.heartbeat_interval().unwrap()));

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
                conn.recv_json_str(msg.to_text()?)?;

                for event in conn.events() {
                    let r = rest.clone();
                    handle_event(event, r);
                }
            }
            _ => {
                log::error!("an error occured, closing connections and reconnecting");
                socket = reconnect_socket(socket, url).await?;
                conn.reconnect();
            }
        }

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

fn handle_event(event: Event, rest: Rest) {
    if let Event::MessageCreate(msg) = event {
        if msg.content.contains("!ping") {
            tokio::spawn(async move {
                rest.create_message(msg.channel_id, "Pong", None)
                    .await
                    .unwrap();
            });
        }
    }
}
