use crate::model::gateway::MessageDelete;
use crate::model::{Guild, Message, Presence, User, VoiceState};
use serde::{Deserialize, Serialize};

/// A Gateway Dispatch Event
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    MessageCreate(Message),
    MessageUpdate(Message),
    MessageDelete(MessageDelete),
    GuildCreate(Guild),
    PresenceUpdate(Presence),
    VoiceStateUpdate(VoiceState),
    ChannelUpdate(),
    Ready(User),
    Close,
}
