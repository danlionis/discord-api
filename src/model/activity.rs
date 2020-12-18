use crate::model::id::ApplicationId;
use crate::model::Emoji;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};

pub type Timestamp = u64;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct Activity {
    name: String,
    #[serde(rename = "type")]
    kind: ActivityType,
    url: Option<String>,
    /// unix timestamp of when the activity was added to the user's session
    created_at: Timestamp,
    timestamps: Option<Timestamps>,
    application_id: Option<ApplicationId>,
    details: Option<String>,
    state: Option<String>,
    emoji: Option<Emoji>,
    // party: Option<Party>
    // assets: Option<Assets>,
    // secrets: Option<Secrets>,
    #[serde(default)]
    instance: bool,
    #[serde(default)]
    flags: u8,
}

#[derive(Debug, SerializeRepr, DeserializeRepr, Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum ActivityType {
    Game = 0,
    Streaming = 1,
    Listening = 2,
    Custom = 4,
    Competing = 5,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct Timestamps {
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}
