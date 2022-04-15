//! Gateway Protocol
//!
//! A protocol implementation that takes received gateway events, handles connection state and generates
//! commands to be sent back to the gateway.
//!
//! The application is responsible for any I/O (e.g. sockets, async runtime)
//! as well as the event loop
//!
//! # Connection
//! ```no_run
//! # use discord::proto::Connection;
//! # let TOKEN = "--your-token--";
//! let conn = Connection::new(TOKEN);
//! ```
//! At this point the [Connection] is still in the `Closed` state.
//! As soon as the connection receives the correct `Hello` message from the gateway it will
//! automatically initialize the connection.
//!
//! # Handle incoming events
//! The socket can pass incoming events to the connection with the [`recv()`] method upon which the
//! connection updates it's state
//!
//! ```no_run
//! # use discord::proto::Connection;
//! # use twilight_model::gateway::event::GatewayEvent;
//! # fn recv_from_socket() -> GatewayEvent { unimplemented!() };
//! # let mut conn = Connection::new("TOKEN");
//! let event = recv_from_socket();
//!
//! conn.recv(event);
//! ```
//! If the socket provides the data formatted as a JSON string the [`recv_json()`] method can be used instead.
//!
//! # Handle outgoing commands
//! The connection generates commands to be sent to the gateway with the `send` methods.
//! - [`send_iter()`] creates an iterator over all commands
//! - [`send()`] creates a single command
//!
//! ```no_run
//! # use discord::proto::{Connection, GatewayCommand};
//! # fn send_to_socket(cmd: GatewayCommand) { unimplemented!() };
//! # let mut conn = Connection::new("TOKEN");
//! for cmd in conn.send_iter() {
//!     send_to_socket(cmd);
//! }
//!
//! while let Some(cmd) = conn.send() {
//!     send_to_socket(cmd);
//! }
//! ```
//!
//! Both `send` methods are also available as `_json` variants to directly generate JSON serialized
//! commands
//!
//!
//! [`recv()`]: Connection::recv
//! [`recv_json()`]: Connection::recv_json
//! [`send_iter()`]: Connection::send_iter
//! [`send()`]: Connection::send

use serde::Serialize;
use twilight_model::gateway::{
    event::{DispatchEvent, Event, GatewayEvent},
    payload::outgoing::{
        identify::{IdentifyInfo, IdentifyProperties},
        Heartbeat, Identify, RequestGuildMembers, Resume, UpdatePresence, UpdateVoiceState,
    },
    Intents,
};

use crate::error::CloseCode;
use std::collections::VecDeque;

#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum GatewayCommand {
    Identify(Identify),
    Heartbeat(Heartbeat),
    RequestGulidMembers(RequestGuildMembers),
    Resume(Resume),
    UpdatePresence(UpdatePresence),
    UpdateVoiceState(UpdateVoiceState),
}

const RECV_QUEUE_SIZE: usize = 1;
const SEND_QUEUE_SIZE: usize = 1;

/// Discord gateway connection handler to
///
/// TODO: maybe rename to GatewayContext
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
    socket_closed: bool,
}

/// State of the gateway connection
///
/// This state is only used this way in this library and does not reflect the state of the gateway
/// connection itself
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Do not attempt to reconnect
    Failed(CloseCode),
}

impl Connection {
    /// Add a command to the send queue
    pub fn enqueue_command(&mut self, cmd: GatewayCommand) {
        self.send_queue.push_back(cmd);
    }

    /// Get the oldest event received from the gateway
    pub fn event(&mut self) -> Option<Event> {
        self.recv_queue.pop_front()
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
    pub fn events_mut(&mut self) -> &mut VecDeque<Event> {
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
            socket_closed: false,
        }
    }

    /// Queue a heartbeat packet to be sent to the gateway
    ///
    /// # Example
    /// ```
    /// # use discord::proto::{Connection, GatewayCommand};
    /// # use twilight_model::gateway::payload::outgoing::Heartbeat;
    /// # let mut conn = Connection::new("");
    /// conn.queue_heartbeat();
    /// assert_eq!(Some(GatewayCommand::Heartbeat(Heartbeat::new(0))), conn.send());
    /// ```
    pub fn queue_heartbeat(&mut self) {
        self.send_queue
            .push_back(GatewayCommand::Heartbeat(Heartbeat::new(self.seq)))
    }

