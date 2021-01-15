use crate::model::{gateway::Opcode, UnavailableGuild};
use crate::model::{
    Channel, Guild, GuildMember, Message, MessageDelete, MessageUpdate, Presence, User, VoiceState,
};
use serde::{
    de::{DeserializeSeed, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};

use crate::model::gateway::dispatch::*;

/// Event received from the Gateway
#[derive(Debug, PartialEq)]
pub enum GatewayEvent {
    Dispatch(u64, Event),
    Heartbeat(u64),
    HeartbeatAck,
    Hello(Hello),
    InvalidSession(bool),
    Reconnect,
}

impl GatewayEvent {
    pub fn opcode(&self) -> Opcode {
        Opcode::from(self)
    }
}

/// A `GatewayEventSeed` is a Deserializer that contains information
/// about how to serialize a GatewayEvent such as the opcode, sequence number
/// and (if it is a dispatch event) the type of the event
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct GatewayEventSeed<'a> {
    pub op: Opcode,
    pub seq: Option<u64>,
    pub event_kind: Option<&'a str>,
}

impl<'a> GatewayEventSeed<'a> {
    /// Create a new `GatewayEventSeed` with the values already known
    pub(crate) fn new(op: Opcode, seq: Option<u64>, event_kind: Option<&'a str>) -> Self {
        Self {
            op,
            seq,
            event_kind,
        }
    }

    /// Create a `GatewayEventSeed` by reading in the incoming JSON and parsing the required values
    pub(crate) fn from_json_str(json_str: &'a str) -> Self {
        let op: Opcode =
            Self::find(json_str, r#""op":"#).expect(&format!("missing opcode: {}", json_str));
        let seq: Option<u64> = Self::find(json_str, r#""s":"#);

        // only search for type if event is dispatch
        let event_kind = if op == Opcode::Dispatch {
            Self::find_event_kind(json_str)
        } else {
            None
        };

        GatewayEventSeed {
            op,
            seq,
            event_kind,
        }
    }

    /// parse the event kind out of the json string
    /// returns `None` if the string is `null`
    fn find_event_kind(json_str: &'a str) -> Option<&'a str> {
        let key = r#""t":"#;

        let from = json_str.find(key)? + key.len();
        let to = json_str[from..].find([',', '}'].as_ref())?;
        let res = json_str[from..from + to].trim();

        match res {
            "null" => None,
            _ => Some(res.trim_matches('"')),
        }
    }

    fn find<T>(json_str: &str, key: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        let from = json_str.find(key)? + key.len();
        let to = json_str[from..].find([',', '}'].as_ref())?;
        let res = json_str[from..from + to].trim();

        T::from_str(res).ok()
    }
}

impl<'de> DeserializeSeed<'de> for GatewayEventSeed<'_> {
    type Value = GatewayEvent;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let fields = &["s", "op", "d", "t"];
        deserializer.deserialize_struct("GatewayEvent", fields, GatewayEventVisitor(self))
    }
}

/// Fields contained in a GatewayEvent response
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
    D,
    Op,
    S,
    T,
}

pub(crate) struct GatewayEventVisitor<'a>(GatewayEventSeed<'a>);

impl<'a> GatewayEventVisitor<'a> {
    fn find_field<'de, T, V>(map: &mut V, field: Field) -> Result<T, V::Error>
    where
        T: Deserialize<'de>,
        V: MapAccess<'de>,
    {
        let mut found = None;

        loop {
            match map.next_key::<Field>() {
                Ok(Some(key)) if key == field => found = Some(map.next_value()?),
                Ok(Some(_)) => {
                    map.next_value::<IgnoredAny>()?;
                    continue;
                }
                _ => {
                    break;
                }
            }
        }

        found.ok_or_else(|| {
            DeError::missing_field(match field {
                Field::D => "d",
                Field::Op => "op",
                Field::S => "s",
                Field::T => "t",
            })
        })
    }

    fn find_field_seed<'de, T, V>(map: &mut V, field: Field, seed: T) -> Result<T::Value, V::Error>
    where
        V: MapAccess<'de>,
        T: DeserializeSeed<'de>,
    {
        let mut found = None;

        loop {
            match map.next_key::<Field>() {
                Ok(Some(key)) if key == field => {
                    found = Some(map.next_value_seed(seed)?);
                    break;
                }
                Ok(Some(_)) => {
                    map.next_value::<IgnoredAny>()?;
                    continue;
                }
                _ => {
                    break;
                }
            }
        }

        found.ok_or_else(|| {
            DeError::missing_field(match field {
                Field::D => "d",
                Field::Op => "op",
                Field::S => "s",
                Field::T => "t",
            })
        })
    }

