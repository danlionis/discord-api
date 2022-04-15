//! Gateway Protocol
//!
//! A protocol implementation that takes received gateway events, handles connection state and generates
//! commands to be sent back to the gateway.
//!
//! The application is responsible for any I/O (e.g. sockets, async runtime)
//! as well as the event loop
//!
//! # Connection
//! Create a context with all intents.
//! ```no_run
//! # use discord::proto::{GatewayContext, Config};
//! # use twilight_model::gateway::Intents;
//! let mut conn = GatewayContext::new(Config::new("TOKEN", Intents::all()));
//! ```
//! At this point the [GatewayContext] is still in the `Closed` state.
//! As soon as the connection receives the correct `Hello` message from the gateway it will
//! automatically initialize the connection.
//!
//! # Handle incoming events
//! The socket can pass incoming events to the connection with the [`recv()`] method upon which the
//! connection updates it's state
//!
//! ```no_run
//! # use discord::proto::GatewayContext;
//! # use twilight_model::gateway::{Intents, event::GatewayEvent};
//! # fn recv_from_socket() -> GatewayEvent { unimplemented!() };
//! # let mut conn = GatewayContext::new(("TOKEN", Intents::empty()));
//! let event = recv_from_socket();
//!
//! conn.recv(event);
//!
//! ```
//! If the socket provides the data formatted as a JSON string the [`recv_json()`] method can be used instead.
//!
//! # Handle outgoing commands
//! The connection generates commands to be sent to the gateway with the `send` methods.
//! - [`send_iter()`] creates an iterator over all commands
//! - [`send()`] creates a single command
//!
//! ```no_run
//! # use discord::{proto::{GatewayContext, GatewayCommand}, model::gateway::Intents};
//! # fn send_to_socket(cmd: GatewayCommand) { unimplemented!() };
//! # let mut ctx = GatewayContext::new(("TOKEN", Intents::empty()));
//! for cmd in ctx.send_iter() {
//!     send_to_socket(cmd);
//! }
//!
//! while let Some(cmd) = ctx.send() {
//!     send_to_socket(cmd);
//! }
//! ```
//!
//! Both `send` methods are also available as `_json` variants to directly generate JSON serialized
//! commands
//!
//! # Heartbeating
//! The application is responsible to maintain a heartbeat timer and queue heartbeat packets at the
//! corret time.
//!
//! To obtain the timer interval and queue the heartbeat message use the following methods:
//! ```
//! use discord::{proto::GatewayContext, model::gateway::Intents};
//! # let mut ctx = GatewayContext::new(("", Intents::empty()));
//! let heartbeat_interval = ctx.heartbeat_interval();
//! ctx.queue_heartbeat();
//! ```
//!
//!
//! [`recv()`]: GatewayContext::recv
//! [`recv_json()`]: GatewayContext::recv_json
//! [`send_iter()`]: GatewayContext::send_iter
//! [`send()`]: GatewayContext::send

use crate::{error::CloseCode, LIB_NAME};
use serde::Serialize;
use std::collections::VecDeque;
use twilight_model::gateway::{
    event::{DispatchEvent, Event, GatewayEvent},
    payload::outgoing::{
        identify::{IdentifyInfo, IdentifyProperties},
        Heartbeat, Identify, RequestGuildMembers, Resume, UpdatePresence, UpdateVoiceState,
    },
};

mod config;
pub use config::*;

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
#[derive(Debug)]
pub struct GatewayContext {
    config: Config,
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

impl GatewayContext {
    /// Add a command to the send queue
    pub fn enqueue_command(&mut self, cmd: GatewayCommand) {
        self.send_queue.push_back(cmd);
    }

    /// Get the oldest event received from the gateway
    pub fn event(&mut self) -> Option<Event> {
        self.recv_queue.pop_front()
    }

    /// Create an iterator of all the events received from the gateway
    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
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

