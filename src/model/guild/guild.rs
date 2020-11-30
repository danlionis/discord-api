use crate::model::{
    id::{ChannelId, GuildId},
    Presence,
};
use crate::model::{Channel, Emoji, GuildMember, Role, VoiceState};
use crate::Snowflake;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Guild {
    /// guild id
    pub id: GuildId,
    /// guild name
    pub name: String,
    /// icon hash
    pub icon: Option<String>,
    /// splash hash
    pub splash: Option<String>,
    /// discovery splash hash; only present for guild with the "DISCOVERABLE" feature
    pub discovery_splash: Option<String>,
    /// true if the user is the owner of the guild
    pub owner: Option<bool>,
    /// id of the owner
    pub owner_id: Snowflake,
    /// legacy total permissions for the uer in the guild (excludes overrides)
    pub permissions: Option<Snowflake>,
    /// voice region id fo the guild
    pub region: String,
    /// id of the afk channel
    pub afk_channel_id: Option<ChannelId>,
    /// afk timeout in seconds
    pub afk_timeout: u32,
    /// verification level required for the guild
    pub verification_level: u32,
    /// default message notification level
    pub default_message_notifications: u32,
    /// explicit content filter level
    pub explicit_content_filter: u32,
    /// roles in the guild
    pub roles: Vec<Role>,
    /// custom guild emojis
    pub emojis: Vec<Emoji>,
    /// enabled guild features
    pub features: Vec<GuildFeature>,
    /// required MFA level for the guild
    pub mfa_level: u32,
    /// application id of the guild creator if it is bot-created
    pub application_id: Option<Snowflake>,
    /// true if the server widget is enabled
    pub widget_enabled: Option<bool>,
    /// the channel id that the widget will generate an invite to, or `None` if set to no invite
    pub widget_channel_id: Option<Snowflake>,
    /// the id of the channel where guild notices such as welcome messages and boost events are posted
    pub system_channel_id: Option<Snowflake>,
    /// system channel flags
    pub system_channel_flags: u32,
    /// the id of the channel where guild with the `PUBLIC` feature can display rules and/or guidelines
    pub rules_channel_id: Option<Snowflake>,
    /// when this guild was joined at (only with the `GUILD_CREATE` event)
    pub joined_at: Option<DateTime<Utc>>,
    /// true if this guild is unavailable due to an outage (only with the `GUILD_CREATE` event
    pub unavailable: Option<bool>,
    /// total number of members in this guild (only with the `GUILD_CREATE` event
    pub member_count: Option<u32>,
    /// states of members currently in voice channels; lacks the guild_id key
    pub voice_states: Vec<VoiceState>,
    /// users in the guild
    pub members: Vec<GuildMember>,
    /// channels in the guild
    pub channels: Vec<Channel>,
    /// presences of the members in the guild, will only include non-offline members if the size is greater than `large_threshold`
    pub presences: Vec<Presence>,
    pub max_presences: Option<u32>,
    /// the maximum number of members for the guild
    pub max_members: u32,
    /// the vanity url code for the guild
    pub vanity_url_code: Option<String>,
    /// the description for the guild, if the guild is discoverable
    pub description: Option<String>,
    /// banner hash
    pub banner: Option<String>,
    /// premium tier (Server boost level)
    pub premium_tier: u8,
    /// the number of boosts this guild currently has
    pub premium_subscription_count: Option<u32>,
    /// the preffered locale of a guild with the `PUBLIC` feature; used in server discovery and notices from Discord
    pub preferred_locale: String,
    /// the id of the channel where admins and moderators of guild with the `PUBLIC` feature recieve notices from Discord
    pub public_updates_channel_id: Option<Snowflake>,
    /// the maximum amount of users in a video channel
    pub max_video_channel_users: Option<u32>,
    /// approximate number of members in this guild
    pub approximate_member_count: Option<u32>,
    /// approximate number of non-offline members in this guild
    pub approximate_presence_count: Option<u32>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct UnavailableGuild {
    id: GuildId,
    unavailable: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct GuildFeature(String);
