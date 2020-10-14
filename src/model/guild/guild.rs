use crate::model::id::{ChannelId, GuildId};
use crate::model::{Emoji, GuildMember, Presence, RawChannel, Role, VoiceState};
use crate::Snowflake;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct Guild {
    /// guild id
    id: GuildId,
    /// guild name
    name: String,
    /// icon hash
    icon: Option<String>,
    /// splash hash
    splash: Option<String>,
    /// discovery splash hash; only present for guild with the "DISCOVERABLE" feature
    discovery_splash: Option<String>,
    /// true if the user is the owner of the guild
    owner: Option<bool>,
    /// id of the owner
    owner_id: Snowflake,
    /// legacy total permissions for the uer in the guild (excludes overrides)
    permissions: Option<Snowflake>,
    /// voice region id fo the guild
    region: String,
    /// id of the afk channel
    afk_channel_id: Option<ChannelId>,
    /// afk timeout in seconds
    afk_timeout: u32,
    /// verification level required for the guild
    verification_level: u32,
    /// default message notification level
    default_message_notifications: u32,
    /// explicit content filter level
    explicit_content_filter: u32,
    /// roles in the guild
    roles: Vec<Role>,
    /// custom guild emojis
    emojis: Vec<Emoji>,
    /// enabled guild features
    features: Vec<GuildFeature>,
    /// required MFA level for the guild
    mfa_level: u32,
    /// application id of the guild creator if it is bot-created
    application_id: Option<Snowflake>,
    /// true if the server widget is enabled
    widget_enabled: Option<bool>,
    /// the channel id that the widget will generate an invite to, or `None` if set to no invite
    widget_channel_id: Option<Snowflake>,
    /// the id of the channel where guild notices such as welcome messages and boost events are posted
    system_channel_id: Option<Snowflake>,
    /// system channel flags
    system_channel_flags: u32,
    /// the id of the channel where guild with the `PUBLIC` feature can display rules and/or guidelines
    rules_channel_id: Option<Snowflake>,
    /// when this guild was joined at (only with the `GUILD_CREATE` event)
    joined_at: Option<DateTime<Utc>>,
    /// true if this guild is unavailable due to an outage (only with the `GUILD_CREATE` event
    unavailable: Option<bool>,
    /// total number of members in this guild (only with the `GUILD_CREATE` event
    member_count: Option<u32>,
    /// states of members currently in voice channels; lacks the guild_id key
    voice_states: Vec<VoiceState>,
    /// users in the guild
    members: Vec<GuildMember>,
    /// channels in the guild
    channels: Vec<RawChannel>,
    /// presences of the members in the guild, will only include non-offline members if the size is greater than `large_threshold`
    presences: Vec<Presence>,
    max_presences: Option<u32>,
    /// the maximum number of members for the guild
    max_members: u32,
    /// the vanity url code for the guild
    vanity_url_code: Option<String>,
    /// the description for the guild, if the guild is discoverable
    description: Option<String>,
    /// banner hash
    banner: Option<String>,
    /// premium tier (Server boost level)
    premium_tier: u8,
    /// the number of boosts this guild currently has
    premium_subscription_count: Option<u32>,
    /// the preffered locale of a guild with the `PUBLIC` feature; used in server discovery and notices from Discord
    preferred_locale: String,
    /// the id of the channel where admins and moderators of guild with the `PUBLIC` feature recieve notices from Discord
    public_updates_channel_id: Option<Snowflake>,
    /// the maximum amount of users in a video channel
    max_video_channel_users: Option<u32>,
    /// approximate number of members in this guild
    approximate_member_count: Option<u32>,
    /// approximate number of non-offline members in this guild
    approximate_presence_count: Option<u32>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct GuildFeature(String);
