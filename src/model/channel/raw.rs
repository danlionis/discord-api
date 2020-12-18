use super::ChannelType;
use crate::model::channel::{
    CategoryChannel, Channel, GuildChannel, GuildTextChannel, PrivateChannel, VoiceChannel,
};
use crate::model::id::{ApplicationId, ChannelId, GuildId, MessageId, UserId};
use crate::model::{PermissonOverwrite, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::convert::From;

/// Represents a guild or DM channel within Discord
///
/// https://discord.com/developers/docs/resources/channel#channel-object
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub(crate) struct RawChannel {
    /// the id of this channel
    pub id: ChannelId,

    /// the type of channel
    #[serde(rename = "type")]
    pub kind: ChannelType,

    /// the id of the guild
    pub guild_id: Option<GuildId>,

    /// sorting position of the channel
    pub position: Option<i32>,

    /// explicit permission overwrites for members and roles
    #[serde(default)]
    pub permission_overwrites: Option<Vec<PermissonOverwrite>>,

    /// the name of the channel (2-100 characters)
    pub name: Option<String>,

    /// the channel topic (0-1024 characters)
    pub topic: Option<String>,

    /// whether the channel is nsfw
    pub nsfw: Option<bool>,

    /// the id of the last message sent in this channel (may not point to an existing or valid message)
    pub last_message_id: Option<MessageId>,

    /// the bitrate (int bits) of the voice channel
    pub bitrate: Option<i32>,

    /// the user limit of the voice channel
    pub user_limit: Option<i32>,

    /// amount of seconds a user has to wait before sending another message (0-21600);
    /// bots, as well as users with the permission `manage_messages` or `manage_channel` are unaffected
    pub rate_limit_per_user: Option<i32>,

    /// the recipients of the DM
    pub recipients: Option<Vec<User>>,

    /// icon hash
    pub icon: Option<String>,

    /// the id of the DM creator
    pub owner_id: Option<UserId>,

    /// application id of the group DM creator if it is bot-created
    pub application_id: Option<ApplicationId>,

    /// id of the parent category for a channel (each parent category can: contain up to 50 channels)
    pub parent_id: Option<ChannelId>,

    /// when the last pinnded message was pinned
    pub last_pin_timestamp: Option<DateTime<Utc>>,
}

impl From<Channel> for RawChannel {
    fn from(c: Channel) -> Self {
        match c {
            Channel::Guild(guild_channel) => RawChannel::from(guild_channel),
            Channel::Private(private_channel) => RawChannel::from(private_channel),
        }
    }
}

impl From<GuildChannel> for RawChannel {
    fn from(c: GuildChannel) -> Self {
        match c {
            GuildChannel::Text(text_channel) => RawChannel::from(text_channel),
            GuildChannel::Voice(voice_channel) => RawChannel::from(voice_channel),
            GuildChannel::GuildCategory(category_channel) => RawChannel::from(category_channel),
        }
    }
}

impl From<GuildTextChannel> for RawChannel {
    fn from(c: GuildTextChannel) -> Self {
        RawChannel {
            id: c.id,
            kind: ChannelType::GuildText,
            guild_id: c.guild_id,
            position: Some(c.position),
            permission_overwrites: Some(c.permission_overwrites),
            name: Some(c.name),
            topic: c.topic,
            nsfw: Some(c.nsfw),
            last_message_id: c.last_message_id,
            bitrate: None,
            user_limit: None,
            rate_limit_per_user: Some(c.rate_limit_per_user),
            recipients: None,
            icon: None,
            owner_id: None,
            application_id: None,
            parent_id: c.parent_id,
            last_pin_timestamp: c.last_pin_timestamp,
        }
    }
}

impl From<VoiceChannel> for RawChannel {
    fn from(c: VoiceChannel) -> Self {
        RawChannel {
            id: c.id,
            kind: ChannelType::GuildVoice,
            guild_id: c.guild_id,
            position: Some(c.position),
            permission_overwrites: Some(c.permission_overwrites),
            name: Some(c.name),
            topic: None,
            nsfw: Some(c.nsfw),
            last_message_id: None,
            bitrate: Some(c.bitrate),
            user_limit: c.user_limit,
            rate_limit_per_user: None,
            recipients: None,
            icon: None,
            owner_id: None,
            application_id: None,
            parent_id: c.parent_id,
            last_pin_timestamp: None,
        }
    }
}

impl From<CategoryChannel> for RawChannel {
    fn from(c: CategoryChannel) -> Self {
        RawChannel {
            id: c.id,
            kind: ChannelType::GuildCategory,
            guild_id: c.guild_id,
            position: Some(c.position),
            permission_overwrites: Some(c.permission_overwrites),
            name: Some(c.name),
            topic: None,
            nsfw: Some(c.nsfw),
            last_message_id: None,
            bitrate: None,
            user_limit: None,
            rate_limit_per_user: None,
            recipients: None,
            icon: None,
            owner_id: None,
            application_id: None,
            parent_id: c.parent_id,
            last_pin_timestamp: None,
        }
    }
}

impl From<PrivateChannel> for RawChannel {
    fn from(c: PrivateChannel) -> Self {
        RawChannel {
            id: c.id,
            kind: ChannelType::DM,
            guild_id: None,
            position: None,
            permission_overwrites: None,
            name: None,
            topic: None,
            nsfw: None,
            last_message_id: c.last_message_id,
            bitrate: None,
            user_limit: None,
            rate_limit_per_user: None,
            recipients: c.recipients,
            icon: None,
            owner_id: None,
            application_id: None,
            parent_id: None,
            last_pin_timestamp: None,
        }
    }
}
