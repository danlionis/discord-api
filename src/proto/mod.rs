//! Gateway Protocol
//!
//! A protocol implementation that takes received gateway events, handles connection state and generates
//! commands to be sent back to the gateway.
//!
//! The application is responsible for any I/O (e.g. sockets, async runtime)
//! as well as the event loop
//!
//! # Example
//! Minimal example that does not handle any heartbeating or I/O.
//! Create a context with all intents.
//! ```no_run
//! use discord::{
//!     proto::{GatewayContext, Config, GatewayCommand},
//!     model::gateway::{Intents, event::GatewayEvent}
//! };
//!
//! // functions to mock I/O functionality (should be handled by the application)
//! fn recv_from_socket() -> GatewayEvent {
//!     // omitted
//! #   unimplemented!()
//! };
//! fn send_to_socket(cmd: GatewayCommand) {
//!     // omitted
//! #   unimplemented!()
//! };
//! fn reconnect_socket() {
//!     // omitted
//! #   unimplemented!()
//! };
//!
//! // At this point the [GatewayContext] is still in the `Closed` state.
//! // As soon as the connection receives the correct `Hello` message from the gateway it will
//! // automatically initialize the connection.
//! let mut ctx = GatewayContext::new(Config::new("<token>", Intents::all()));
//!
//! loop {
//!     // reconnect the socket if necessary
//!     if ctx.should_reconnect() {
//!         reconnect_socket();
//!     }
//!
//!     // wait for a GatewayEvent to be received from the socket
//!     let event = recv_from_socket();
//!
//!     // pass the received event to the context for processing
//!     ctx.recv(&event);
//!
//!     // The context will generate messages that need to be sent to the server
//!     for cmd in ctx.send_iter() {
//!         send_to_socket(cmd);
//!     }
//! }
//! ```
//!
//! # Heartbeating
//! The application is responsible to maintain a heartbeat timer and queue heartbeat packets at the
//! correct time.
//!
//! Get the heartbeat interaval and queue a heartbeat:
//! ```
//! use discord::{proto::GatewayContext, model::gateway::Intents};
//! let mut ctx = GatewayContext::new(("<token>", Intents::empty()));
//!
//! let heartbeat_interval = ctx.heartbeat_interval();
//!
//! ctx.queue_heartbeat();
//! ```
//!
//! [`recv()`]: GatewayContext::recv
//! [`recv_json()`]: GatewayContext::recv_json
//! [`send_iter()`]: GatewayContext::send_iter
//! [`send()`]: GatewayContext::send

