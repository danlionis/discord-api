use crate::model::{gateway::Opcode, UnavailableGuild};
use crate::model::{
    Channel, Guild, GuildMember, Message, MessageDelete, MessageUpdate, Presence, User, VoiceState,
};
use serde::{
    de::{DeserializeSeed, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};

use crate::model::gateway::dispatch::*;

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

        // don't bother to parse the event kind if the event is not a dispatch
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
        let to = json_str[from..].find(|c: char| [',', '}'].contains(&c))?;
        let res = json_str[from..from + to].trim();

        match res {
            "null" => None,
            _ => Some(res.trim_matches('"')),
        }
    }

    fn find<T: std::str::FromStr>(json_str: &str, key: &str) -> Option<T> {
        let from = json_str.find(key)? + key.len();
        let to = json_str[from..].find(|c: char| [',', '}'].contains(&c))?;
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

impl<'de> Visitor<'de> for GatewayEventVisitor<'_> {
    type Value = GatewayEvent;

    fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let op = self.0.op;
        let seq = self.0.seq;

        let res = match op {
            Opcode::Dispatch => {
                let dispatch_event = find_field_seed(
                    &mut map,
                    Field::D,
                    DispatchEventSeed::new(self.0.event_kind),
                )?;
                GatewayEvent::Dispatch(seq.unwrap_or(0), dispatch_event)
            }
            Opcode::Hello => {
                let hello = find_field(&mut map, Field::D)?;
                GatewayEvent::Hello(hello)
            }
            Opcode::Heartbeat => {
                let last_seq = find_field(&mut map, Field::D)?;
                GatewayEvent::Heartbeat(last_seq)
            }
            Opcode::Reconect => GatewayEvent::Reconnect,
            Opcode::InvalidSession => {
                let resumable = find_field(&mut map, Field::D)?;
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
        ignore_all(&mut map)?;

        Ok(res)
    }
}

#[derive(Debug, PartialEq)]
pub enum GatewayEvent {
    Dispatch(u64, DispatchEvent),
    Heartbeat(u64),
    HeartbeatAck,
    Hello(Hello),
    InvalidSession(bool),
    Reconnect,
    Identify,
    PresenceUpdate,
    VoiceStateUpdate,
    Resume,
    RequestGuildMembers,
}

/// A Gateway Dispatch Event
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum DispatchEvent {
    Resume,
    MessageCreate(Message),
    MessageUpdate(MessageUpdate),
    MessageDelete(MessageDelete),
    MessageDeleteBulk(MessageDeleteBulk),
    MessageReactionAdd(MessageReactionAdd),
    MessageReactionRemove(MessageReactionRemove),
    MessageReactionRemoveAll(MessageReactionRemoveAll),
    MessageReactionRemoveEmoji(MessageReactionRemoveEmoji),
    ChannelCreate(Channel),
    ChannelDelete(Channel),
    ChannelUpdate(Channel),
    ChannelPinsUpdates(Channel),
    GuildCreate(Guild),
    GuildUpdate(Guild),
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

pub(crate) struct DispatchEventSeed<'a> {
    event_kind: Option<&'a str>,
}

impl<'a> DispatchEventSeed<'a> {
    fn new(event_kind: Option<&'a str>) -> Self {
        DispatchEventSeed { event_kind }
    }
}

impl<'de> DeserializeSeed<'de> for DispatchEventSeed<'_> {
    type Value = DispatchEvent;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let event_kind = self.event_kind.expect("event_kind required");

        let res = match event_kind {
            "READY" => DispatchEvent::Ready(Ready::deserialize(deserializer)?),
            "RESUMED" => {
                IgnoredAny::deserialize(deserializer)?;
                DispatchEvent::Resume
            }
            "CHANNEL_CREATE" => DispatchEvent::ChannelCreate(Channel::deserialize(deserializer)?),
            "CHANNEL_UPDATE" => DispatchEvent::ChannelUpdate(Channel::deserialize(deserializer)?),
            "CHANNEL_DELETE" => DispatchEvent::ChannelDelete(Channel::deserialize(deserializer)?),
            "CHANNEL_PINS_UPDATE" => {
                DispatchEvent::ChannelPinsUpdates(Channel::deserialize(deserializer)?)
            }
            "GUILD_BAN_ADD" => DispatchEvent::GuildBanAdd(GuildBanAdd::deserialize(deserializer)?),
            "GUILD_BAN_REMOVE" => {
                DispatchEvent::GuildBanRemove(GuildBanRemove::deserialize(deserializer)?)
            }
            "GUILD_CREATE" => DispatchEvent::GuildCreate(Guild::deserialize(deserializer)?),
            "GUILD_DELETE" => {
                DispatchEvent::GuildDelete(UnavailableGuild::deserialize(deserializer)?)
            }
            "GUILD_EMOJIS_UPDATE" => {
                DispatchEvent::GuildEmojisUpdate(GuildEmojisUpdate::deserialize(deserializer)?)
            }
            "GUILD_INTEGRATIONS_UPDATE" => DispatchEvent::GuildIntegrationsUpdate(
                GuildIntegrationsUpdate::deserialize(deserializer)?,
            ),
            "GUILD_MEMBER_ADD" => {
                DispatchEvent::GuildMemberAdd(GuildMember::deserialize(deserializer)?)
            }
            "GUILD_MEMBER_REMOVE" => {
                DispatchEvent::GuildMemberRemove(GuildMemberRemove::deserialize(deserializer)?)
            }
            "GUILD_MEMBER_UPDATE" => {
                DispatchEvent::GuildMemberUpdate(GuildMemberUpdate::deserialize(deserializer)?)
            }
            "GUILD_MEMBERS_CHUNK" => {
                DispatchEvent::GuildMembersChunk(GuildMembersChunk::deserialize(deserializer)?)
            }
            "GUILD_ROLE_CREATE" => {
                DispatchEvent::GuildRoleCreate(GuildRoleCreate::deserialize(deserializer)?)
            }
            "GUILD_ROLE_DELETE" => {
                DispatchEvent::GuildRoleDelete(GuildRoleDelete::deserialize(deserializer)?)
            }
            "GUILD_ROLE_UPDATE" => {
                DispatchEvent::GuildRoleUpdate(GuildRoleUpdate::deserialize(deserializer)?)
            }
            "INVITE_CREATE" => {
                DispatchEvent::InviteCreate(InviteCreate::deserialize(deserializer)?)
            }
            "INVITE_DELETE" => {
                DispatchEvent::InviteDelete(InviteDelete::deserialize(deserializer)?)
            }
            "GUILD_UPDATE" => DispatchEvent::GuildUpdate(Guild::deserialize(deserializer)?),
            "MESSAGE_CREATE" => DispatchEvent::MessageCreate(Message::deserialize(deserializer)?),
            "MESSAGE_DELETE" => {
                DispatchEvent::MessageDelete(MessageDelete::deserialize(deserializer)?)
            }
            "MESSAGE_DELETE_BULK" => {
                DispatchEvent::MessageDeleteBulk(MessageDeleteBulk::deserialize(deserializer)?)
            }
            "MESSAGE_REACTION_ADD" => {
                DispatchEvent::MessageReactionAdd(MessageReactionAdd::deserialize(deserializer)?)
            }
            "MESSAGE_REACTION_REMOVE" => DispatchEvent::MessageReactionRemove(
                MessageReactionRemove::deserialize(deserializer)?,
            ),
            "MESSAGE_REACTION_REMOVE_ALL" => DispatchEvent::MessageReactionRemoveAll(
                MessageReactionRemoveAll::deserialize(deserializer)?,
            ),
            "MESSAGE_UPDATE" => {
                DispatchEvent::MessageUpdate(MessageUpdate::deserialize(deserializer)?)
            }
            "PRESENCE_UPDATE" => {
                DispatchEvent::PresenceUpdate(Presence::deserialize(deserializer)?)
            }
            // "PRESENCES_REPLACE" => DispatchEvent::Unhandled,
            "TYPING_START" => DispatchEvent::TypingStart(TypingStart::deserialize(deserializer)?),
            "USER_UPDATE" => DispatchEvent::UserUpdate(User::deserialize(deserializer)?),
            "VOICE_SERVER_UPDATE" => {
                DispatchEvent::VoiceServerUpdate(VoiceServerUpdate::deserialize(deserializer)?)
            }
            "VOICE_STATE_UPDATE" => {
                DispatchEvent::VoiceStateUpdate(VoiceState::deserialize(deserializer)?)
            }
            "WEBHOOKS_UPDATE" => {
                DispatchEvent::WebhooksUpdate(WebhooksUpdate::deserialize(deserializer)?)
            }
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
    fn test_gateway_event_seed() {
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
