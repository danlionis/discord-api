use super::socket::GatewaySocket;
use crate::{
    error::{DiscordError, Error},
    model::gateway::{
        command::{self, GatewayCommand},
        Event, GatewayEvent, Hello, Ready,
    },
    rest::Rest,
};
use futures::{future::poll_fn, future::Either, prelude::*};
use std::{
    future::Future,
    pin::Pin,
    // sync::{Arc, RwLock},
    task::{Context, Poll},
    time::{self, Instant},
};
use time::Duration;
use tokio::{sync::mpsc, time::Interval};

/// Event handler
#[derive(Debug)]
pub struct Shard {
    token: String,
    rest: Rest,
    rx: mpsc::UnboundedReceiver<Event>,
    // state: Arc<RwLock<SharedConnState>>,
}

/// creates a new pair of `Shard` and a `Connection`
///
/// The `Connection` has to be spawned onto a runtime in order to connect to the gateway.
///
/// The `Shard` can be used to receive Events from and send Commands to the Gateway.
pub fn new(token: &str) -> (Shard, Connection) {
    with_rest_client(token, Rest::new(token))
}

/// same as `gateway::new` but does not create a new ApiClient
pub fn with_rest_client(token: &str, api: Rest) -> (Shard, Connection) {
    let (e_tx, e_rx) = mpsc::unbounded_channel();

    // let state = Arc::new(RwLock::new(SharedConnState { ping: None }));

    let shard = Shard {
        token: token.to_owned(),
        rest: api.clone(),
        rx: e_rx,
        // state: Arc::clone(&state),
    };

    let conn = ConnectionImpl {
        token: token.to_owned(),
        api: api.clone(),
        seq: 0,
        session_id: None,
        tx: e_tx,
        socket: GatewaySocket::new(),
        heartbeat_interval: None,
        hearbeat_ackd: true,
        // state,
    };

    (shard, Connection::new(conn))
}

impl Shard {
    // /// Ping to the gateway. `None` if not connected
    // pub fn ping(&self) -> Option<u128> {
    //     self.state.read().unwrap().ping
    // }

    /// Receive an Event
    ///
    /// returns `None` if the Connection has terminated
    pub async fn recv_event(&mut self) -> Option<Event> {
        poll_fn(|cx| self.poll_recv_event(cx)).await
    }

    /// Poll for the next Event
    ///
    /// returns `None` if the Connection has terminated
    pub fn poll_recv_event(&mut self, cx: &mut Context<'_>) -> Poll<Option<Event>> {
        self.rx.poll_recv(cx)
    }
}

impl Stream for Shard {
    type Item = Event;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.poll_recv_event(cx)
    }
}

/// Future that handles the connection
///
/// Spawning this Future will connect to the gateway, start heartbeating and forward Events to the
/// Shard
#[allow(missing_debug_implementations)]
#[must_use = "connection must be started with `.await` or polled"]
pub struct Connection(Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>);

impl Connection {
    fn new(conn: ConnectionImpl) -> Self {
        Connection(Box::pin(conn.start()))
    }
}

impl Future for Connection {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

/// State shared between a Shard and its Connection
#[derive(Debug)]
struct SharedConnState {
    /// ping from the last heartbeat, `None` if not connected
    pub ping: Option<u128>,
}

/// TODO: docs
#[derive(Debug)]
struct ConnectionImpl {
    token: String,
    api: Rest,
    seq: u64,
    session_id: Option<String>,
    tx: mpsc::UnboundedSender<Event>,
    socket: GatewaySocket,
    heartbeat_interval: Option<Interval>,
    hearbeat_ackd: bool,
    // state: Arc<RwLock<SharedConnState>>,
}

impl ConnectionImpl {
    /// Start the connection and start recieving events
    async fn start(mut self) -> Result<(), Error> {
        let mut gateway_url = self
            .api
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

        self.send_event(Event::Ready(ready))?;

        let mut last_heartbeat = None;

        loop {
            let interval = self.heartbeat_interval.as_mut().unwrap();
            // select between the heartbeat interval and a new gateway event
            let select = futures::future::select(
                poll_fn(|mut cx| interval.poll_tick(&mut cx)),
                self.socket.next(),
            );

            match select.await {
                Either::Left(_) => {
                    last_heartbeat = Some(Instant::now());
                    self.heartbeat().await;
                }
                Either::Right((Some(Ok(event)), _)) => {
                    // send each event in its raw json form
                    self.send_event(Event::Raw(event.1))?;

                    match event.0 {
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
                            // self.state.write().unwrap().ping =
                            //     Some(last_heartbeat.unwrap().elapsed().as_millis());

                            self.send_event(Event::Ping(
                                last_heartbeat.unwrap().elapsed().as_millis(),
                            ))?;

                            log::trace!("heartbeat ack");
                        }
                    };
                }
                Either::Right((Some(Err(Error::GatewayClosed(code))), _)) => {
                    log::warn!("connection closed: {:?}", code);

                    self.reconnect(&gateway_url).await?;
                }
                Either::Right((Some(Err(err)), _)) => {
                    log::error!("an error occured: {:?}", err);

                    self.reconnect(&gateway_url).await?;
                }
                Either::Right((None, _)) => return Err(Error::GatewayClosed(None)),
            }
        }
        Ok(())
    }

    /// initialize the connection to the gateway
    async fn init_connection(&mut self, gateway_url: &str) -> Result<(Hello, Ready), Error> {
        self.socket.connect(gateway_url).await?;

        let hello = match self.socket.next().await.expect("socket closed")?.0 {
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

        let ready = match self.socket.next().await.expect("socket closed").unwrap().0 {
            GatewayEvent::Dispatch(_, Event::Ready(ready)) => ready,
            GatewayEvent::InvalidSession(_reconnectable) => {
                panic!("invalid session");
            }
            _ => unreachable!(),
        };

        Ok((hello, ready))
    }

    /// reconnect to the gateway
    async fn reconnect(&mut self, gateway_url: &str) -> Result<u64, Error> {
        log::debug!("reconnecting");
        self.socket.reconnect(gateway_url).await?;

        log::debug!("sending hello");
        let hello = match self.socket.next().await.expect("socket closed")?.0 {
            GatewayEvent::Hello(h) => h,
            _ => unreachable!("hello should be first packet to recieve"),
        };

        log::debug!("sending resume");
        let resume = command::Resume {
            token: self.token.clone(),
            session_id: self
                .session_id
                .as_ref()
                .expect("resume only connections with session")
                .to_owned(),
            seq: self.seq,
        };
        self.socket.send(GatewayCommand::Resume(resume)).await?;

        self.heartbeat_interval = Some(tokio::time::interval(Duration::from_millis(
            hello.heartbeat_interval,
        )));

        Ok(hello.heartbeat_interval)
    }

    fn send_event(&self, event: Event) -> Result<(), Error> {
        self.tx
            .send(event)
            .map_err(|s| Error::DiscordError(DiscordError::SendError))
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
}
