use crate::model::id::{ChannelId, GuildId, MessageId};
use crate::model::User;
use crate::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Ready {
    #[serde(rename = "v")]
    pub version: u16,
    pub user: User,
    pub session_id: String,
    pub shard: Option<(Snowflake, u16)>,
}
#[derive(Debug, Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Empty {}

#[derive(Debug, Deserialize)]
pub struct GatewayPayload<D> {
    #[serde(rename = "t")]
    pub kind: Option<String>,
    #[serde(rename = "s")]
    pub seq: Option<u64>,
    #[serde(rename = "op")]
    pub opcode: u8,
    #[serde(rename = "d")]
    pub data: D,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct MessageDelete {
    id: MessageId,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
}
