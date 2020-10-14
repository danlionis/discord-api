use crate::model::id::{ChannelId, GuildId, UserId};
use crate::model::GuildMember;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct VoiceState {
    /// the guild id this voice state is for
    guild_id: Option<GuildId>,
    /// the channel id this user is connected to
    channel_id: Option<ChannelId>,
    /// the user id this voice state is for
    user_id: UserId,
    /// the guild member this voice state is for
    member: Option<GuildMember>,
    /// the session id for this voice state
    session_id: String,
    /// whether this user is deafened by the server
    #[serde(rename = "deaf")]
    server_deaf: bool,
    /// whether this user is muted by the server
    #[serde(rename = "mute")]
    server_mute: bool,
    /// whether this user is locally deafened
    self_mute: bool,
    /// whether this user is locally muted
    self_deaf: bool,
    /// whether this user is streaming using "Go Live"
    #[serde(default)]
    self_stream: bool,
    /// whether this user's camera is enabled
    self_video: bool,
    /// whether this user is muted by the current user
    suppress: bool,
}
