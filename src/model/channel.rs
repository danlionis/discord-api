//! Channel types
use crate::model::{
    id::{ApplicationId, ChannelId, GuildId, MessageId, UserId},
    PermissonOverwrite, ThreadMetadata, User,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, ops::Deref};

use super::ThreadMember;

/// Represents the most general channel type within Discord
///
/// [Reference](https://discord.com/developers/docs/resources/channel#channel-object)
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct Channel {
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

    /// voice region id for the voice channel, automatic when set to null
    pub rtc_region: Option<String>,

    /// the camera video quality mode of the voice channel, 1 when not present
    pub video_quality_mode: Option<i32>,

    /// an approximate count of messages in a thread, stops counting at 50
    pub message_count: Option<i32>,

    /// an approximate count of users in a thread, stops counting at 50
    pub member_count: Option<i32>,

    /// thread-specific fields not needed by other channels
    pub thread_metadata: Option<ThreadMetadata>,

    /// thread member object for the current user, if they have joined the thread, only included on certain API endpoints
    pub member: Option<ThreadMember>,

    /// default duration that the clients (not the API) will use for newly created threads, in minutes, to automatically archive the thread after recent activity, can be set to: 60, 1440, 4320, 10080
    pub default_auto_archive_duration: Option<i32>,

    /// computed permissions for the invoking user in the channel, including overwrites, only included when part of the `resolved` data received on a slash command interaction
    pub permissions: Option<String>,

    /// channel flags combined as a bitfield
    pub flags: Option<i32>,
}

impl AsRef<ChannelId> for Channel {
    fn as_ref(&self) -> &ChannelId {
        &self.id
    }
}

/// Error when trying to cast a Channel
#[derive(Debug)]
pub struct InvalidChannelType;

/// Channel type
///
/// <https://discord.com/developers/docs/resources/channel#channel-object-channel-types>
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ChannelType {
    GuildText = 0,
    DM = 1,
    GuildVoice = 2,
    GroupDM = 3,
    GuildCategory = 4,
    GuildNews = 5,
    #[deprecated]
    GuildStore = 6,
    GuildNewsThread = 10,
    GuildPublicThread = 11,
    GuildPrivateThread = 12,
    GuildStageVoice = 13,
    GuildDirectory = 14,
    GuildForum = 15,
}

impl std::convert::From<u8> for ChannelType {
    fn from(v: u8) -> Self {
        match v {
            0 => ChannelType::GuildText,
            1 => ChannelType::DM,
            2 => ChannelType::GuildVoice,
            3 => ChannelType::GroupDM,
            4 => ChannelType::GuildCategory,
            5 => ChannelType::GuildNews,
            10 => ChannelType::GuildNewsThread,
            11 => ChannelType::GuildPublicThread,
            12 => ChannelType::GuildPrivateThread,
            13 => ChannelType::GuildStageVoice,
            14 => ChannelType::GuildDirectory,
            15 => ChannelType::GuildForum,
            _ => panic!("unknown channel type"),
        }
    }
}

impl<'de> Deserialize<'de> for ChannelType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = u8::deserialize(deserializer)?;

        Ok(ChannelType::from(v))
    }
}

impl Serialize for ChannelType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

macro_rules! impl_concrete_channel {
    ($name:ident, $t:expr) => {
        #[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
        #[allow(missing_docs)]
        pub struct $name(Channel);

        impl TryFrom<Channel> for $name {
            type Error = InvalidChannelType;

            fn try_from(c: Channel) -> Result<Self, Self::Error> {
                if c.kind != $t {
                    return Err(InvalidChannelType);
                }
                Ok($name(c))
            }
        }

        impl Deref for $name {
            type Target = Channel;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

impl_concrete_channel!(GuildTextChannel, ChannelType::GuildText);
impl_concrete_channel!(GuildVoiceChannel, ChannelType::GuildVoice);
impl_concrete_channel!(GuildCategoryChannel, ChannelType::GuildCategory);
impl_concrete_channel!(GuildNewsChannel, ChannelType::GuildNews);
impl_concrete_channel!(DMChannel, ChannelType::DM);
impl_concrete_channel!(GroupDMChannel, ChannelType::GroupDM);
