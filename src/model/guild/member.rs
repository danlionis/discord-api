use crate::model::id::{GuildId, RoleId};
use crate::model::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base Discord [Guild Member]
///
/// [Guild Member]: https://discord.com/developers/docs/resources/guild#guild-member-object
#[derive(Clone, Hash, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub struct GuildMember {
    /// the user this guild member represents
    pub user: Option<User>,
    /// the users guild nickname
    pub nick: Option<String>,
    /// array of [`Role`] object ids
    pub roles: Vec<RoleId>,
    /// when the user joined the guild
    pub joined_at: DateTime<Utc>,
    /// when the user started boosting the guild
    pub premium_since: Option<DateTime<Utc>>,
    /// whether the user is server-deafened in voice channels
    #[serde(rename = "deaf")]
    pub server_deaf: bool,
    /// whether the user is server-muted in voice channels
    #[serde(rename = "mute")]
    pub server_mute: bool,
    /// id of the guild
    pub guild_id: Option<GuildId>,
}
