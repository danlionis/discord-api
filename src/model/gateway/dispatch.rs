//! Various Gateway Dispatch Events

use crate::model::{
    id::{ChannelId, GuildId, MessageId, RoleId, UserId},
    Emoji, Presence, Role,
};
use crate::model::{GuildMember, PartialUser, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Initial state information
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#ready)
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct Ready {
    /// gateway version
    #[serde(rename = "v")]
    pub version: u16,
    /// information about the current user
    pub user: User,
    /// SessionID (used for resuming connections)
    pub session_id: String,
    /// The shard information associated with this session
    pub shard: Option<(u16, u16)>,
}

/// Sent when a user starts typing in a channel
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#typing-start)
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

/// Sent when a user is removed from a guild (leave/ban/kick)
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-member-remove)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildMemberRemove {
    /// id of the guild
    pub guild_id: GuildId,
    /// the user that was removed
    pub user: User,
}

/// Sent when a guild member is updated
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-member-update)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildMemberUpdate {
    /// id of the guild
    pub guild_id: GuildId,
    /// array of RoleIds
    pub roles: Vec<RoleId>,
    /// the user
    pub user: User,
    /// nickname of the user in the guild
    pub nick: Option<String>,
    /// when the user joined the guild
    pub joined_at: DateTime<Utc>,
    /// when the user started boosting the guild
    pub premium_since: Option<DateTime<Utc>>,
}

/// Sent when a new invite to a channel is created
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#invite-create)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct InviteCreate {
    /// the channel the invite is for
    pub channel_id: ChannelId,
    /// the unique invite code
    pub code: String,
    /// the time at which the invite was created
    pub created_at: DateTime<Utc>,
    /// the guild of the invite
    pub guild_id: Option<GuildId>,
    /// the user that created the invite
    pub intiver: Option<User>,
    /// how long the invite is valid for (in seconds)
    pub max_age: i32,
    /// the maximun number of times the invite can be used
    pub max_uses: i32,
    /// the target user for this invite
    pub target_user: Option<PartialUser>,
    /// the type of user target for this invite
    pub target_user_type: Option<i32>,
    /// whether or not the invite is temporary
    pub temporary: bool,
    /// how many times the invite has been used
    pub uses: i32,
}

/// Sent when an invite was deleted
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#invite-delete)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct InviteDelete {
    /// the channel of the invite
    pub channel_id: ChannelId,
    /// the guild of the invite
    pub guild_id: Option<GuildId>,
    /// the unique invite code
    pub code: String,
}

/// Sent when a message is pinned or unpinned in the text channel. This is not sent when a pinned
/// message is deleted
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#channel-pins-update)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct ChannelPinsUpdate {
    /// the id of the guild
    pub guild_id: Option<GuildId>,
    /// the id of the channel
    pub channel_id: ChannelId,
    /// the time at which the most recent pinned message was pinned
    pub last_pin_timestamp: Option<DateTime<Utc>>,
}

/// Sent when a user is banned from a guild
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-ban-add)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildBanAdd {
    /// id of the guild
    pub guild_id: GuildId,
    /// the banned user
    pub user: User,
}

/// Sent when a user is unbanned from a guild
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-ban-remove)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildBanRemove {
    /// id of the guild
    pub guild_id: GuildId,
    /// the unbanned user
    pub user: User,
}

/// Sent when a guild's emojis have been updated
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-emojis-update)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildEmojisUpdate {
    /// id of the guild
    pub guild_id: GuildId,
    /// array of Emojis
    pub emojis: Vec<Emoji>,
}

/// Sent when a guild integration is updated
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-integrations-update)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct GuildIntegrationsUpdate {
    /// id of the guild whose integrations were updated
    pub guild_id: GuildId,
}

/// Sent in response to `GuildRequestMembers`
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-members-chunk)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildMembersChunk {
    /// the id of the guild
    pub guild_id: GuildId,
    /// set of guild members
    pub members: Vec<GuildMember>,
    /// the chunk index in the expected chunks for this response
    pub chunk_index: i32,
    /// the total number of expected chunks for this response
    pub chunk_count: i32,
    // #[serde(default)]
    // /// if passing an invalid ID to REQUEST_GUILD_MEMBERS, it will be returned here
    // pub not_found: Vec<serde_json::Value>,
    /// if passing true to REQUEST_GUILD_MEMBERS, presences of the returned members will be here
    #[serde(default)]
    pub presences: Vec<Presence>,
    /// the nonce used in the GuildMembersRequest
    pub nonce: Option<String>,
}

/// Sent when a guild role is created
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-role-create)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleCreate {
    /// the id of the guild
    pub guild_id: GuildId,
    /// the role created
    pub role: Role,
}

/// Sent when a guild role is updated
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-role-update)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleUpdate {
    /// the id of the guild
    pub guild_id: GuildId,
    /// the role updated
    pub role: Role,
}

/// Sent when a guild role is deleted
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#guild-role-delete))
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GuildRoleDelete {
    /// the id of the guild
    pub guild_id: GuildId,
    /// the id of the deleted role
    pub role_id: RoleId,
}

/// Sent when multiple messages are deleted at once
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#message-delete-bulk)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageDeleteBulk {
    /// the ids of the messages
    pub ids: Vec<MessageId>,
    /// the id of the channel
    pub channel_id: ChannelId,
    /// the id of the guild
    pub guild_id: Option<GuildId>,
}

/// Sent when a user adds a reaction to a message
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#message-reaction-add)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionAdd {
    /// the if of the user
    pub user_id: UserId,
    /// the id of the channel
    pub channel_id: ChannelId,
    /// the id of the message
    pub message_id: MessageId,
    /// the id of the guild
    pub guild_id: Option<GuildId>,
    /// the member who reacted if this happened in a guild
    pub member: Option<GuildMember>,
    /// the emoji used to react
    pub emoji: Emoji,
}

/// Sent when a user removes a reaction from a message
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#message-reaction-remove)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemove {
    /// the id of the user
    pub user_id: UserId,
    /// the id of the channel
    pub channel_id: ChannelId,
    /// the id of the message
    pub message_id: MessageId,
    /// the id of the guild
    pub guild_id: Option<GuildId>,
    /// the emoji used to react
    pub emoji: Emoji,
}

/// Sent when a user explicitly removes all reactions from a message
///
/// [Reference](https://discord.com/developers/docs/topics/gateway#message-reaction-remove-all)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemoveAll {
    /// the id of the channel
    pub channel_id: ChannelId,
    /// the id of the message
    pub message_id: MessageId,
    /// the id of the guild
    pub guild_id: Option<GuildId>,
}

/// Sent when a bot removes all instances of a given emoji from the reactions of a message
/// [Reference](https://discord.com/developers/docs/topics/gateway#message-reaction-remove-emoji)
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct MessageReactionRemoveEmoji {
    /// the id of the channel
    pub channel_id: ChannelId,
    /// the id of the guild
    pub message_id: MessageId,
    /// the id of the message
    pub guild_id: Option<GuildId>,
    /// the emoji that was removed
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
