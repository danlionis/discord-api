use crate::model::id::ApplicationId;
use crate::model::Emoji;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};

pub type Timestamp = u64;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct Activity {
    /// the activity's name
    pub name: String,
    /// activity type
    #[serde(rename = "type")]
    pub kind: ActivityType,
    /// stream url, is validated when type is 'Streaming'
    pub url: Option<String>,
    /// unix timestamp of when the activity was added to the user's session
    pub created_at: Timestamp,
    /// unix timestamps for start and/or end of the game
    pub timestamps: Option<Timestamps>,
    /// application id for the game
    pub application_id: Option<ApplicationId>,
    /// what the player is currently doing
    pub details: Option<String>,
    /// the user's current party status
    pub state: Option<String>,
    /// the emoji used for a custom status
    pub emoji: Option<Emoji>,
    // party: Option<Party>
    // assets: Option<Assets>,
    // secrets: Option<Secrets>,
    /// whether or not the activity is an instanced game session
    #[serde(default)]
    pub instance: bool,
    /// activity flags, describes what the payload includes
    #[serde(default)]
    pub flags: u32,
}

#[derive(Debug, SerializeRepr, DeserializeRepr, Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ActivityType {
    Game = 0,
    Streaming = 1,
    Listening = 2,
    Custom = 4,
    Competing = 5,
}

/// start and ending times for an activity
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct Timestamps {
    /// unix time in milliseconds of when the activity started
    pub start: Option<Timestamp>,
    /// unix time in milliseconds of when the activity ends
    pub end: Option<Timestamp>,
}
