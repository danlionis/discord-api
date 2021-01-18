use crate::model::id::{GuildId, RoleId, UserId};
use crate::model::Activity;
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
    pub guild_id: Option<GuildId>,
    pub status: String,
    pub activities: Vec<Activity>,
    pub client_status: ClientStatus,
}
