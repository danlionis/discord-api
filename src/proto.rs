//! Discord Gateway Protocol implementation as a state machine
use crate::model::gateway::{Event, GatewayCommand, GatewayEvent, Identify, Resume};
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

/// State of the gatway connection
#[derive(Clone, Debug)]
pub enum State {
    /// No connection established
    Closed,
    /// Waiting for identification
    Identify,
    /// Ready
    Ready,
    /// Attempt to reconnect and resume immediately
    Reconnect,
    /// Invalid session, client should reconnect, possibly resumable
    InvalidSession(
        /// resumable
        bool,
    ),
}

impl Connection {
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
            session_id: "".to_owned(),
        }
    }

    /// Returns the heartbeat interval.
    pub fn heartbeat_interval(&self) -> u64 {
        self.heartbeat_interval
    }

    /// get the current state
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Resume a reconnected session
    pub fn resume(&mut self) {
        self.state = State::Reconnect;
        let cmd = GatewayCommand::Resume(Resume::new(
            self.token.clone(),
            self.session_id.clone(),
            self.seq,
        ));
        self.send_queue.push_back(cmd);
    }

    /// Reconnect with initial handshake
    pub fn reconnect(&mut self) {
        self.state = State::Closed;
        self.seq = 0;
        self.session_id = "".to_owned();

        self.send_queue.clear();
    }

    /// queue a heartbeat packet to be sent to the gateway
    pub fn queue_heartbeat(&mut self) {
        self.send_queue
            .push_back(GatewayCommand::Heartbeat(self.seq))
    }

    /// Processes a discord event received from the gateway
    pub fn recv_json(&mut self, event: &str) -> Result<(), serde_json::Error> {
        let event = GatewayEvent::from_json_str(event)?;

        self.recv(event);

        Ok(())
    }

    /// Processes discord events received from the gateway
    pub fn recv(&mut self, event: GatewayEvent) {
        log::debug!("recv event: {}", event.kind());

        if let GatewayEvent::InvalidSession(resumable) = event {
            self.state = State::InvalidSession(resumable);
            return;
        }

        if let GatewayEvent::Reconnect = event {
            self.state = State::Reconnect;
            return;
        }

        if let GatewayEvent::Heartbeat(_) = event {
            self.queue_heartbeat();
            return;
        }

        if let GatewayEvent::HeartbeatAck = event {
            return;
        }

        self.state = match &self.state {
            State::Closed => {
                if let GatewayEvent::Hello(ref hello) = event {
                    log::debug!(
                        "recv hello: heartbeat_interval= {}",
                        hello.heartbeat_interval
                    );

                    self.heartbeat_interval = hello.heartbeat_interval;

                    self.send_queue
                        .push_back(GatewayCommand::Identify(Identify::new(&self.token)));
                    State::Identify
                } else {
                    log::debug!("expected hello packet");
                    State::Closed
                }
            }
            State::Identify => {
                if let GatewayEvent::Dispatch(seq, Event::Ready(ref ready)) = event {
                    log::info!(
                        "client ready: version= {} session_id= {} tag= {} shard= {:?}",
                        ready.version,
                        ready.session_id,
                        ready.user.tag(),
                        ready.shard
                    );

                    self.seq = seq;
                    self.session_id = ready.session_id.clone();

                    State::Ready
                } else {
                    log::warn!("received invalid packet");

                    State::InvalidSession(false)
                }
            }
            State::Ready => State::Ready,
            State::Reconnect => State::Reconnect,
            State::InvalidSession(resumable) => State::InvalidSession(*resumable),
        };

        // forward all dispatch events, no matter the state
        if let GatewayEvent::Dispatch(seq, event) = event {
            log::debug!("recv dispatch: kind= {} seq= {}", event.kind(), seq);

            self.seq = seq;
            self.recv_queue.push_back(event);
        }
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
        let cmd = self.send_queue.pop_front();
        log::trace!("sending command: {:?}", cmd);
        cmd
    }

    /// Creates a single discord command to be sent to the gateway
    ///
    /// `None` if nothing to send
    pub fn send_single_json(&mut self) -> Option<String> {
        self.send_single()
            .map(|cmd| serde_json::to_string(&cmd).unwrap())
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
        log::trace!("sending commands {:?}", self.send_queue);
        self.send_queue.drain(..)
    }

    /// Create an iterator of all the commands to be sent to the gateway
    ///
    /// The commands are already in json format
    pub fn send_iter_json<'a>(&'a mut self) -> impl Iterator<Item = String> + 'a {
        self.send_iter()
            .map(|cmd| serde_json::to_string(&cmd).unwrap())
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

    /// Add a command to the send queue
    pub fn enqueue_command(&mut self, cmd: GatewayCommand) {
        self.send_queue.push_back(cmd);
    }
}
