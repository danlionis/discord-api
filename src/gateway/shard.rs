use crate::error::Error;
use crate::event::Event;
use crate::model::{self, gateway, Message, Presence, VoiceState};
use crate::rest::RestClient;
use async_tungstenite::{tokio as ws, tungstenite::Message as WsMessage, WebSocketStream};
use futures::prelude::*;
use std::sync::Arc;
use std::time;
use tokio::sync::mpsc;

pub struct Connection {
    pub token: String,
    pub rest_client: Arc<RestClient>,
    // pub stream: Option<WebSocketStream<ws::ConnectStream>>,
    pub seq: u64,
    pub running_since: Option<time::Instant>,
    pub session_id: Option<String>,
    pub event_sender: mpsc::UnboundedSender<Event>,
}

impl Connection {
    /// Initiate the connection and start recieving events
    ///
    /// This function has to be awaited in order to run it
    /// and only returns, if there is an error or the connection stops
    pub async fn run(mut self) -> Result<(), Error> {
        let ws_url = self.rest_client.get_gateway().await?;

        // connect to websocket
        let (mut stream, _) = ws::connect_async(ws_url).await.unwrap();
        log::debug!("Connected to websocket");

        // wait for hello packet
        let msg = stream.next().await.unwrap().unwrap().into_text().unwrap();
        let msg: gateway::GatewayPayload<gateway::Hello> = serde_json::from_str(&msg).unwrap();
        let heartbeat_interval = msg.data.heartbeat_interval;
        log::debug!("got heartbeat interval {}", heartbeat_interval);

        log::debug!("sending identification packet");
        stream
            .send(WsMessage::text(identify_payload(&self.token)))
            .await
            .unwrap();

        // wait for ready packet
        let msg = stream.next().await.unwrap().unwrap().into_text().unwrap();
        let ready: gateway::GatewayPayload<gateway::Ready> = serde_json::from_str(&msg).unwrap();
        let shard = ready.data.shard.unwrap_or((0.into(), 0));
        log::info!("Shard ready: [{}, {}]", shard.0, shard.1);
        log::info!("Session ID: {}", ready.data.session_id);
        log::info!("User: {}", ready.data.user.tag());
        self.session_id = Some(ready.data.session_id);

        let mut interval = tokio::time::interval(time::Duration::from_millis(heartbeat_interval));
        // self.stream = Some(stream);

        self.running_since = Some(time::Instant::now());

        self.send_event(Event::Ready(ready.data.user)).unwrap();

        loop {
            let select = futures::future::select(interval.next(), stream.next());
            match select.await {
                future::Either::Left(_) => {
                    self.heartbeat(&mut stream).await;
                }
                future::Either::Right((Some(event), _)) => {
                    let event = event?.into_text().unwrap();
                    let payload: gateway::GatewayPayload<serde_json::Value> =
                        serde_json::from_str(&event).unwrap();

                    if let Some(seq) = payload.seq {
                        self.seq = seq;
                    }

                    log::debug!(
                        "Recieved event: opcode= {:2} kind= {}",
                        payload.opcode,
                        payload.kind.as_deref().unwrap_or("None")
                    );

                    if let Some(kind) = &payload.kind {
                        match kind.as_str() {
                            "GUILD_CREATE" => {
                                let guild: model::Guild =
                                    serde_json::from_value(payload.data).unwrap();

                                self.send_event(Event::GuildCreate(guild)).unwrap();
                            }
                            "VOICE_STATE_UPDATE" => {
                                let voice_state: VoiceState =
                                    serde_json::from_value(payload.data).unwrap();
                                self.send_event(Event::VoiceStateUpdate(voice_state))
                                    .unwrap();
                            }
                            "MESSAGE_CREATE" => {
                                let message: Message =
                                    serde_json::from_value(payload.data).unwrap();
                                self.send_event(Event::MessageCreate(message)).unwrap();
                            }
                            "MESSAGE_UPDATE" => {
                                let message: Message =
                                    serde_json::from_value(payload.data).unwrap();
                                self.send_event(Event::MessageUpdate(message)).unwrap();
                            }
                            "MESSAGE_DELETE" => {
                                let delete: gateway::MessageDelete =
                                    serde_json::from_value(payload.data).unwrap();
                                self.send_event(Event::MessageDelete(delete)).unwrap();
                            }
                            "PRESENCE_UPDATE" => {
                                let presence: Presence = serde_json::from_value(payload.data)?;
                                self.send_event(Event::PresenceUpdate(presence)).unwrap();
                            }
                            _ => {
                                log::warn!(
                                    "Unknown event opcode= {} kind= {}",
                                    payload.opcode,
                                    payload.kind.as_deref().unwrap_or("None")
                                );
                            }
                        }
                    }
                }
                _ => panic!("weird state"),
            }
        }
    }

    fn send_event(&self, event: Event) -> Result<(), tokio::sync::mpsc::error::SendError<Event>> {
        self.event_sender.send(event)
    }

    async fn heartbeat(&self, stream: &mut WebSocketStream<ws::ConnectStream>) {
        log::debug!("Sending heartbeat");
        stream
            .send(WsMessage::text(heartbeat_payload(self.seq)))
            .await
            .unwrap();
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.running_since.map(|r| r.elapsed()).unwrap_or_default()
    }
}

// impl Future for Connection {
//     type Output = Result<(), Error>;

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         self.run().poll_unpin(cx)
//     }
// }

pub struct Events {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl tokio::stream::Stream for Events {
    type Item = Event;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

pub struct Shard {
    token: String,
    rest_client: Arc<RestClient>,
    // connection: Connection,
    //event_reciever: mpsc::UnboundedReceiver<Event>,
}

impl Shard {
    pub fn new(token: &str) -> Self {
        Self::with_rest_client(token, Arc::new(RestClient::new(token)))
    }

    pub fn with_rest_client(token: &str, rest_client: Arc<RestClient>) -> Self {
        Shard {
            token: token.to_string(),
            rest_client: rest_client,
        }
    }

    pub fn connection(&self) -> (Connection, Events) {
        let (tx, rx) = mpsc::unbounded_channel();
        let conn = Connection {
            token: self.token.to_owned(),
            rest_client: Arc::clone(&self.rest_client),
            // stream: None,
            seq: 0,
            session_id: None,
            running_since: None,
            event_sender: tx,
        };

        let events = Events { rx };

        (conn, events)
    }
}

// pub fn connect(token: &str) -> (Shard, Connection) {
//     let (tx, rx) = mpsc::unbounded_channel();
//     let rest_client = Arc::new(RestClient::new(token));
//     let conn = Connection {
//         token: token.to_owned(),
//         rest_client: Arc::clone(&rest_client),
//         stream: None,
//         seq: 0,
//         session_id: None,
//         running_since: None,
//         event_sender: tx,
//     };
//
//     let shard = Shard {
//         rest_client: Arc::clone(&rest_client),
//         event_reciever: rx,
//     };
//
//     (shard, conn)
// }

fn identify_payload(token: &str) -> String {
    format!(
        r#"{{ "op": 2, "d": {{ "token": "{}", "properties": {{ "$os": "linux", "$browser": "donbot", "$device": "donbot" }} }} }}"#,
        token
    )
}

fn heartbeat_payload(seq: u64) -> String {
    format!(r#"{{"op": 1, "d": {}}}"#, seq)
    // let res = serde_json::json!({
    //     "op": 1, "d": seq
    // });
    // serde_json::to_string(&res).unwrap()
}
