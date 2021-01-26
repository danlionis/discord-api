use crate::model::id::{GuildId, UserId};
use crate::model::Activity;
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
    pub guild_id: Option<GuildId>,
    pub status: String,
    pub activities: Vec<Activity>,
    pub client_status: ClientStatus,
}