    fn ignore_all<'de, V>(map: &mut V) -> Result<(), V::Error>
    where
        V: MapAccess<'de>,
    {
        while let Ok(Some(_)) | Err(_) = map.next_key::<Field>() {
            map.next_value::<IgnoredAny>()?;
        }

        Ok(())
    }
}

impl<'de> Visitor<'de> for GatewayEventVisitor<'_> {
    type Value = GatewayEvent;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a GatewayEvent")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let op = self.0.op;
        let seq = self.0.seq;

        let res = match op {
            Opcode::Dispatch => {
                let dispatch_event = Self::find_field_seed(
                    &mut map,
                    Field::D,
                    DispatchEventSeed::new(self.0.event_kind),
                )?;
                GatewayEvent::Dispatch(seq.unwrap_or(0), dispatch_event)
            }
            Opcode::Hello => {
                let hello = Self::find_field(&mut map, Field::D)?;
                GatewayEvent::Hello(hello)
            }
            Opcode::Heartbeat => {
                let last_seq = Self::find_field(&mut map, Field::D)?;
                GatewayEvent::Heartbeat(last_seq)
            }
            Opcode::Reconect => GatewayEvent::Reconnect,
            Opcode::InvalidSession => {
                let resumable = Self::find_field(&mut map, Field::D)?;
                GatewayEvent::InvalidSession(resumable)
            }
            Opcode::HeartbeatACK => GatewayEvent::HeartbeatAck,
            // Opcode::Identify => {}
            // Opcode::PresenceUpdate => {}
            // Opcode::VoiceStateUpdate => {}
            // Opcode::Resume => {}
            // Opcode::RequestGuildMembers => {}
            _ => {
                panic!("unknown opcode");
            }
        };
        // ignore the rest of the fields
        Self::ignore_all(&mut map)?;

        Ok(res)
    }
}

/// A Gateway Dispatch Event
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Resume,
    MessageCreate(Box<Message>),
    MessageUpdate(Box<MessageUpdate>),
    MessageDelete(MessageDelete),
    MessageDeleteBulk(MessageDeleteBulk),
    MessageReactionAdd(Box<MessageReactionAdd>),
    MessageReactionRemove(MessageReactionRemove),
    MessageReactionRemoveAll(MessageReactionRemoveAll),
    MessageReactionRemoveEmoji(MessageReactionRemoveEmoji),
    ChannelCreate(Channel),
    ChannelDelete(Channel),
    ChannelUpdate(Channel),
    ChannelPinsUpdates(ChannelPinsUpdate),
    GuildCreate(Box<Guild>),
    GuildUpdate(Box<Guild>),
    GuildDelete(UnavailableGuild),
    GuildBanAdd(GuildBanAdd),
    GuildEmojisUpdate(GuildEmojisUpdate),
    GuildMembersChunk(GuildMembersChunk),
    GuildIntegrationsUpdate(GuildIntegrationsUpdate),
    GuildBanRemove(GuildBanRemove),
    GuildMemberAdd(GuildMember),
    GuildMemberRemove(GuildMemberRemove),
    GuildMemberUpdate(GuildMemberUpdate),
    GuildRoleCreate(GuildRoleCreate),
    GuildRoleUpdate(GuildRoleUpdate),
    GuildRoleDelete(GuildRoleDelete),
    PresenceUpdate(Presence),
    UserUpdate(User),
    VoiceStateUpdate(VoiceState),
    VoiceServerUpdate(VoiceServerUpdate),
    Ready(Ready),
    TypingStart(TypingStart),
    InviteCreate(InviteCreate),
    InviteDelete(InviteDelete),
    WebhooksUpdate(WebhooksUpdate),
}

