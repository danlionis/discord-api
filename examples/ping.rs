use discord::{
    model::gateway::{event::Event, Intents},
    proto::{Config, GatewayContext},
    Error, API_VERSION,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_tungstenite::{
    self as ws,
    tungstenite::{protocol::CloseFrame, Message},
    MaybeTlsStream, WebSocketStream,
};
use twilight_http::Client;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::args().skip(1).next().expect("missing token");

    env_logger::init();

    // create Discord Rest Client
    let rest = Arc::new(Client::new(token.to_string()));

    // connect to websocket
    let info = {
        let mut info = rest.gateway().authed().exec().await?.model().await.unwrap();
        info.url.push_str("/?v=");
        info.url.push_str(&API_VERSION.to_string());
        info
    };
    let (mut socket, _) = ws::connect_async(&info.url).await?;

    // initialize connection and receive first hello packet
    let config = Config::new(token, Intents::GUILD_MESSAGES);
    let mut conn = GatewayContext::new(config);
    let hello = socket.next().await.unwrap()?;
    let hello = hello.to_text()?;
    conn.recv_json(hello)?;

    // create heartbeat interval
    let mut interval = tokio::time::interval(Duration::from_millis(conn.heartbeat_interval()));

    loop {
        // reconnect the websocket if requested
        if conn.should_reconnect() {
            socket = reconnect_socket(socket, &info.url).await?;
        }

        if let Some(code) = conn.failed() {
            return Err(code.into());
        }

        tokio::select! {
            _ = interval.tick() => {
                conn.queue_heartbeat();
            }
            ws_msg = socket.next() => {
                match ws_msg {
                    Some(Ok(msg)) => {
                        handle_ws_message(msg, &mut conn, &rest).await?;
                    }
                    _ => {
                        log::info!("an error occured, closing connection and reconnecting");
                        socket = reconnect_socket(socket, &info.url).await?;
                    }
                }
            }
        };

        // iterate through all packets generated and send them to the gateway
        for s in conn.send_iter_json() {
            socket
                .send(ws::tungstenite::Message::Text(s))
                .await
                .expect("could not send");
        }
    }
}

async fn handle_ws_message(
    msg: ws::tungstenite::Message,
    conn: &mut GatewayContext,
    rest: &Arc<Client>,
) -> Result<(), Error> {
    match msg {
        Message::Close(Some(CloseFrame {
            code,
            reason: _reason,
        })) => {
            conn.recv_close_code(code);
        }
        Message::Text(msg) => {
            let event = conn.recv_json(&msg)?;
            tokio::spawn(handle_event(Event::from(event), Arc::clone(rest)));
        }
        msg => {
            log::info!("ignoring message: {:?}", msg);
        }
    }

    Ok(())
}

async fn reconnect_socket<S>(
    mut socket: WebSocketStream<S>,
    url: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ws::tungstenite::Error>
where
    S: AsyncWrite + AsyncRead + Unpin,
{
    log::info!("reconnecting socket");
    let _ = socket.close(None).await;
    let (socket, _) = ws::connect_async(url).await?;
    Ok(socket)
}

async fn handle_event(event: Event, rest: Arc<Client>) {
    if let Event::MessageCreate(msg) = event {
        if msg.content.contains("ping") {
            rest.create_message(msg.channel_id)
                .content("Pong")
                .unwrap()
                .exec()
                .await
                .unwrap();
        }
    }
}
