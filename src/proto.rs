//! Discord Gateway Protocol implementation as a state machine
use crate::{
    error::CloseCode,
    model::gateway::{Event, GatewayCommand, GatewayEvent, Identify, Resume},
};
use serde_json;
use std::collections::VecDeque;

const RECV_QUEUE_SIZE: usize = 4;
const SEND_QUEUE_SIZE: usize = 4;

/// Discord gateway connection handler to
#[derive(Debug)]
pub struct Connection {
    token: String,
    /// sequence number
    seq: u64,
    session_id: String,
    heartbeat_interval: u64,
    recv_queue: VecDeque<Event>,
    send_queue: VecDeque<GatewayCommand>,
    state: State,
}

/// State of the gateway connection
#[derive(Clone, Debug)]
pub enum State {
    /// No connection established
    Closed,
    /// Waiting for identification
    Identify,
    /// Ready
    Ready,
    /// Attempt to reconnect
    Reconnect,
    /// Resume immediately
    Resume,
    /// Replaying missed events
    Replaying,
    /// The connection failed with the close code
    Failed(CloseCode),
}

impl Connection {
    /// Close connection
    ///
    /// Remember to close the connection with a close code of 1000 or 1001
    pub fn close(&mut self) -> u16 {
        *self = Connection::new(self.token.clone());
        1000
    }

    /// Reconnect
    pub fn reconnect(&mut self) {
        self.state = State::Reconnect;
    }

    /// Resume
    pub fn resume(&mut self) {
        self.state = State::Resume;
    }

    /// Add a command to the send queue
    pub fn enqueue_command(&mut self, cmd: GatewayCommand) {
        self.send_queue.push_back(cmd);
    }