    /// Create a new GatewayContext to the discord gateway
    pub fn new<C>(config: C) -> Self
    where
        C: Into<Config>,
    {
        GatewayContext {
            config: config.into(),
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
    /// # use discord::proto::{GatewayContext, GatewayCommand};
    /// # use twilight_model::gateway::{Intents, payload::outgoing::Heartbeat};
    /// # let mut conn = GatewayContext::new(("TOKEN", Intents::empty()));
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
    /// # use discord::{proto::GatewayContext, error::CloseCode};
    /// # use twilight_model::gateway::Intents;
    /// # let mut conn = GatewayContext::new(("TOKEN", Intents::empty()));
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

        match event {
            // an invalid session can potentially be resumed
            GatewayEvent::InvalidateSession(resumable) => {
                self.state = if resumable {
                    State::Resume
                } else {
                    State::Reconnect
                };
            }
            // a reconnect event can be resumed after the socket has reconnected to the gateway
            GatewayEvent::Reconnect => {
                self.state = State::Resume;
            }
            // queue a heartbeat if it was requested
            GatewayEvent::Heartbeat(_) => {
                self.queue_heartbeat();
            }
            // do nothing if hearbeat was ack'd
            GatewayEvent::HeartbeatAck => {
                // TODO: maybe keep track if the heartbeat was ack'd and if not send it again
            }
            // hello events indicate that the underlying socket has (re)connected to the gateway
            GatewayEvent::Hello(heartbeat_interval) => {
                log::debug!("recv hello: heartbeat_interval= {}", heartbeat_interval);

                self.heartbeat_interval = heartbeat_interval;

                self.state = match self.state {
                    // if the connection was ready we try to resume first
                    State::Resume | State::Ready => {
                        self.send_queue
                            .push_back(GatewayCommand::Resume(Resume::new(
                                self.seq,
                                self.session_id.clone(),
                                self.config.token.clone(),
                            )));
                        State::Replaying
                    }
                    // client got reconnected
                    _ => {
                        self.send_queue
                            .push_back(GatewayCommand::Identify(Identify::new(IdentifyInfo {
                                compress: false,
                                token: self.config.token.clone(),
                                shard: Some(self.config.shard),
                                intents: self.config.intents,
                                large_threshold: 100000,
                                presence: self.config.presence.clone(),
                                properties: IdentifyProperties::new(
                                    LIB_NAME,
                                    LIB_NAME,
                                    std::env::consts::OS,
                                    "",
                                    "",
                                ),
                            })));
                        State::Identify
                    }
                }
            }
            GatewayEvent::Dispatch(seq, event) => {
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
    /// # use discord::proto::{GatewayContext, GatewayCommand};
    /// # use twilight_model::gateway::Intents;
    /// # fn send_to_socket(cmd: GatewayCommand) { unimplemented!() };
    /// # let mut conn = GatewayContext::new(("TOKEN", Intents::empty()));
    /// for cmd in conn.send_iter() {
    ///     send_to_socket(cmd);
    /// }
    /// ```
    pub fn send_iter(&mut self) -> impl Iterator<Item = GatewayCommand> + '_ {
        log::trace!("sending commands {:?}", self.send_queue);
        self.send_queue.drain(..)
    }

    /// Create an iterator of all the commands to be sent to the gateway
    ///
    /// The commands will already be serialized in JSON.
    #[cfg(feature = "json")]
    pub fn send_iter_json(&mut self) -> impl Iterator<Item = String> + '_ {
        self.send_iter()
            .map(|cmd| serde_json::to_string(&cmd).unwrap())
    }

    /// Creates a single discord command to be sent to the gateway.
    ///
    /// Returns `None` if there is nothing to send.
    ///
    /// # Example
    /// ```
    /// # use discord::proto::{GatewayContext, GatewayCommand};
    /// # use twilight_model::gateway::Intents;
    /// # fn send_to_socket(cmd: GatewayCommand) { unimplemented!() };
    /// # let mut conn = GatewayContext::new(("TOKEN", Intents::empty()));
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
    /// [send]: GatewayContext::send
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
        matches!(self.state, State::Closed | State::Reconnect | State::Resume)
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
        gateway::{payload::incoming::Ready, Intents},
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
        let mut conn = GatewayContext::new((token, Intents::empty()));
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
        let mut conn = GatewayContext::new((token, Intents::empty()));
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
        let token = "TOKEN";
        let mut conn = GatewayContext::new((token, Intents::empty()));
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
        let token = "TOKEN";
        let mut conn = GatewayContext::new((token, Intents::empty()));
        assert_eq!(State::Closed, *conn.state());

        conn.recv(GatewayEvent::Heartbeat(1));

        assert_eq!(
            Some(GatewayCommand::Heartbeat(Heartbeat::new(0))),
            conn.send()
        );
    }
}