impl Event {
    pub fn kind(&self) -> &str {
        match self {
            Event::Resume => "RESUME",
            Event::MessageCreate(_) => "MESSAGE_CREATE",
            Event::MessageUpdate(_) => "MESSAGE_UPDATE",
            Event::MessageDelete(_) => "MESSAGE_DELETE",
            Event::MessageDeleteBulk(_) => "MESSAGE_DELETE_BULK",
            Event::MessageReactionAdd(_) => "MESSAGE_REACTION_ADD",
            Event::MessageReactionRemove(_) => "MESSAGE_REACTION_REMOVE",
            Event::MessageReactionRemoveAll(_) => "MESSAGE_REACTION_REMOVE_ALL",
            Event::MessageReactionRemoveEmoji(_) => "MESSAGE_REACTOIN_REMOVE_EMOJI",
            Event::ChannelCreate(_) => "CHANNEL_CREATE",
            Event::ChannelDelete(_) => "CHANNEL_DELETE",
            Event::ChannelUpdate(_) => "CHANNEL_UDPATE",
            Event::ChannelPinsUpdates(_) => "CHANNEL_PINS_UPDATE",
            Event::GuildCreate(_) => "GUILD_CREATE",
            Event::GuildUpdate(_) => "GUILD_UPDATE",
            Event::GuildDelete(_) => "GUILD_DELETE",
            Event::GuildBanAdd(_) => "GUILD_BAN_ADD",
            Event::GuildBanRemove(_) => "GUILD_BAN_REMOVE",
            Event::GuildEmojisUpdate(_) => "GUILD_EMOJIS_UPDATE",
            Event::GuildMembersChunk(_) => "GUILD_MEMBERS_CHUNK",
            Event::GuildIntegrationsUpdate(_) => "GUILD_INTEGRATIONS_UPDATE",
            Event::GuildMemberAdd(_) => "GUILD_MEMBER_ADD",
            Event::GuildMemberRemove(_) => "GUILD_MEMBER_REMOVE",
            Event::GuildMemberUpdate(_) => "GUILD_MEMBER_UPDATE",
            Event::GuildRoleCreate(_) => "GUILD_ROLE_CREATE",
            Event::GuildRoleUpdate(_) => "GUILD_ROLE_UPDATE",
            Event::GuildRoleDelete(_) => "GUILD_ROLE_DELETE",
            Event::PresenceUpdate(_) => "PRESENCE_UPDATE",
            Event::UserUpdate(_) => "USER_UPDATE",
            Event::VoiceStateUpdate(_) => "VOICE_STATE_UPDATE",
            Event::VoiceServerUpdate(_) => "VOICE_SERVER_UPDATE",
            Event::Ready(_) => "READY",
            Event::TypingStart(_) => "TYPING_START",
            Event::InviteCreate(_) => "INVITE_CREATE",
            Event::InviteDelete(_) => "INVITE_DELETE",
            Event::WebhooksUpdate(_) => "WEBHOOKS_UPDATE",
        }
    }
}

pub(crate) struct DispatchEventSeed<'a> {
    event_kind: Option<&'a str>,
}

impl<'a> DispatchEventSeed<'a> {
    fn new(event_kind: Option<&'a str>) -> Self {
        DispatchEventSeed { event_kind }
    }
}

