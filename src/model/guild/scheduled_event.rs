use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{
    id::{ChannelId, EntityId, EventId, GuildId, UserId},
    User,
};

/// A representation of a scheduled event in a guild.
///
/// <https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct GuildScheduledEvent {
    /// the id of the scheduled event
    id: EventId,
    /// the [`GuildId`] which the scheduled event belongs to
    guild_id: GuildId,
    /// the [`ChannelId`] in which the scheduled event will be hosted, or `None` if scheduled entity type is [`External`][EntityType::External]
    channel_id: Option<ChannelId>,
    /// the [`UserId`] of the user that created the scheduled event *
    creator_id: Option<UserId>,
    /// the name of the scheduled event
    name: String,
    /// the description of the scheduled event
    description: Option<String>,
    /// the time the scheduled event will start
    scheduled_start_time: DateTime<Utc>,
    /// the time the scheduled event will end, required if entity_type is [`External`][EntityType::External]
    scheduled_end_time: Option<DateTime<Utc>>,
    /// the privacy level of the scheduled event
    privacy_level: PrivacyLevel,
    /// the status of the scheduled event
    status: EventStatus,
    /// the type of the scheduled event
    entity_type: EntityType,
    /// the id of an entity associated with a guild scheduled event
    entity_id: Option<EntityId>,
    /// additional metadata for the guild scheduled event
    entity_metadata: Option<EntityMetadata>,
    /// the user that created the scheduled event
    creator: Option<User>,
    /// the number of users subscribed to the scheduled event
    user_count: Option<i32>,
    /// the cover image hash of the scheduled event
    image: Option<String>,
}

/// <https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-privacy-level>
#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[allow(missing_docs)]
pub enum PrivacyLevel {
    GuildOnly = 2,
}

/// <https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-status>
#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[allow(missing_docs)]
pub enum EventStatus {
    Scheduled = 1,
    Active = 2,
    Completed = 3,
    Canceled = 4,
}

/// [Field Requirements By Entity
/// Type](https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-field-requirements-by-entity-type)
#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[allow(missing_docs)]
pub enum EntityType {
    StageInstance = 1,
    Voice = 2,
    External = 3,
}

/// <https://discord.com/developers/docs/resources/guild-scheduled-event#guild-scheduled-event-object-guild-scheduled-event-entity-metadata>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct EntityMetadata {
    /// location of the event
    pub location: Option<String>,
}

/// Event body for
/// - [`Event::GuildScheduledEventUserAdd`][crate::model::gateway::Event::GuildScheduledEventUserAdd]
/// - [`Event::GuildScheduledEventUserRemove`][crate::model::gateway::Event::GuildScheduledEventUserRemove]
///
/// <https://discord.com/developers/docs/topics/gateway#guild-scheduled-event-user-add-guild-scheduled-event-user-add-event-fields>
/// <https://discord.com/developers/docs/topics/gateway#guild-scheduled-event-user-remove-guild-scheduled-event-user-remove-event-fields>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct GuildScheduledEventUser {
    /// id of the guild scheduled event
    guild_scheduled_event_id: EventId,
    /// id of the user
    user_id: UserId,
    /// id of the guild
    guild_id: GuildId,
}
