//! Gateway Commands

use crate::model::{
    gateway::{
        intents::{self, Intents},
        Opcode,
    },
    id::{ChannelId, GuildId, UserId},
    Activity,
};
use serde::{ser::SerializeStruct, Serialize};

/// Commands used to make requests to the gateway
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#commands-and-events-gateway-commands)
#[derive(Debug, PartialEq, Eq)]
pub enum GatewayCommand {
    /// triggers the initial handshake with the gateway
    Identify(Identify),

    /// resume a dropped gateway connection
    Resume(Resume),

    /// maintains an active gateway connection
    Heartbeat(u64),

    /// request members for a guild
    RequestGuildMembers(RequestGuildMembers),

    /// joins, moves, or disconnects the client from a voice channel
    UpdateVoiceState(UpdateVoiceState),

    /// update presence
    UpdateStatus(UpdateStatus),
}

/// Triggers the initial handshake with the gateway
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Identify {
    /// authentication token
    pub(crate) token: String,

    /// connection properties
    pub(crate) properties: ConnectionProperties,

    // /// whether this connection supports compression of packets (TODO: implement compression)
    // compress: Option<bool>,
    /// value between 50 and 250, total number of members where the gateway will stop sending
    /// offline members in the guild member list
    pub(crate) large_threshold: Option<i32>,

    /// guild shard and total shards
    pub(crate) shard: (i32, i32),

    /// initial presence information
    pub(crate) presence: Option<UpdateStatus>,

    /// gateway intens to recieve
    pub(crate) intents: Intents,
}

impl Identify {
    /// Create a new Identify command
    pub fn builder(token: &str) -> IdentifyBuilder {
        IdentifyBuilder::new(token.to_string())
    }
}

/// Builder for [`Identify`][Identify]
#[derive(Debug)]
pub struct IdentifyBuilder {
    token: String,
    properties: ConnectionProperties,
    // compress: Option<bool>,
    large_threshold: Option<i32>,
    shard: (i32, i32),
    presence: Option<UpdateStatus>,
    intents: Intents,
}

impl IdentifyBuilder {
    fn new(token: String) -> Self {
        IdentifyBuilder {
            token,
            properties: Default::default(),
            large_threshold: None,
            shard: (0, 1),
            presence: None,
            intents: intents::ALL,
        }
    }

    /// Set connection properties
    pub fn properties(mut self, properties: ConnectionProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Set large threshold
    pub fn large_threshold(mut self, large_threshold: i32) -> Self {
        self.large_threshold = Some(large_threshold);
        self
    }

    /// Set shard
    pub fn shard(mut self, shard: (i32, i32)) -> Self {
        self.shard = shard;
        self
    }

    /// Set presence
    pub fn presence(mut self, presence: UpdateStatus) -> Self {
        self.presence = Some(presence);
        self
    }

    /// Set presence
    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents = intents;
        self
    }

    /// Consume builder and create [`Identify`][Identify]
    pub fn build(self) -> Identify {
        Identify {
            token: self.token,
            properties: self.properties,
            large_threshold: self.large_threshold,
            shard: self.shard,
            presence: self.presence,
            intents: self.intents,
        }
    }
}

/// [Reference](https://discord.com/developers/docs/topics/gateway#identify-identify-connection-properties)
#[derive(Debug, Serialize, PartialEq, Eq, Default)]
pub struct ConnectionProperties {
    /// your operating system
    #[serde(rename = "$os")]
    pub os: String,

    /// your library name
    #[serde(rename = "$browser")]
    pub browser: String,

    /// your library name
    #[serde(rename = "$device")]
    pub device: String,
}

/// Sent by the client to indicate a presence or status update
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#update-status)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdateStatus {
    /// unix time (in milliseconds) of when the client went idle, or `None` if the client is not
    /// idle
    pub since: Option<i32>,

    /// `None`, or the user's activities
    #[serde(default)]
    pub activities: Option<Vec<Activity>>,

    /// the user's new status
    pub status: String,

    /// wather or not the client is afk
    pub afk: bool,
}

/// Used to replay missed events when a disconnected client resumes
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#resume)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Resume {
    /// session token
    pub token: String,
    /// session id
    pub session_id: String,
    /// last sequence number received
    pub seq: u64,
}

impl Resume {
    /// Create a new resume command
    pub fn new(token: String, session_id: String, seq: u64) -> Self {
        Resume {
            token,
            session_id,
            seq,
        }
    }
}

/// Used to request all members for a guild or a list of guilds.
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#request-guild-members)
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RequestGuildMembers {
    /// id of the guild to get members for
    pub guild_id: GuildId,

    /// string that username starts with, or an empty string to return all members
    pub query: Option<String>,

    /// maximum number of members to send matching the `query`; a limit of `0` can be used with an
    /// empty string `query` to return all members
    pub limit: i32,

    /// used to specify if we want the presences of the matched members
    pub presences: Option<bool>,

    /// used to specify which users you wish to fetch
    pub user_ids: Vec<UserId>,

    /// nonce to identify the `GuildMembersChunk` response
    pub nonce: Option<String>,
}

/// Sent when a client wants to join, move, or disconnect from a voice channel
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct UpdateVoiceState {
    /// id of the guild
    pub guild_id: GuildId,
    /// id of the voice channel the client wants to join (`None` if disconnecting)
    pub channel_id: Option<ChannelId>,
    /// is the client muted
    pub self_mute: bool,
    /// is the client deafend
    pub self_deaf: bool,
}

impl Serialize for GatewayCommand {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GatewayCommand", 2)?;
        state.serialize_field("op", &Opcode::from(self))?;

        match self {
            GatewayCommand::Identify(identify) => state.serialize_field("d", identify),
            GatewayCommand::Resume(resume) => state.serialize_field("d", resume),
            GatewayCommand::Heartbeat(seq) => state.serialize_field("d", seq),
            GatewayCommand::RequestGuildMembers(requets_guild_members) => {
                state.serialize_field("d", requets_guild_members)
            }
            GatewayCommand::UpdateVoiceState(update_voice_state) => {
                state.serialize_field("d", update_voice_state)
            }
            GatewayCommand::UpdateStatus(update_status) => {
                state.serialize_field("d", update_status)
            }
        }?;
        state.end()
    }
}
