use super::socket::GatewaySocket;
use crate::model::gateway::DispatchEvent;
use crate::{error::Error, model::gateway::GatewayEvent};
// use crate::model::gateway::EventType;
use crate::rest::RestClient;
use futures::prelude::*;
use std::time;
use tokio::sync::mpsc;

pub struct Shard {
    token: String,
    rest_client: RestClient,
    // connection: Connection,
    //event_reciever: mpsc::UnboundedReceiver<Event>,
}

impl Shard {
    pub fn new(token: &str) -> Self {
        Self::with_rest_client(token, RestClient::new(token))
    }

    pub fn with_rest_client(token: &str, rest_client: RestClient) -> Self {
        Shard {
            token: token.to_string(),
            rest_client,
        }
    }

    pub fn connection(&self) -> (Connection, Events) {
        let (tx, rx) = mpsc::unbounded_channel();
        let conn = Connection {
            token: self.token.to_owned(),
            rest_client: self.rest_client.clone(),
            seq: 0,
            session_id: None,
            running_since: None,
            event_sender: tx,
            socket: GatewaySocket::new(),
        };

        let events = Events { rx };

        (conn, events)
    }
}

pub struct Connection {
    pub token: String,
    pub rest_client: RestClient,
    pub seq: u64,
    pub running_since: Option<time::Instant>,
    pub session_id: Option<String>,
    pub event_sender: mpsc::UnboundedSender<DispatchEvent>,
    pub socket: GatewaySocket,
}

impl Connection {
    /// Initiate the connection and start recieving events
    ///
    /// This function has to be awaited in order to run it
    /// and only returns, if there is an error or the connection stops
    pub async fn run(mut self) -> Result<(), Error> {
        let mut gateway_url = self.rest_client.get_gateway().await.unwrap();
        gateway_url.push_str("/?v=8");

        let (heartbeat_interval, ready) = self.socket.connect(&gateway_url, &self.token).await?;

        let shard = ready.shard.unwrap_or((0.into(), 0));
        log::info!("Shard ready: [{}, {}]", shard.0, shard.1);
        log::info!("Session ID: {}", &ready.session_id);
        log::info!("User: {}", ready.user.tag());
        self.session_id = Some(ready.session_id.clone());

        let mut interval = tokio::time::interval(time::Duration::from_millis(heartbeat_interval));

        self.running_since = Some(time::Instant::now());

        self.send_event(DispatchEvent::Ready(ready)).unwrap();

        loop {
            let select = futures::future::select(interval.next(), self.socket.next());

            match select.await {
                future::Either::Left(_) => {
                    self.heartbeat().await;
                }
                future::Either::Right((Some(event), _)) => match event {
                    GatewayEvent::Dispatch(seq, e) => {
                        self.seq = seq;
                        match &e {
                            DispatchEvent::MessageCreate(m) => {
                                if m.content.as_str() == "reconnect" {
                                    self.socket
                                        .reconnect(
                                            &gateway_url,
                                            &self.token,
                                            self.session_id.as_ref().unwrap(),
                                            self.seq,
                                        )
                                        .await
                                        .unwrap();
                                }
                            }
                            _ => {}
                        }
                        self.send_event(e).expect("could not send event");
                    }
                    GatewayEvent::Heartbeat(_) => {
                        self.heartbeat().await;
                    }
                    GatewayEvent::Reconnect => {
                        log::warn!("reconnect requested");
                        self.socket
                            .reconnect(
                                &gateway_url,
                                &self.token,
                                self.session_id.as_ref().unwrap(),
                                self.seq,
                            )
                            .await?;
                    }
                    GatewayEvent::RequestGuildMembers => {}
                    GatewayEvent::InvalidSession(_reconnectable) => {
                        log::warn!("invalid session");
                        break;
                    }
                    GatewayEvent::Hello(_hello) => {}
                    GatewayEvent::HeartbeatAck => {}
                    // GatewayEvent::Identify => {}
                    // GatewayEvent::PresenceUpdate => {}
                    // GatewayEvent::VoiceStateUpdate => {}
                    // GatewayEvent::Resume => {}
                    _ => unreachable!(),
                },
                _ => panic!("weird state"),
            }
        }
        // unimplemented!()
        Ok(())
    }

    fn send_event(
        &self,
        event: DispatchEvent,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<DispatchEvent>> {
        self.event_sender.send(event)
    }

    async fn heartbeat(&mut self) {
        log::debug!("heartbeating seq= {}", self.seq);
        self.socket.send(heartbeat_payload(self.seq)).await.unwrap();
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.running_since.map(|r| r.elapsed()).unwrap_or_default()
    }
}

fn heartbeat_payload(seq: u64) -> String {
    format!(r#"{{"op": 1, "d": {}}}"#, seq)
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<DispatchEvent>,
}

impl Stream for Events {
    type Item = DispatchEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}
