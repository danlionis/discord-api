use crate::model::id::{GuildId, RoleId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PartialUser {
    pub id: UserId,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClientStatus {
    pub desktop: Option<String>,
    pub mobile: Option<String>,
    pub web: Option<String>,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Presence {
    pub user: PartialUser,
    #[serde(default)]
    pub roles: Vec<RoleId>,
    // game: Option<Activity>,
    pub guild_id: Option<GuildId>,
    pub status: String,
    // activities: Vec<Activity>
    pub client_status: ClientStatus,
    pub premium_since: Option<DateTime<Utc>>,
    pub nick: Option<String>,
}
