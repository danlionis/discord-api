use crate::model::{
    id::{ChannelId, GuildId, MessageId, RoleId, UserId},
    Emoji, Presence, Role,
};
use crate::model::{GuildMember, PartialUser, User};
use crate::Snowflake;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct Ready {
    #[serde(rename = "v")]
    pub version: u16,
    pub user: User,
    pub session_id: String,
    pub shard: Option<(Snowflake, u16)>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct TypingStart {
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
    user_id: UserId,
    timestamp: u64,
    member: Option<GuildMember>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildMemberRemove {
    guild_id: GuildId,
    user: User,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildMemberUpdate {
    guild_id: GuildId,
    roles: Vec<RoleId>,
    user: User,
    nick: Option<String>,
    joined_at: DateTime<Utc>,
    premium_since: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct InviteCreate {
    channel_id: ChannelId,
    code: String,
    created_at: u64,
    guild_id: Option<GuildId>,
    intiver: Option<User>,
    max_age: i32,
    max_uses: i32,
    target_user: Option<PartialUser>,
    target_user_type: Option<i32>,
    temporary: bool,
    uses: i32,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct InviteDelete {
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
    code: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct ChannelPinsUpdate {
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
    last_pin_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildBanAdd {
    guild_id: GuildId,
    user: User,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildBanRemove {
    guild_id: GuildId,
    user: User,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildEmojisUpdate {
    guild_id: GuildId,
    emojis: Vec<Emoji>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildIntegrationsUpdate {
    guild_id: GuildId,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildMembersChunk {
    guild_id: GuildId,
    members: Vec<GuildMember>,
    chunk_index: i32,
    chunk_count: i32,
    #[serde(default)]
    not_found: Vec<serde_json::Value>,
    #[serde(default)]
    presences: Vec<Presence>,
    nonce: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleCreate {
    guild_id: GuildId,
    role: Role,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleUpdate {
    guild_id: GuildId,
    role: Role,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleDelete {
    guild_id: GuildId,
    role_id: RoleId,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageDeleteBulk {
    ids: Vec<MessageId>,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionAdd {
    user_id: UserId,
    channel_id: ChannelId,
    message_id: MessageId,
    guild_id: Option<GuildId>,
    member: Option<GuildMember>,
    emoji: Emoji,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemove {
    user_id: UserId,
    channel_id: ChannelId,
    message_id: MessageId,
    guild_id: Option<GuildId>,
    emoji: Emoji,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemoveAll {
    channel_id: ChannelId,
    message_id: MessageId,
    guild_id: Option<GuildId>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemoveEmoji {
    channel_id: ChannelId,
    message_id: MessageId,
    guild_id: Option<GuildId>,
    emoji: Emoji,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct VoiceServerUpdate {
    token: String,
    guild_id: GuildId,
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct WebhooksUpdate {
    guild_id: GuildId,
    channel_id: ChannelId,
}
