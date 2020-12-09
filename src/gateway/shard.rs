use super::socket::GatewaySocket;
use crate::error::{DiscordError, Error};
use crate::model::gateway::command::{self, GatewayCommand};
use crate::model::gateway::{intents, Event, GatewayEvent, Ready};
use crate::rest::RestClient;
use futures::{future, SinkExt, Stream, StreamExt};
use std::time;
use time::Duration;
use tokio::{sync::mpsc, time::Interval};

pub struct Shard {
    token: String,
    rest_client: RestClient,
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
            heartbeat_interval: None,
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
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub socket: GatewaySocket,
    pub heartbeat_interval: Option<Interval>,
}

impl Connection {
    /// Initiate the connection and start recieving events
    ///
    /// This function has to be awaited in order to run it
    /// and only returns, if there is an error or the connection stops
    pub async fn run(mut self) -> Result<(), Error> {
        let mut gateway_url = self.rest_client.get_gateway().await.unwrap();
        gateway_url.push_str("/?v=8");

        let ready = self.init_connection(&gateway_url).await?;

        let shard = ready.shard.unwrap_or((0.into(), 0));
        log::info!("Shard ready: [{}, {}]", shard.0, shard.1);
        log::info!("Session ID: {}", &ready.session_id);
        log::info!("User: {}", ready.user.tag());
        self.session_id = Some(ready.session_id.clone());

        self.running_since = Some(time::Instant::now());

        self.send_event(Event::Ready(ready))?;

        loop {
            // select between the heartbeat interval and a new gateway event
            let select = futures::future::select(
                self.heartbeat_interval.as_mut().unwrap().next(),
                self.socket.next(),
            );

            match select.await {
                future::Either::Left(_) => {
                    self.heartbeat().await;
                }
                future::Either::Right((Some(event), _)) => match event {
                    GatewayEvent::Dispatch(seq, e) => {
                        self.seq = seq;
                        match &e {
                            Event::MessageCreate(m) => {
                                if m.content.as_str() == "reconnect" {
                                    self.reconnect(&gateway_url).await?;
                                }
                            }
                            _ => {}
                        }
                        self.send_event(e)?;
                    }
                    GatewayEvent::Heartbeat(_) => {
                        log::debug!("heartbeat requested");
                        self.heartbeat().await;
                    }
                    GatewayEvent::Reconnect => {
                        log::warn!("reconnect requested");
                        self.reconnect(&gateway_url).await?;
                    }
                    GatewayEvent::InvalidSession(reconnectable) => {
                        log::warn!("invalid session");
                        if reconnectable {
                            self.reconnect(&gateway_url).await?;
                        } else {
                            break;
                        }
                    }
                    GatewayEvent::Hello(_hello) => {}
                    GatewayEvent::HeartbeatAck => {}
                },
                _ => panic!("weird state"),
            }
        }
        // unimplemented!()
        Ok(())
    }

    /// initialize the connection to the gateway
    pub async fn init_connection(&mut self, gateway_url: &str) -> Result<Ready, Error> {
        self.socket.connect(gateway_url).await?;

        let hello = match self.socket.next().await.unwrap() {
            GatewayEvent::Hello(h) => h,
            _ => unreachable!("hello should be first packet to recieve"),
        };

        self.heartbeat_interval = Some(tokio::time::interval(Duration::from_millis(
            hello.heartbeat_interval,
        )));

        self.socket
            .send(GatewayCommand::Identify(identify_payload(&self.token)))
            .await?;

        let ready = match self.socket.next().await.unwrap() {
            GatewayEvent::Dispatch(_, Event::Ready(ready)) => ready,
            GatewayEvent::InvalidSession(_reconnectable) => {
                dbg!(_reconnectable);
                todo!()
            }
            _ => unreachable!(),
        };

        Ok(ready)
    }

    pub async fn reconnect(&mut self, gateway_url: &str) -> Result<u64, Error> {
        self.socket.reconnect(gateway_url).await?;

        let hello = match self.socket.next().await.unwrap() {
            GatewayEvent::Hello(h) => h,
            _ => unreachable!("hello should be first packet to recieve"),
        };
        self.socket
            .send(GatewayCommand::Resume(resume_payload(
                &self.token,
                self.session_id.as_ref().unwrap(),
                self.seq,
            )))
            .await?;

        Ok(hello.heartbeat_interval)
    }

    fn send_event(&self, event: Event) -> Result<(), Error> {
        self.event_sender
            .send(event)
            .map_err(|_| Error::DiscordError(DiscordError::SendError))
    }

    async fn heartbeat(&mut self) {
        log::debug!("heartbeating seq= {}", self.seq);
        self.socket
            .send(GatewayCommand::Heartbeat(self.seq))
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
//         dbg!("polling");
//         let fut = self.run();
//         pin_mut!(fut);
//         fut.poll(cx)
//     }
// }

fn identify_payload(token: &str) -> command::Identify {
    let properties = command::ConnectionProperties {
        os: "donbot".to_owned(),
        device: "donbot".to_owned(),
        browser: "donbot".to_owned(),
    };
    command::Identify {
        token: token.to_owned(),
        properties,
        large_threshold: None,
        shard: (0, 1),
        presence: None,
        intents: intents::ALL,
    }
}

fn resume_payload(token: &str, session_id: &str, seq: u64) -> command::Resume {
    command::Resume {
        token: token.to_owned(),
        session_id: session_id.to_owned(),
        seq,
    }
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl Events {
    pub fn send_test(&mut self) {}
}

impl Stream for Events {
    type Item = Event;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}
