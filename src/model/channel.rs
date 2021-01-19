use crate::model::{
    id::{ApplicationId, ChannelId, GuildId, MessageId, UserId},
    PermissonOverwrite, User,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the most general channel type within Discord
///
/// https://discord.com/developers/docs/resources/channel#channel-object
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
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum ChannelType {
    GuildText = 0,
    DM = 1,
    GuildVoice = 2,
    GroupDM = 3,
    GuildCategory = 4,
    GuildNews = 5,
    GuildStore = 6,
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
            6 => ChannelType::GuildStore,
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
