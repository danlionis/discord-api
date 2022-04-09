use discord::{
    model::gateway::Event,
    proto::Connection,
    rest::{Client, CreateMessageParams},
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{error::Error, sync::Arc, time::Duration};
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
    let mut gateway_info = rest.get_gateway_bot().await?;
    log::debug!("gateway: {:?}", gateway_info);
    gateway_info.url.push_str("/?v=9");
    let url = gateway_info.url.as_str();
    let (mut socket, _) = ws::connect_async(url).await?;

    // initialize connection and receive first hello packet
    let mut conn = Connection::new(token);
    let hello = socket.next().await.unwrap()?;
    let hello = hello.to_text()?;
    conn.recv_json(hello)?;

    // create heartbeat interval
    let mut interval = tokio::time::interval(Duration::from_millis(conn.heartbeat_interval()));

    loop {
        // reconnect the websocket if requested
        if conn.should_reconnect() {
            socket = reconnect_socket(socket, url).await?;
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
                        socket = reconnect_socket(socket, url).await?;
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
    conn: &mut Connection,
    rest: &Arc<Client>,
) -> Result<(), Box<dyn Error>> {
    // TODO: handle close frames

    conn.recv_json(msg.to_text()?)?;

    let mut content = String::new();

    for event in conn.events() {
        if let Event::MessageCreate(m) = &event {
            content = m.content.clone();
        }

        let rest = Arc::clone(rest);
        tokio::spawn(async move {
            let rest = Arc::as_ref(&rest);
            handle_event(event, rest).await
        });
    }

    if content.contains("resume") {
        log::warn!("request resume");
        conn.recv(discord::model::gateway::GatewayEvent::Reconnect);
    }
    if content.contains("reconnect") {
        log::warn!("request reconnect");
        conn.recv(discord::model::gateway::GatewayEvent::InvalidSession(false));
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

async fn handle_event(event: Event, rest: &Client) {
    if let Event::MessageCreate(msg) = event {
        if msg.content.contains("ping") {
            let params = CreateMessageParams::default().content("Pong");
            rest.create_message(msg.channel_id, params).await.unwrap();
        }
    }
}
