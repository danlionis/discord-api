use super::socket::GatewaySocket;
use crate::{
    error::{CloseCode, DiscordError, Error},
    model::gateway::{
        command::{self, GatewayCommand},
        Event, GatewayEvent, Hello, Ready,
    },
    rest::RestClient,
};
use futures::{future::poll_fn, prelude::*, ready};
use std::{task::Poll, time};
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
            hearbeat_ackd: true,
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
    pub hearbeat_ackd: bool,
}

impl Connection {
    /// Initiate the connection and start recieving events
    ///
    /// This function has to be awaited in order to run it
    /// and only returns, if there is an error or the connection stops
    pub async fn run(&mut self) -> Result<(), Error> {
        let mut gateway_url = self
            .rest_client
            .get_gateway()
            .await
            .expect("could not get gateway url");
        gateway_url.push_str("/?v=8");

        let (hello, ready) = self.init_connection(&gateway_url).await?;

        let shard = ready.shard.unwrap_or((0, 0));
        log::info!("shard ready: [{}, {}]", shard.0, shard.1);
        log::info!("session ID: {}", &ready.session_id);
        log::info!("user: {}", ready.user.tag());
        log::debug!("heartbeat interval: {}s", hello.heartbeat_interval / 1000);
        self.session_id = Some(ready.session_id.clone());

        self.running_since = Some(time::Instant::now());

        self.send_event(Event::Ready(ready))?;

        loop {
            let interval = self.heartbeat_interval.as_mut().unwrap();
            // select between the heartbeat interval and a new gateway event
            let select = futures::future::select(
                poll_fn(|mut cx| interval.poll_tick(&mut cx)),
                self.socket.next(),
            );

            match select.await {
                future::Either::Left(_) => {
                    self.heartbeat().await;
                }
                future::Either::Right((Some(Ok(event)), _)) => match event {
                    GatewayEvent::Dispatch(seq, e) => {
                        log::debug!("dispatch event= {}", e.kind());
                        self.seq = seq;
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
                        log::warn!("invalid session; reconnectable: {}", reconnectable);
                        if reconnectable {
                            self.reconnect(&gateway_url).await?;
                        } else {
                            break;
                        }
                    }
                    GatewayEvent::Hello(_hello) => {}
                    GatewayEvent::HeartbeatAck => {
                        self.hearbeat_ackd = true;
                        log::trace!("heartbeat ack");
                    }
                },
                future::Either::Right((Some(Err(Error::GatewayClosed(code))), _)) => {
                    match code {
                        CloseCode::UnknownError | CloseCode::SessionTimedOut => {
                            log::warn!("connection closed: {:?}", code);
                            self.reconnect(&gateway_url).await?;
                        }
                        _ => {
                            panic!(code);
                        }
                    };
                }
                future::Either::Right((Some(Err(err)), _)) => {
                    log::error!("an error occured: {:?}", err);

                    self.reconnect(&gateway_url).await?;
                }
                future::Either::Right((None, _)) => panic!("socket closed"),
            }
        }
        Ok(())
    }

    /// initialize the connection to the gateway
    pub async fn init_connection(&mut self, gateway_url: &str) -> Result<(Hello, Ready), Error> {
        self.socket.connect(gateway_url).await?;

        let hello = match self.socket.next().await.unwrap().unwrap() {
            GatewayEvent::Hello(h) => h,
            _ => unreachable!("hello should be first packet to recieve"),
        };

        log::debug!("received initial hello");

        self.heartbeat_interval = Some(tokio::time::interval(Duration::from_millis(
            hello.heartbeat_interval,
        )));
        log::debug!("initialized heartbeat interval");

        self.socket
            .send(GatewayCommand::Identify(command::Identify::new(
                &self.token,
            )))
            .await?;

        log::debug!("sent identify payload");

        let ready = match self.socket.next().await.unwrap().unwrap() {
            GatewayEvent::Dispatch(_, Event::Ready(ready)) => ready,
            GatewayEvent::InvalidSession(_reconnectable) => {
                panic!("invalid session");
            }
            _ => unreachable!(),
        };

        Ok((hello, ready))
    }

    /// try reconnecting to the gatewy
    pub async fn try_reconnect(&mut self, gateway_url: &str, max_retries: u16) -> Option<u64> {
        let mut retries_left = max_retries;

        while retries_left > 0 {
            match self.reconnect(gateway_url).await {
                Ok(heartbeat) => return Some(heartbeat),
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    retries_left -= 1;
                }
            }
        }

        None
    }

    /// reconnect to the gateway
    pub async fn reconnect(&mut self, gateway_url: &str) -> Result<u64, Error> {
        log::debug!("reconnecting");
        self.socket.reconnect(gateway_url).await?;

        log::debug!("sending hello");
        let hello = match self.socket.next().await.unwrap().unwrap() {
            GatewayEvent::Hello(h) => h,
            _ => unreachable!("hello should be first packet to recieve"),
        };

        log::debug!("sending resume");
        let resume = command::Resume {
            token: self.token.clone(),
            session_id: self.session_id.as_ref().unwrap().to_owned(),
            seq: self.seq,
        };
        self.socket.send(GatewayCommand::Resume(resume)).await?;

        self.heartbeat_interval = Some(tokio::time::interval(Duration::from_millis(
            hello.heartbeat_interval,
        )));

        Ok(hello.heartbeat_interval)
    }

    fn send_event(&self, event: Event) -> Result<(), Error> {
        self.event_sender
            .send(event)
            .map_err(|_| Error::DiscordError(DiscordError::SendError))
    }

    async fn heartbeat(&mut self) {
        log::debug!("heartbeating seq= {}", self.seq);

        // TODO: handle heartbeat not ackd

        self.hearbeat_ackd = false;
        self.socket
            .send(GatewayCommand::Heartbeat(self.seq))
            .await
            .unwrap();
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.running_since.map(|r| r.elapsed()).unwrap_or_default()
    }
}

impl Future for Connection {
    type Output = Result<(), Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let fut = self.run();
        futures::pin_mut!(fut);
        Poll::Ready(ready!(fut.poll(cx)))
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
        // self.rx.poll_next_unpin(cx)
        self.rx.poll_recv(cx)
    }
}