use crate::error::CloseCode;
use serde::Serialize;
use std::collections::VecDeque;
use twilight_model::gateway::{
    event::{DispatchEvent, GatewayEvent},
    payload::outgoing::{
        identify::IdentifyInfo, Heartbeat, Identify, RequestGuildMembers, Resume, UpdatePresence,
        UpdateVoiceState,
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

const SEND_QUEUE_SIZE: usize = 1;

/// Discord gateway context
///
/// Context for a given discord gateway connection.
/// Handles incoming events and generates outgoig commands
///
/// # Example
/// see module docs
#[derive(Debug)]
#[allow(missing_docs)]
pub struct GatewayContext {
    pub config: Config,
    /// sequence number
    pub seq: u64,
    pub session_id: String,
    pub heartbeat_interval: u64,
    pub send_queue: VecDeque<GatewayCommand>,
    pub state: State,
    pub socket_closed: bool,
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

    /// Returns the heartbeat interval.
    pub fn heartbeat_interval(&self) -> u64 {
        self.heartbeat_interval
    }

    /// Create a new GatewayContext to the discord gateway
    pub fn new<C>(config: C) -> Self
    where
        C: Into<Config>,
    {
        let config: Config = config.into();
        assert!(!config.token.is_empty(), "token cannot be empty");

        GatewayContext {
            config,
            seq: 0,
            heartbeat_interval: 0,
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
    pub fn recv(&mut self, event: &GatewayEvent) {
        log::trace!("gateway event= {:?}", event);

        // we've received an event so the socket can't be closed
        self.socket_closed = false;

        match event {
            // an invalid session can potentially be resumed
            GatewayEvent::InvalidateSession(resumable) => {
                self.state = if *resumable {
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

                self.heartbeat_interval = *heartbeat_interval;

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
                                large_threshold: self.config.large_threshold,
                                presence: self.config.presence.clone(),
                                properties: self.config.identify_properties.clone(),
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

                self.seq = *seq;
                // self.recv_queue.push_back((*event).into "
            }
        }
    }

    /// Processes a discord event received from the gateway.
    ///
    /// Takes an JSON string as input and returns the deserialized [`GatewayEvent`].
    #[cfg(feature = "json")]
    pub fn recv_json(&mut self, input: &str) -> Result<GatewayEvent, serde_json::Error> {
        use serde::de::DeserializeSeed;
        use serde_json::Deserializer;
        use twilight_model::gateway::event::GatewayEventDeserializer;

        let deserializer = GatewayEventDeserializer::from_json(input).unwrap();
        let mut json_deserializer = Deserializer::from_str(input);
        let event = deserializer.deserialize(&mut json_deserializer).unwrap();
        self.recv(&event);
        Ok(event)
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
            .map(|cmd| serde_json::to_string(&cmd).expect("command is always serializable"))
    }

    /// Creates a discord command to be sent to the gateway.
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
        self.send()
            .map(|cmd| serde_json::to_string(&cmd).expect("command is always serializable"))
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
    pub fn closed(&self) -> bool {
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
        oauth::{ApplicationFlags, PartialApplication},
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
    #[should_panic]
    fn empty_token() {
        let _ = GatewayContext::new(("", Intents::empty()));
    }

    #[test]
    fn connect() {
        let token = "TOKEN";
        let mut conn = GatewayContext::new((token, Intents::empty()));
        assert_eq!(State::Closed, *conn.state());

        let hello = GatewayEvent::Hello(10);
        conn.recv(&hello);
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
        conn.recv(&ready);
        assert_eq!(State::Ready, *conn.state());
    }

    #[test]
    fn resume() {
        let token = "TOKEN";
        let mut conn = GatewayContext::new((token, Intents::empty()));
        let _session_id = "session_id".to_string();
        assert_eq!(State::Closed, *conn.state());

        let hello = GatewayEvent::Hello(10);
        conn.recv(&hello);
        assert_eq!(State::Identify, *conn.state());
        assert_eq!(10, conn.heartbeat_interval());

        let _identify = conn.send().unwrap();

        let ready = create_default_ready();
        conn.recv(&ready);
        assert_eq!(State::Ready, *conn.state());

        // simulate socket reconnect
        let hello = GatewayEvent::Hello(15);
        conn.recv(&hello);
        assert_eq!(State::Replaying, *conn.state());
        assert_eq!(15, conn.heartbeat_interval());

        conn.recv(&GatewayEvent::Dispatch(
            conn.seq,
            Box::new(DispatchEvent::Resumed),
        ));
        assert_eq!(State::Ready, *conn.state());
    }

    #[test]
    fn invalid_session() {
        let token = "TOKEN";
        let mut conn = GatewayContext::new((token, Intents::empty()));
        assert_eq!(State::Closed, *conn.state());

        conn.recv(&GatewayEvent::InvalidateSession(true));
        assert_eq!(State::Resume, *conn.state());
        assert!(conn.should_reconnect());

        conn.recv(&GatewayEvent::InvalidateSession(false));
        assert_eq!(State::Reconnect, *conn.state());
        assert!(conn.should_reconnect());

        conn.recv(&GatewayEvent::Reconnect);
        assert_eq!(State::Resume, *conn.state());
        assert!(conn.should_reconnect());
    }

    #[test]
    fn heartbeat_request() {
        let token = "TOKEN";
        let mut conn = GatewayContext::new((token, Intents::empty()));
        assert_eq!(State::Closed, *conn.state());

        conn.recv(&GatewayEvent::Heartbeat(1));

        assert_eq!(
            Some(GatewayCommand::Heartbeat(Heartbeat::new(0))),
            conn.send()
        );
    }
}
