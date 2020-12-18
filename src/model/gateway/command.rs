use crate::model::gateway::intents::{self, Intents};
use crate::model::gateway::Opcode;
use crate::model::id::{ChannelId, GuildId, UserId};
use crate::model::Activity;
use serde::ser::SerializeStruct;
use serde::Serialize;

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

#[derive(Serialize)]
pub struct Identify {
    /// authentication token
    pub token: String,

    /// connection properties
    pub properties: ConnectionProperties,

    // /// whether this connection supports compression of packets (TODO: implement compression)
    // compress: Option<bool>,
    //
    /// value between 50 and 250, total number of members where the gateway will stop sending
    /// offline members in the guild member list
    pub large_threshold: Option<i32>,

    /// guild sharding
    pub shard: (i32, i32),

    /// initial presence information
    pub presence: Option<UpdateStatus>,

    /// gateway intens to recieve
    pub intents: Intents,
}

impl Identify {
    pub(crate) fn new(token: &str) -> Self {
        let properties = ConnectionProperties {
            os: "linux".to_owned(),
            device: "donbot".to_owned(),
            browser: "donbot".to_owned(),
        };

        Self {
            token: token.to_owned(),
            properties,
            large_threshold: None,
            shard: (0, 1),
            presence: None,
            intents: intents::ALL,
        }
    }
}

#[derive(Serialize)]
pub struct ConnectionProperties {
    #[serde(rename = "$os")]
    pub os: String,
    #[serde(rename = "$browser")]
    pub browser: String,
    #[serde(rename = "$device")]
    pub device: String,
}

#[derive(Serialize)]
pub struct UpdateStatus {
    /// unix time (in milliseconds) of when the client went idle, or `None` if the client is not
    /// idle
    pub since: Option<i32>,

    /// `None`, or the user's activities
    #[serde(default)]
    pub activities: Vec<Activity>,

    /// the user's new status
    pub status: String,

    /// wather or not the client is afk
    pub afk: bool,
}

#[derive(Serialize)]
pub struct Resume {
    pub token: String,
    pub session_id: String,
    pub seq: u64,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct UpdateVoiceState {
    pub guild_id: GuildId,
    pub channel_id: Option<ChannelId>,
    pub self_mute: bool,
    pub self_deaf: bool,
}

impl GatewayCommand {
    pub fn opcode(&self) -> Opcode {
        Opcode::from(self)
    }
}

impl Serialize for GatewayCommand {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GatewayCommand", 2)?;
        state.serialize_field("op", &self.opcode())?;

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