impl<'de> DeserializeSeed<'de> for DispatchEventSeed<'_> {
    type Value = Event;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let event_kind = self.event_kind.expect("event_kind required");

        let res = match event_kind {
            "READY" => Event::Ready(Ready::deserialize(deserializer)?),
            "RESUMED" => {
                IgnoredAny::deserialize(deserializer)?;
                Event::Resume
            }
            "CHANNEL_CREATE" => Event::ChannelCreate(Channel::deserialize(deserializer)?),
            "CHANNEL_UPDATE" => Event::ChannelUpdate(Channel::deserialize(deserializer)?),
            "CHANNEL_DELETE" => Event::ChannelDelete(Channel::deserialize(deserializer)?),
            "CHANNEL_PINS_UPDATE" => {
                Event::ChannelPinsUpdates(ChannelPinsUpdate::deserialize(deserializer)?)
            }
            "GUILD_BAN_ADD" => Event::GuildBanAdd(GuildBanAdd::deserialize(deserializer)?),
            "GUILD_BAN_REMOVE" => Event::GuildBanRemove(GuildBanRemove::deserialize(deserializer)?),
            "GUILD_CREATE" => Event::GuildCreate(Box::new(Guild::deserialize(deserializer)?)),
            "GUILD_DELETE" => Event::GuildDelete(UnavailableGuild::deserialize(deserializer)?),
            "GUILD_EMOJIS_UPDATE" => {
                Event::GuildEmojisUpdate(GuildEmojisUpdate::deserialize(deserializer)?)
            }
            "GUILD_INTEGRATIONS_UPDATE" => {
                Event::GuildIntegrationsUpdate(GuildIntegrationsUpdate::deserialize(deserializer)?)
            }
            "GUILD_MEMBER_ADD" => Event::GuildMemberAdd(GuildMember::deserialize(deserializer)?),
            "GUILD_MEMBER_REMOVE" => {
                Event::GuildMemberRemove(GuildMemberRemove::deserialize(deserializer)?)
            }
            "GUILD_MEMBER_UPDATE" => {
                Event::GuildMemberUpdate(GuildMemberUpdate::deserialize(deserializer)?)
            }
            "GUILD_MEMBERS_CHUNK" => {
                Event::GuildMembersChunk(GuildMembersChunk::deserialize(deserializer)?)
            }
            "GUILD_ROLE_CREATE" => {
                Event::GuildRoleCreate(GuildRoleCreate::deserialize(deserializer)?)
            }
            "GUILD_ROLE_DELETE" => {
                Event::GuildRoleDelete(GuildRoleDelete::deserialize(deserializer)?)
            }
            "GUILD_ROLE_UPDATE" => {
                Event::GuildRoleUpdate(GuildRoleUpdate::deserialize(deserializer)?)
            }
            "INVITE_CREATE" => Event::InviteCreate(InviteCreate::deserialize(deserializer)?),
            "INVITE_DELETE" => Event::InviteDelete(InviteDelete::deserialize(deserializer)?),
            "GUILD_UPDATE" => Event::GuildUpdate(Box::new(Guild::deserialize(deserializer)?)),
            "MESSAGE_CREATE" => Event::MessageCreate(Box::new(Message::deserialize(deserializer)?)),
            "MESSAGE_DELETE" => Event::MessageDelete(MessageDelete::deserialize(deserializer)?),
            "MESSAGE_DELETE_BULK" => {
                Event::MessageDeleteBulk(MessageDeleteBulk::deserialize(deserializer)?)
            }
            "MESSAGE_REACTION_ADD" => {
                Event::MessageReactionAdd(Box::new(MessageReactionAdd::deserialize(deserializer)?))
            }
            "MESSAGE_REACTION_REMOVE" => {
                Event::MessageReactionRemove(MessageReactionRemove::deserialize(deserializer)?)
            }
            "MESSAGE_REACTION_REMOVE_ALL" => Event::MessageReactionRemoveAll(
                MessageReactionRemoveAll::deserialize(deserializer)?,
            ),
            "MESSAGE_UPDATE" => {
                Event::MessageUpdate(Box::new(MessageUpdate::deserialize(deserializer)?))
            }
            "PRESENCE_UPDATE" => Event::PresenceUpdate(Presence::deserialize(deserializer)?),
            // "PRESENCES_REPLACE" => DispatchEvent::Unhandled,
            "TYPING_START" => Event::TypingStart(TypingStart::deserialize(deserializer)?),
            "USER_UPDATE" => Event::UserUpdate(User::deserialize(deserializer)?),
            "VOICE_SERVER_UPDATE" => {
                Event::VoiceServerUpdate(VoiceServerUpdate::deserialize(deserializer)?)
            }
            "VOICE_STATE_UPDATE" => Event::VoiceStateUpdate(VoiceState::deserialize(deserializer)?),
            "WEBHOOKS_UPDATE" => Event::WebhooksUpdate(WebhooksUpdate::deserialize(deserializer)?),
            _ => panic!("unknown event type"),
        };

        Ok(res)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Hello {
    pub heartbeat_interval: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_event_from_json() {
        let input = r#"{"op":0,"s":0,"t":null}"#;
        let seed = GatewayEventSeed::from_json_str(input);
        assert_eq!(seed, GatewayEventSeed::new(0.into(), Some(0), None));

        let input = r#"{"op":0,"s":1,"t":"READY"}"#;
        let seed = GatewayEventSeed::from_json_str(input);
        assert_eq!(
            seed,
            GatewayEventSeed::new(0.into(), Some(1), Some("READY"))
        );

        let input = r#"{"t":null,"s":null,"op":11,"d":null}"#;
        let seed = GatewayEventSeed::from_json_str(input);
        assert_eq!(seed, GatewayEventSeed::new(11.into(), None, None));
        GatewayEventSeed::new(0.into(), Some(1), Some("READY"));

        let input = r#"{"t":null,"s":null,"op":11,"d":null}"#;
        let seed = GatewayEventSeed::from_json_str(input);
        assert_eq!(seed, GatewayEventSeed::new(11.into(), None, None));
    }
}
