use crate::model::id::{ChannelId, GuildId, UserId};
use crate::model::GuildMember;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Eq, PartialEq, Debug, Deserialize, Serialize)]
/// VoiceState
pub struct VoiceState {
    /// the guild id this voice state is for
    pub guild_id: Option<GuildId>,
    /// the channel id this user is connected to
    pub channel_id: Option<ChannelId>,
    /// the user id this voice state is for
    pub user_id: UserId,
    /// the guild member this voice state is for
    pub member: Option<GuildMember>,
    /// the session id for this voice state
    pub session_id: String,
    /// whether this user is deafened by the server
    #[serde(rename = "deaf")]
    pub server_deaf: bool,
    /// whether this user is muted by the server
    #[serde(rename = "mute")]
    pub server_mute: bool,
    /// whether this user is locally deafened
    pub self_mute: bool,
    /// whether this user is locally muted
    pub self_deaf: bool,
    /// whether this user is streaming using "Go Live"
    #[serde(default)]
    pub self_stream: bool,
    /// whether this user's camera is enabled
    pub self_video: bool,
    /// whether this user is muted by the current user
    pub suppress: bool,
}
