use crate::error::Error;
use crate::model::id::{ApplicationId, ChannelId, GuildId, MessageId, UserId};
use crate::model::PermissonOverwrite;
use crate::model::{Message, User};
use crate::wrapper::ModelWrapper;
use crate::Snowflake;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub enum ChannelType {
    GuildText = 0,
    DM = 1,
    GuildVoice = 2,
    GroupDM = 3,
    GuildCategory = 4,
    GuildNews = 5,
    GuildStore = 6,
}

pub enum Channel {
    Guild(GuildChannel),
    Private(PrivateChannel),
    //     Group(GroupChannel)
}

pub enum GuildChannel {
    Text(TextChannel),
    Voice(VoiceChannel),
    // GuildCategory(GuildCategoryChannel)
}

pub enum PrivateChannel {}

/// Represents a guild or DM channel within Discord
///
/// https://discord.com/developers/docs/resources/channel#channel-object
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct RawChannel {
    /// the id of this channel
    id: ChannelId,

    /// the type of channel
    r#type: u8,

    /// the id of the guild
    guild_id: Option<GuildId>,

    /// sorting position of the channel
    position: Option<i32>,

    /// explicit permission overwrites for members and roles
    permisson_overwrites: Option<Vec<PermissonOverwrite>>,

    /// the name of the channel (2-100 characters)
    name: Option<String>,

    /// the channel topic (0-1024 characters)
    topic: Option<String>,

    /// whether the channel is nsfw
    nsfw: Option<bool>,

    /// the id of the last message sent in this channel (may not point to an existing or valid message)
    last_message_id: Option<MessageId>,

    /// the bitrate (int bits) of the voice channel
    bitrate: Option<i32>,

    /// the user limit of the voice channel
    user_limit: Option<i32>,

    /// amout of seconds a user has to wait before sending another message (0-21600);
    /// bots, as well as users with the permission `manage_messages` or `manage_channel` are unaffected
    rate_limit_per_user: Option<i32>,

    /// the recipients of the DM
    recipients: Option<Vec<User>>,

    /// icon hash
    icon: Option<String>,

    /// the id of the DM creator
    owner_id: Option<UserId>,

    /// application id of the group DM creator if it is bot-created
    application_id: Option<ApplicationId>,

    /// id of the parent category for a channel (each parent category can contain up to 50 channels)
    parent_id: Option<String>,

    /// when the last pinnded message was pinned
    last_pin_timestamp: Option<DateTime<Utc>>,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub enum Channel {
//     GuildText(GuildTextChannel),
//     GuildVoice(GuildVoiceChannel),
// }

/// Represents a guild's text channel
#[derive(Debug, Deserialize, Serialize)]
pub struct TextChannel {
    pub id: ChannelId,
    pub guild_id: GuildId,
    pub name: String,
    pub position: u64,
    pub permission_overwrites: Vec<PermissonOverwrite>,
    pub rate_limit_per_user: u32,
    pub nsfw: bool,
    pub topic: Option<String>,
    pub last_message_id: Option<MessageId>,
    pub parent_id: Option<u64>,
    pub last_pin_timestamp: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoiceChannel {
    id: ChannelId,
    guild_id: GuildId,
    name: String,
    position: u64,
    nsfw: bool,
    pub permission_overwrites: Vec<PermissonOverwrite>,
    bitrate: Option<i32>,
    /// the user limit of the voice channel
    pub user_limit: Option<i32>,
    /// id of the parent category for a channel (each parent category can contain up to 50 channels)
    pub parent_id: Option<Snowflake>,
}

// impl From<RawChannel> for Channel {
//     fn from(raw_channel: RawChannel) -> Self {
//         match raw_channel.r#type {
//             0 => Channel::GuildText(GuildTextChannel {
//                 id: raw_channel.id.into(),
//                 guild_id: raw_channel.guild_id.unwrap(),
//                 position: raw_channel.position.unwrap(),
//                 name: raw_channel.name.unwrap(),
//                 last_message_id: raw_channel.last_message_id,
//                 last_pin_timestamp: raw_channel.last_pin_timestamp,
//                 parent_id: raw_channel.parent_id,
//                 nsfw: raw_channel.nsfw.unwrap(),
//                 rate_limit_per_user: raw_channel.rate_limit_per_user.unwrap(),
//                 topic: raw_channel.topic,
//                 // permission_overwrites: raw_channel.permisson_overwrites.unwrap(),
//             }),
//             _ => panic!("unknown channel type"),
//         }
//     }
// }

pub type TextChannelWrapper = ModelWrapper<TextChannel>;

impl TextChannelWrapper {
    pub async fn send_message(&self, content: &str) -> Result<Message, Error> {
        self.rest_client().create_message(self.id, content).await
    }
}
