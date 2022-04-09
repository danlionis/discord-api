use std::{convert::TryFrom, fmt::Display, str::FromStr};

use http::{uri::InvalidUri, Uri};

use crate::model::id::{ChannelId, MessageId};

const DISCORD_API_PREFIX: &str = "https://discord.com/api/v9";

/// Enum containing all routes for the discord rest api
#[derive(Debug)]
pub enum Route {
    /// Channel messages
    ChannelMessages {
        /// ChannelId
        channel_id: ChannelId,
    },
    /// Gateway URL
    Gateway,
    /// Gateway URL with additional information for bots
    GatewayBot,
    /// Single Text message
    TextMessage {
        /// ChannelId
        channel_id: ChannelId,
        /// MessageId
        message_id: MessageId,
    },
}

impl Display for Route {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", DISCORD_API_PREFIX)?;
        match self {
            Route::ChannelMessages { channel_id } => {
                write!(fmt, "/channels/{}/messages", channel_id)
            }
            Route::TextMessage {
                channel_id,
                message_id,
            } => {
                write!(fmt, "/channels/{}/messages/{}", channel_id, message_id)
            }
            Route::Gateway => {
                write!(fmt, "/gateway")
            }
            Route::GatewayBot => {
                write!(fmt, "/gateway/bot")
            }
        }
    }
}

impl TryFrom<Route> for Uri {
    type Error = InvalidUri;
    fn try_from(route: Route) -> Result<Self, <Self as TryFrom<Route>>::Error> {
        Uri::from_str(route.to_string().as_str())
    }
}

// pub fn guilds() -> String {
//     api!("/users/@me/guilds").to_owned()
// }

// pub fn guild(id: u64) -> String {
//     api!("/guild/{}", id)
// }

// pub fn guild_channels(id: u64) -> String {
//     api!("/guilds/{}/channels", id)
// }

// pub fn guild_member(guild_id: u64, user_id: u64) -> String {
//     api!("/guilds/{}/members/{}", guild_id, user_id)
// }

// pub fn channel_messages(id: u64) -> String {
//     api!("/channels/{}/messages", id)
// }

// pub fn user_dm() -> String {
//     api!("/users/@me/channels").to_owned()
// }

// pub fn text_message(id: u64, message_id: u64) -> String {
//     api!("/channels/{}/messages/{}", id, message_id)
// }

// pub fn trigger_typing_indicator(channel_id: u64) -> String {
//     api!("/channels/{}/typing", channel_id)
// }

// pub fn gateway() -> String {
//     api!("/gateway/bot").to_owned()
// }
