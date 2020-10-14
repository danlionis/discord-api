use crate::model::id::ApplicationId;

pub struct Activity {
    name: String,
    #[serde(rename = "type")]
    kind: u8,
    url: Option<String>,
    created_at: u32,
    // timestamp: Timestamps
    application_id: Option<ApplicationId>
    details: Option<String>,
    state: Option<String>,
    // emoji: Option<Emoji>,
    // party: Option<Party>
    // assets: Option<Assets>,
    // secrets: Option<Secrets>,
    #[serde(default)]
    instance: bool,
    #[serde(default)]
    flags: u8,
}