    /// Process a close code received from the gateway websocket connection
    ///
    /// # Example
    /// ```
    /// # use discord::{proto::Connection, error::CloseCode};
    /// # let mut conn = Connection::new("");
    /// // connection closed normally
    /// conn.recv_close_code(1000u16);
    /// assert!(conn.should_reconnect());
    /// assert_eq!(None, conn.failed());
    ///
    /// // authentication failed code
    /// conn.recv_close_code(4005u16);
    /// assert!(!conn.should_reconnect());
    /// assert_eq!(Some(CloseCode::AuthenticationFailed), conn.failed());
    /// ```
    pub fn recv_close_code<T>(&mut self, code: T)
    where
        T: Into<u16>,
    {
        let code = CloseCode::from(code.into());
        log::debug!("recv_close_code: {}", code);
        self.socket_closed = true;

        self.state = if code.is_recoverable() {
            State::Resume
        } else {
            State::Failed(code)
        };
    }

    /// Processes discord events received from the gateway
    pub fn recv(&mut self, event: GatewayEvent) {
        log::trace!("gateway event= {:?}", event);

        // we've received an event so the socket can't be closed
        self.socket_closed = false;

        // an invalid session can potentially be resumed
        if let GatewayEvent::InvalidateSession(resumable) = event {
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
        if let GatewayEvent::Hello(heartbeat_interval) = event {
            log::debug!("recv hello: heartbeat_interval= {}", heartbeat_interval);

            self.heartbeat_interval = heartbeat_interval;

            self.state = match self.state {
                // if the connection was ready we try to resume first
                State::Resume | State::Ready => {
                    self.send_queue
                        .push_back(GatewayCommand::Resume(Resume::new(
                            self.seq,
                            self.session_id.clone(),
                            self.token.clone(),
                        )));
                    State::Replaying
                }
                // client got reconnected
                _ => {
                    self.send_queue
                        .push_back(GatewayCommand::Identify(Identify::new(IdentifyInfo {
                            compress: false,
                            token: self.token.clone(),
                            shard: Some([0, 1]),
                            intents: Intents::all(),
                            large_threshold: 100000,
                            presence: None,
                            properties: IdentifyProperties::new(
                                "twilight.rs",
                                "twilight.rs",
                                "OS",
                                "",
                                "",
                            ),
                        })));
                    State::Identify
                }
            }
        }

        if let GatewayEvent::Dispatch(seq, event) = event {
            match event.as_ref() {
                DispatchEvent::Ready(ready) => {
                    log::info!(
                        "client ready: version= {} session_id= {} tag= {}#{} shard= {:?}",
                        ready.version,
                        ready.session_id,
                        ready.user.name,
                        ready.user.discriminator(),
                        ready.shard
                    );

                    self.session_id = ready.session_id.clone();
                    self.state = State::Ready;
                }
                DispatchEvent::Resumed => {
                    log::info!("resumed: session_id= {}", self.session_id);
                    self.state = State::Ready;
                }
                _ => {}
            }
            log::debug!("recv dispatch: kind= {:?} seq= {}", event.kind(), seq);

            self.seq = seq;
            self.recv_queue.push_back((*event).into());
        }
    }

    /// Processes a discord event received from the gateway
    #[cfg(feature = "json")]
    pub fn recv_json(&mut self, input: &str) -> Result<(), serde_json::Error> {
        use serde::de::DeserializeSeed;
        use serde_json::Deserializer;
        use twilight_model::gateway::event::GatewayEventDeserializer;

        let deserializer = GatewayEventDeserializer::from_json(input).unwrap();
        let mut json_deserializer = Deserializer::from_str(input);
        let event = deserializer.deserialize(&mut json_deserializer).unwrap();

        self.recv(event);

        Ok(())
    }

    /// Create an iterator of all the commands to be sent to the gateway
    ///
    /// # Example
    /// ```no_run
    /// # use discord::proto::{Connection, GatewayCommand};
    /// # fn send_to_socket(cmd: GatewayCommand) { unimplemented!() };
    /// # let mut conn = Connection::new("token");
    /// for cmd in conn.send_iter() {
    ///     send_to_socket(cmd);
    /// }
    /// ```
    pub fn send_iter<'a>(&'a mut self) -> impl Iterator<Item = GatewayCommand> + 'a {
        log::trace!("sending commands {:?}", self.send_queue);
        self.send_queue.drain(..)
    }

    /// Create an iterator of all the commands to be sent to the gateway
    ///
    /// The commands will already be serialized in JSON.
    #[cfg(feature = "json")]
    pub fn send_iter_json<'a>(&'a mut self) -> impl Iterator<Item = String> + 'a {
        self.send_iter()
            .map(|cmd| serde_json::to_string(&cmd).unwrap())
    }

    /// Creates a single discord command to be sent to the gateway.
    ///
    /// Returns `None` if there is nothing to send.
    ///
    /// # Example
    /// ```
    /// # use discord::proto::{Connection, GatewayCommand};
    /// # fn send_to_socket(cmd: GatewayCommand) { unimplemented!() };
    /// # let mut conn = Connection::new("token");
    /// while let Some(cmd) = conn.send() {
    ///     send_to_socket(cmd);
    /// }
    /// ```
    pub fn send(&mut self) -> Option<GatewayCommand> {
        let cmd = self.send_queue.pop_front();
        log::trace!("sending command: {:?}", cmd);
        cmd
    }

    /// Creates a single discord command to be sent to the gateway.
    ///
    /// The command will already be serialized as JSON.
    ///
    /// # Example
    /// see [send]
    ///
    /// [send]: Connection::send
    #[cfg(feature = "json")]
    pub fn send_json(&mut self) -> Option<String> {
        self.send().map(|cmd| serde_json::to_string(&cmd).unwrap())
    }

    /// Returns true if the underlying gateway connection has to be reconnected
    pub fn should_reconnect(&self) -> bool {
        match self.state {
            State::Resume | State::Reconnect => true,
            State::Failed(_) => false,
            _ => self.socket_closed,
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

#[cfg(test)]
mod tests {
    use twilight_model::{
        gateway::payload::incoming::Ready,
        id::Id,
        oauth::{current_application_info::ApplicationFlags, PartialApplication},
        user::CurrentUser,
    };

    use super::*;

    fn create_default_ready() -> GatewayEvent {
        GatewayEvent::Dispatch(
            0,
            Box::new(DispatchEvent::Ready(Box::new(Ready {
                guilds: Vec::new(),
                version: 0,
                application: PartialApplication {
                    flags: ApplicationFlags::empty(),
                    id: Id::new(1),
                },
                user: create_default_user(),
                session_id: "session_id".into(),
                shard: Some([0, 1]),
            }))),
        )
    }

    fn create_default_user() -> CurrentUser {
        CurrentUser {
            id: Id::new(1),
            name: "username".into(),
            discriminator: 0000,
            avatar: None,
            bot: true,
            mfa_enabled: false,
            locale: None,
            verified: None,
            email: None,
            premium_type: None,
            public_flags: None,
            flags: None,
            accent_color: None,
            banner: None,
        }
    }

    #[test]
    fn test_connect() {
        let token = "TOKEN";
        let mut conn = Connection::new(token);
        assert_eq!(State::Closed, *conn.state());

        let hello = GatewayEvent::Hello(10);
        conn.recv(hello);
        assert_eq!(State::Identify, *conn.state());

        let identify = conn.send().unwrap();
        let identify = if let GatewayCommand::Identify(identify) = identify {
            identify
        } else {
            panic!("excpeted GatewayCommand::Identify, got {:?}", identify);
        };

        assert_eq!(token, &identify.d.token);
        assert_eq!(Some([0, 1]), identify.d.shard);

        let ready = create_default_ready();
        conn.recv(ready);
        assert_eq!(State::Ready, *conn.state());
    }

    #[test]
    fn test_resume() {
        let token = "TOKEN";
        let mut conn = Connection::new(token);
        let _session_id = "session_id".to_string();
        assert_eq!(State::Closed, *conn.state());

        let hello = GatewayEvent::Hello(10);
        conn.recv(hello);
        assert_eq!(State::Identify, *conn.state());
        assert_eq!(10, conn.heartbeat_interval());

        let _identify = conn.send().unwrap();

        let ready = create_default_ready();
        conn.recv(ready);
        assert_eq!(State::Ready, *conn.state());

        // simulate socket reconnect
        let hello = GatewayEvent::Hello(15);
        conn.recv(hello);
        assert_eq!(State::Replaying, *conn.state());
        assert_eq!(15, conn.heartbeat_interval());

        let ready = create_default_ready();
        conn.recv(ready);
        assert_eq!(State::Ready, *conn.state());
    }

    #[test]
    fn test_invalid_session() {
        let mut conn = Connection::new("TOKEN");
        assert_eq!(State::Closed, *conn.state());

        conn.recv(GatewayEvent::InvalidateSession(true));
        assert_eq!(State::Resume, *conn.state());
        assert!(conn.should_reconnect());

        conn.recv(GatewayEvent::InvalidateSession(false));
        assert_eq!(State::Reconnect, *conn.state());
        assert!(conn.should_reconnect());

        conn.recv(GatewayEvent::Reconnect);
        assert_eq!(State::Resume, *conn.state());
        assert!(conn.should_reconnect());
    }

    #[test]
    fn test_heartbeat_request() {
        let mut conn = Connection::new("TOKEN");
        assert_eq!(State::Closed, *conn.state());

        conn.recv(GatewayEvent::Heartbeat(1));

        assert_eq!(
            Some(GatewayCommand::Heartbeat(Heartbeat::new(0))),
            conn.send()
        );
    }
}
