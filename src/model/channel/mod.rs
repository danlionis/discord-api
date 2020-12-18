use crate::error::Error;
use crate::model::id::{ChannelId, GuildId, MessageId, UserId};
use crate::model::PermissonOverwrite;
use crate::model::{Message, User};
use crate::wrapper::ModelWrapper;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

mod raw;
use raw::RawChannel;

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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Channel {
    Guild(GuildChannel),
    Private(PrivateChannel),
    // Group(GroupChannel)
}

impl Channel {
    pub fn name(&self) -> &str {
        match self {
            Channel::Guild(c) => c.name(),
            Channel::Private(_c) => "private channel name unimplemented",
        }
    }
}

pub enum TextChannel {
    Guild(GuildTextChannel),
    Private(PrivateChannel),
}

impl TextChannel {
    pub fn id(&self) -> &ChannelId {
        match self {
            TextChannel::Guild(c) => &c.id,
            TextChannel::Private(c) => &c.id,
        }
    }
}

impl std::convert::From<RawChannel> for Channel {
    fn from(raw: RawChannel) -> Self {
        match raw.kind {
            ChannelType::GuildText
            | ChannelType::GuildVoice
            | ChannelType::GuildNews
            | ChannelType::GuildStore
            | ChannelType::GuildCategory => Channel::Guild(GuildChannel::from(raw)),
            ChannelType::DM => unimplemented!(),
            ChannelType::GroupDM => unimplemented!(),
        }
    }
}

impl std::convert::From<RawChannel> for GuildChannel {
    fn from(raw: RawChannel) -> Self {
        match raw.kind {
            ChannelType::GuildText => GuildChannel::Text(GuildTextChannel::try_from(raw).unwrap()),
            ChannelType::GuildVoice => GuildChannel::Voice(VoiceChannel::try_from(raw).unwrap()),
            ChannelType::GuildCategory => {
                GuildChannel::GuildCategory(CategoryChannel::try_from(raw).unwrap())
            }
            ChannelType::GuildNews => unimplemented!(),
            ChannelType::GuildStore => unimplemented!(),
            _ => unreachable!(),
        }
    }
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let raw = RawChannel::from(self.clone());
        raw.serialize(serializer)
    }
}

#[derive(Debug)]
pub struct InvalidChannelTypeError;

impl TryFrom<RawChannel> for GuildTextChannel {
    type Error = InvalidChannelTypeError;

    fn try_from(raw: RawChannel) -> Result<Self, Self::Error> {
        if raw.kind != ChannelType::GuildText {
            return Err(InvalidChannelTypeError);
        }

        Ok(GuildTextChannel {
            id: raw.id,
            guild_id: raw.guild_id,
            name: raw.name.unwrap(),
            position: raw.position.unwrap(),
            permission_overwrites: raw.permission_overwrites.unwrap(),
            rate_limit_per_user: raw.rate_limit_per_user.unwrap(),
            nsfw: raw.nsfw.unwrap_or_default(),
            topic: raw.topic,
            last_message_id: raw.last_message_id,
            last_pin_timestamp: raw.last_pin_timestamp,
            parent_id: raw.parent_id,
        })
    }
}

impl TryFrom<RawChannel> for VoiceChannel {
    type Error = InvalidChannelTypeError;

    fn try_from(raw: RawChannel) -> Result<Self, Self::Error> {
        if raw.kind != ChannelType::GuildVoice {
            return Err(InvalidChannelTypeError);
        }

        Ok(VoiceChannel {
            id: raw.id,
            guild_id: raw.guild_id,
            name: raw.name.unwrap(),
            position: raw.position.unwrap(),
            permission_overwrites: raw.permission_overwrites.unwrap(),
            nsfw: raw.nsfw.unwrap_or_default(),
            bitrate: raw.bitrate.unwrap(),
            parent_id: raw.parent_id,
            user_limit: raw.user_limit,
        })
    }
}

impl TryFrom<RawChannel> for CategoryChannel {
    type Error = InvalidChannelTypeError;

    fn try_from(raw: RawChannel) -> Result<Self, Self::Error> {
        if raw.kind != ChannelType::GuildCategory {
            return Err(InvalidChannelTypeError);
        }

        Ok(CategoryChannel {
            id: raw.id,
            guild_id: raw.guild_id,
            name: raw.name.unwrap(),
            position: raw.position.unwrap(),
            permission_overwrites: raw.permission_overwrites.unwrap(),
            nsfw: raw.nsfw.unwrap_or_default(),
            parent_id: raw.parent_id,
        })
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawChannel::deserialize(deserializer)?;

        Ok(Channel::from(raw))
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum GuildChannel {
    Text(GuildTextChannel),
    Voice(VoiceChannel),
    GuildCategory(CategoryChannel),
}

impl GuildChannel {
    pub fn id(&self) -> &ChannelId {
        match self {
            GuildChannel::Text(c) => &c.id,
            GuildChannel::Voice(c) => &c.id,
            GuildChannel::GuildCategory(c) => &c.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            GuildChannel::Text(c) => &c.name,
            GuildChannel::Voice(c) => &c.name,
            GuildChannel::GuildCategory(c) => &c.name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct PrivateChannel {
    id: ChannelId,
    last_message_id: Option<MessageId>,
    recipients: Option<Vec<User>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct GroupChannel {
    id: ChannelId,
    name: String,
    last_message_id: Option<MessageId>,
    recipients: Option<Vec<User>>,
    icon: Option<String>,
    owner_id: Option<UserId>,
}

/// Represents a guild's text channel
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct GuildTextChannel {
    pub id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub name: String,
    pub position: i32,
    pub permission_overwrites: Vec<PermissonOverwrite>,
    pub rate_limit_per_user: i32,
    pub nsfw: bool,
    pub topic: Option<String>,
    pub last_message_id: Option<MessageId>,
    pub parent_id: Option<ChannelId>,
    pub last_pin_timestamp: Option<DateTime<Utc>>,
}

/// Represents a guild's category
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct CategoryChannel {
    pub id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub name: String,
    pub position: i32,
    pub permission_overwrites: Vec<PermissonOverwrite>,
    pub nsfw: bool,
    pub parent_id: Option<ChannelId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct VoiceChannel {
    pub id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub name: String,
    pub position: i32,
    pub nsfw: bool,
    pub permission_overwrites: Vec<PermissonOverwrite>,
    pub bitrate: i32,
    /// the user limit of the voice channel
    pub user_limit: Option<i32>,
    /// id of the parent category for a channel (each parent category can contain up to 50 channels)
    pub parent_id: Option<ChannelId>,
}

pub type TextChannelWrapper = ModelWrapper<TextChannel>;

impl TextChannelWrapper {
    pub async fn send_message(&self, content: &str) -> Result<Message, Error> {
        self.rest_client()
            .create_message(*self.id(), content, None)
            .await
    }
}

#[cfg(test)]
mod tests {}