    /// Create an iterator of all the events received from the gateway
    pub fn events<'a>(&'a mut self) -> impl Iterator<Item = Event> + 'a {
        self.recv_queue.drain(..)
    }

    /// Get a reference to the underlying event queue
    pub fn events_ref(&self) -> &VecDeque<Event> {
        &self.recv_queue
    }

    /// Get a mutable reference to the underlying event queue
    pub fn events_ref_mut(&mut self) -> &mut VecDeque<Event> {
        &mut self.recv_queue
    }

    /// Returns the heartbeat interval.
    pub fn heartbeat_interval(&self) -> u64 {
        self.heartbeat_interval
    }

    /// Create a new connection to the discord gateway
    pub fn new<S>(token: S) -> Self
    where
        S: Into<String>,
    {
        Connection {
            token: token.into(),
            seq: 0,
            heartbeat_interval: 0,
            recv_queue: VecDeque::with_capacity(RECV_QUEUE_SIZE),
            send_queue: VecDeque::with_capacity(SEND_QUEUE_SIZE),
            state: State::Closed,
            session_id: String::new(),
        }
    }

    /// Queue a heartbeat packet to be sent to the gateway
    pub fn queue_heartbeat(&mut self) {
        self.send_queue
            .push_back(GatewayCommand::Heartbeat(self.seq))
    }

    /// Process a close code received from the gateway websocket connection
    pub fn recv_close_code<T>(&mut self, code: T)
    where
        T: Into<u16>,
    {
        let code = CloseCode::from(code.into());
        log::debug!("recv_close_code: {}", code);

        self.state = if code.is_recoverable() {
            State::Reconnect
        } else {
            State::Failed(code)
        };
    }

    /// Processes discord events received from the gateway
    pub fn recv(&mut self, event: GatewayEvent) {
        log::debug!(
            "recv event: kind= {} state= {:?}",
            event.kind(),
            self.state()
        );
        log::trace!("gateway event= {:?}", event);

        // an invalid session can potentially be resumed
        if let GatewayEvent::InvalidSession(resumable) = event {
            self.state = if resumable {
                State::Resume
            } else {
                State::Reconnect
            };
            return;
        }

        // a reconnect event can be resumed after the socket has reconnected to the gateway
        if let GatewayEvent::Reconnect = event {
            self.state = State::Resume;
            return;
        }

        // queue a heartbeat if it was requested
        if let GatewayEvent::Heartbeat(_) = event {
            self.queue_heartbeat();
            return;
        }

        // do nothing if hearbeat was ack'd
        if let GatewayEvent::HeartbeatAck = event {
            // TODO: maybe keep track if the heartbeat was ack'd and if not send it again
            return;
        }

        // hello events indicate that the underlying socket has (re)connected to the gateway
        if let GatewayEvent::Hello(ref hello) = event {
            log::debug!(
                "recv hello: heartbeat_interval= {}",
                hello.heartbeat_interval
            );

            self.heartbeat_interval = hello.heartbeat_interval;

            self.state = match self.state {
                State::Resume => {
                    self.send_queue
                        .push_back(GatewayCommand::Resume(Resume::new(
                            self.token.clone(),
                            self.session_id.clone(),
                            self.seq,
                        )));
                    State::Replaying
                }
                // client got reconnected
                _ => {
                    self.send_queue
                        .push_back(GatewayCommand::Identify(Identify::new(&self.token)));
                    State::Identify
                }
            }
        }

        if let GatewayEvent::Dispatch(_seq, Event::Ready(ref ready)) = event {
            log::info!(
                "client ready: version= {} session_id= {} tag= {} shard= {:?}",
                ready.version,
                ready.session_id,
                ready.user.tag(),
                ready.shard
            );

            self.session_id = ready.session_id.clone();
            self.state = State::Ready;
        }

        if let GatewayEvent::Dispatch(_seq, Event::Resume) = event {
            log::info!("resumed: session_id= {}", self.session_id);
            self.state = State::Ready;
        }

        // forward all dispatch events, no matter the state
        if let GatewayEvent::Dispatch(seq, event) = event {
            log::debug!("recv dispatch: kind= {} seq= {}", event.kind(), seq);

            self.seq = seq;
            self.recv_queue.push_back(event);
        }
    }

    /// Processes a discord event received from the gateway
    pub fn recv_json(&mut self, event: &str) -> Result<(), serde_json::Error> {
        let event = GatewayEvent::from_json_str(event)?;

        self.recv(event);

        Ok(())
    }

    /// Create an iterator of all the commands to be sent to the gateway
    ///
    /// # Example
    /// ```
    /// # use discord::proto::Connection;
    /// # use discord::model::gateway::GatewayCommand;
    /// # let mut conn = Connection::new("token");
    /// conn.queue_heartbeat();
    ///
    /// assert_eq!(Some(GatewayCommand::Heartbeat(0)), conn.send_iter().next());
    /// assert_eq!(None, conn.send_iter().next());
    /// ```
    pub fn send_iter<'a>(&'a mut self) -> impl Iterator<Item = GatewayCommand> + 'a {
        if self.is_closed() {
            log::trace!("connection closed, not sending: {:?}", self.send_queue);
            self.send_queue.clear();
        } else {
            log::trace!("sending commands {:?}", self.send_queue);
        }
        self.send_queue.drain(..)
    }

    /// Create an iterator of all the commands to be sent to the gateway
    ///
    /// The commands are already in json format
    pub fn send_iter_json<'a>(&'a mut self) -> impl Iterator<Item = String> + 'a {
        self.send_iter()
            .map(|cmd| serde_json::to_string(&cmd).unwrap())
    }

    /// Creates a single discord command to be sent to the gateway
    ///
    /// `None` if nothing to send
    ///
    /// # Example
    /// ```
    /// # use discord::proto::Connection;
    /// # use discord::model::gateway::GatewayCommand;
    /// # let mut conn = Connection::new("token");
    /// conn.queue_heartbeat();
    ///
    /// assert_eq!(Some(GatewayCommand::Heartbeat(0)), conn.send_single());
    /// assert_eq!(None, conn.send_single());
    /// ```
    pub fn send_single(&mut self) -> Option<GatewayCommand> {
        if self.is_closed() {
            return None;
        }

        let cmd = self.send_queue.pop_front();
        log::trace!("sending command: {:?}", cmd);
        cmd
    }

    /// Creates a single discord command to be sent to the gateway
    ///
    /// # Example
    /// see [send_single]
    ///
    /// [send_single]: Connection::send_single
    pub fn send_single_json(&mut self) -> Option<String> {
        self.send_single()
            .map(|cmd| serde_json::to_string(&cmd).unwrap())
    }

    /// Returns true if the underlying gateway connection has to be reconnected
    pub fn should_reconnect(&self) -> bool {
        match self.state {
            State::Resume | State::Reconnect => true,
            _ => false,
        }
    }

    /// get the current state
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns true if the connection is closed
    pub fn is_closed(&self) -> bool {
        match self.state {
            State::Closed | State::Reconnect | State::Resume => true,
            _ => false,
        }
    }

    /// Returns [Some(CloseCode)] if the connection failed
    pub fn failed(&self) -> Option<CloseCode> {
        match self.state {
            State::Failed(code) => Some(code),
            _ => None,
        }
    }
}
