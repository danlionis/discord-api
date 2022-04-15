use crate::model::{
    id::{IntegrationId, RoleId},
    User,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::id::{ApplicationId, GuildId};

/// <https://discord.com/developers/docs/resources/guild#integration-object-integration-structure>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct Integration {
    /// integration id
    pub id: IntegrationId,
    /// integration id
    pub guild_id: Option<GuildId>,
    /// integration name
    pub name: String,
    /// integration type (twitch, youtube, or discord)
    pub r#type: String,
    /// is this integration enabled
    pub enabled: bool,
    /// is this integration syncing
    pub syncing: Option<bool>,
    /// id that this integration uses for _subscribers_
    pub role_id: Option<RoleId>,
    /// whether emoticons should be synced for this integration (twitch only currently)
    pub enable_emoticons: Option<bool>,
    /// the behavior of expiring subscribers
    pub expire_behavior: Option<IntegrationExpireBehavior>,
    /// the grace period (in days) before expiring subscribers
    pub expire_grace_period: Option<i32>,
    /// user for this integration
    pub user: Option<User>,
    /// integration account information
    pub account: IntegrationAccount,
    /// when this integration was last synced
    pub synced_at: Option<DateTime<Utc>>,
    /// how many subscribers this integration has
    pub subscriber_count: Option<i32>,
    /// has this integration been revoked
    pub revoked: Option<bool>,
    /// the bot/OAuth2 application for discord integrations
    pub application: IntegrationApplication,
}

/// <https://discord.com/developers/docs/resources/guild#integration-object-integration-expire-behaviors>
#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[allow(missing_docs)]
pub enum IntegrationExpireBehavior {
    RemoveRole = 0,
    Kick = 1,
}

/// <https://discord.com/developers/docs/resources/guild#integration-application-object-integration-application-structure>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct IntegrationApplication {
    /// the id of the app
    pub id: ApplicationId,
    /// the name of the app
    pub name: String,
    /// the ion hash of the app
    pub icon: Option<String>,
    /// the description of the app
    pub description: String,
    /// the bot associated with this application
    pub bot: Option<User>,
}

/// <https://discord.com/developers/docs/resources/guild#integration-account-object-integration-account-structure>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct IntegrationAccount {
    /// id of the account
    pub id: String,
    /// name of the account
    pub name: String,
}

/// <https://discord.com/developers/docs/topics/gateway#integration-delete-integration-delete-event-fields>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct IntegrationDelete {
    /// integration id
    pub id: IntegrationId,
    /// id of the guild
    pub guild_id: GuildId,
    /// id of the bot/OAuth2 application for this discord integration
    pub application_id: Option<ApplicationId>,
}
