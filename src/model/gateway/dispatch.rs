use crate::model::{
    id::{ChannelId, GuildId, MessageId, RoleId, UserId},
    Emoji, Presence, Role,
};
use crate::model::{GuildMember, PartialUser, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct Ready {
    #[serde(rename = "v")]
    pub version: u16,
    pub user: User,
    pub session_id: String,
    pub shard: Option<(u16, u16)>,
}

/// Sent when a user starts typing in a channel
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct TypingStart {
    /// id of the channel
    pub channel_id: ChannelId,
    /// id of the guild
    pub guild_id: Option<GuildId>,
    /// id of the user
    pub user_id: UserId,
    /// unix time (in seconds) of when the user started typing
    pub timestamp: u64,
    /// the member who started typing if this happened in a guild
    pub member: Option<GuildMember>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildMemberRemove {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildMemberUpdate {
    pub guild_id: GuildId,
    pub roles: Vec<RoleId>,
    pub user: User,
    pub nick: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub premium_since: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct InviteCreate {
    pub channel_id: ChannelId,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub guild_id: Option<GuildId>,
    pub intiver: Option<User>,
    pub max_age: i32,
    pub max_uses: i32,
    pub target_user: Option<PartialUser>,
    pub target_user_type: Option<i32>,
    pub temporary: bool,
    pub uses: i32,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct InviteDelete {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct ChannelPinsUpdate {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub last_pin_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildBanAdd {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildBanRemove {
    pub guild_id: GuildId,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildEmojisUpdate {
    pub guild_id: GuildId,
    pub emojis: Vec<Emoji>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildIntegrationsUpdate {
    pub guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildMembersChunk {
    pub guild_id: GuildId,
    pub members: Vec<GuildMember>,
    pub chunk_index: i32,
    pub chunk_count: i32,
    #[serde(default)]
    pub not_found: Vec<serde_json::Value>,
    #[serde(default)]
    pub presences: Vec<Presence>,
    pub nonce: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleCreate {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleUpdate {
    pub guild_id: GuildId,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleDelete {
    pub guild_id: GuildId,
    pub role_id: RoleId,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageDeleteBulk {
    pub ids: Vec<MessageId>,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionAdd {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
    pub member: Option<GuildMember>,
    pub emoji: Emoji,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemove {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
    pub emoji: Emoji,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemoveAll {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemoveEmoji {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub guild_id: Option<GuildId>,
    pub emoji: Emoji,
}

/// Sent when a guild's voice server is updated. This is sent when initially connecting to voice,
/// and when the current voice instance fails over to a new server.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct VoiceServerUpdate {
    /// voice connection token
    pub token: String,
    /// the guild this voice server update is for
    pub guild_id: GuildId,
    /// the voice server host
    pub endpoint: String,
}

/// Sent when a guild channel's webhook is created, updated, or deleted
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct WebhooksUpdate {
    /// id of the guild
    pub guild_id: GuildId,
    /// id of the channel
    pub channel_id: ChannelId,
}
