use twilight_model::gateway::{
    payload::outgoing::{identify::IdentifyProperties, update_presence::UpdatePresencePayload},
    Intents,
};

use crate::LIB_NAME;

/// Connection Config
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(missing_docs)]
pub struct Config {
    pub gateway_url: Option<String>,
    pub identify_properties: IdentifyProperties,
    pub intents: Intents,
    pub large_threshold: u64,
    pub presence: Option<UpdatePresencePayload>,
    pub shard: [u64; 2],
    pub token: String,
}

impl From<(&str, Intents)> for Config {
    fn from((token, intents): (&str, Intents)) -> Self {
        Config::new(token, intents)
    }
}

impl Config {
    /// create a new config
    pub fn new<S>(token: S, intents: Intents) -> Self
    where
        S: Into<String>,
    {
        Config {
            gateway_url: None,
            identify_properties: IdentifyProperties::new(LIB_NAME, LIB_NAME, std::env::consts::OS),
            intents,
            large_threshold: 50,
            presence: None,
            shard: [0, 1],
            token: token.into(),
        }
    }

    /// set the initial presence
    pub fn presence(mut self, presence: UpdatePresencePayload) -> Self {
        self.presence = Some(presence);
        self
    }

    /// set the gateway url
    pub fn identify_properties(mut self, props: IdentifyProperties) -> Self {
        self.identify_properties = props;
        self
    }
    /// set the gateway url
    pub fn large_threshold(mut self, large_threshold: u64) -> Self {
        self.large_threshold = large_threshold;
        self
    }
    /// set the gateway url
    pub fn shard(mut self, shard: [u64; 2]) -> Self {
        self.shard = shard;
        self
    }
    /// set the gateway url
    pub fn gateway_url(mut self, url: String) -> Self {
        self.gateway_url = Some(url);
        self
    }
}
