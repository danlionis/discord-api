use crate::model::{
    id::{ChannelId, GuildId, UserId},
    Channel,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The thread metadata object contains a number of thread-specific channel fields that are not needed by other channel types.
///
/// <https://discord.com/developers/docs/resources/channel#thread-metadata-object>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ThreadMetadata {
    /// whether the thread is archived
    pub archived: bool,

    /// duration in minutes to automatically archive the thread after recent activity, can be set to: 60, 1440, 4320, 10080
    pub auto_archive_duration: i32,

    /// timestamp when the thread's archive status was last changed, used for calculating recent activity
    pub archive_timestamp: DateTime<Utc>,

    /// whether the thread is locked; when a thread is locked, only users with MANAGE_THREADS can unarchive it
    pub locked: bool,

    #[serde(default)]
    /// whether non-moderators can add other non-moderators to a thread; only available on private threads
    pub invitable: bool,

    /// timestamp when the thread was created; only populated for threads created after 2022-01-09
    pub create_timestamp: DateTime<Utc>,
}

/// A thread member is used to indicate whether a user has joined a thread or not.
///
/// <https://discord.com/developers/docs/resources/channel#thread-member-object>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ThreadMember {
    /// the id of the thread
    pub id: Option<ChannelId>,
    /// the id of the user
    pub user_id: Option<UserId>,
    /// the id of the guild. Only set in the [`ThreadMemberUpdate`][crate::model::gateway::Event::ThreadMemberUpdate] event
    pub guild_id: Option<GuildId>,
    /// the time the current user last joined the thread
    pub join_timestamp: DateTime<Utc>,
    /// any user-thread settings, currently only used for notifications
    pub flags: i32,
}

/// <https://discord.com/developers/docs/topics/gateway#thread-list-sync-thread-list-sync-event-fields>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ThreadListSync {
    /// the id of the guild
    pub guild_id: GuildId,
    /// the parent channel ids whose threads are being synced. If omitted, then threads were synced for the entire guild. This array may contain channel_ids that have no active threads as well, so you know to clear that data.
    pub channel_ids: Vec<ChannelId>,
    /// all active threads in the given channels that the current user can access
    pub threads: Vec<Channel>,
    /// all thread member objects from the synced threads for the current user, indicating which threads the current user has been added to
    pub members: Vec<ThreadMember>,
}

/// <https://discord.com/developers/docs/topics/gateway#thread-members-update-thread-members-update-event-fields>
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ThreadMembersUpdate {
    /// the id of the thread
    pub id: ChannelId,
    /// the id of the guild
    pub guild_id: GuildId,
    ///  the approximate number of members in the thread, capped at 50
    pub members_count: i32,
    ///  the users who were added to the thread
    pub added_members: Vec<ThreadMember>,
    /// the id of the users who were removed from the thread
    pub removed_member_ids: Vec<UserId>,
}
