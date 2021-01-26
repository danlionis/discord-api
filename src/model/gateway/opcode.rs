use crate::model::gateway::{GatewayCommand, GatewayEvent};
use serde::{Deserialize, Serialize};

/// Opcode values for Gateway Events
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub(crate) enum Opcode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatACK = 11,
}

impl From<&GatewayEvent> for Opcode {
    fn from(event: &GatewayEvent) -> Self {
        match event {
            GatewayEvent::Dispatch(_, _) => Opcode::Dispatch,
            GatewayEvent::Heartbeat(_) => Opcode::Heartbeat,
            GatewayEvent::Reconnect => Opcode::Reconect,
            GatewayEvent::InvalidSession(_) => Opcode::InvalidSession,
            GatewayEvent::Hello(_) => Opcode::Hello,
            GatewayEvent::HeartbeatAck => Opcode::HeartbeatACK,
        }
    }
}

impl From<&GatewayCommand> for Opcode {
    fn from(command: &GatewayCommand) -> Self {
        match command {
            GatewayCommand::Heartbeat(_) => Opcode::Heartbeat,
            GatewayCommand::Identify(_) => Opcode::Identify,
            GatewayCommand::Resume(_) => Opcode::Resume,
            GatewayCommand::UpdateVoiceState(_) => Opcode::VoiceStateUpdate,
            GatewayCommand::UpdateStatus(_) => Opcode::PresenceUpdate,
            GatewayCommand::RequestGuildMembers(_) => Opcode::RequestGuildMembers,
        }
    }
}

impl std::convert::From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::Dispatch,
            1 => Opcode::Heartbeat,
            2 => Opcode::Identify,
            3 => Opcode::PresenceUpdate,
            4 => Opcode::VoiceStateUpdate,
            6 => Opcode::Resume,
            7 => Opcode::Reconect,
            8 => Opcode::RequestGuildMembers,
            9 => Opcode::InvalidSession,
            10 => Opcode::Hello,
            11 => Opcode::HeartbeatACK,
            _ => panic!("unknown opcode"),
        }
    }
}

impl std::str::FromStr for Opcode {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u8::from_str(s).map(From::from)
    }
}

impl<'de> Deserialize<'de> for Opcode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = u8::deserialize(deserializer)?;

        Ok(Opcode::from(v))
    }
}

impl Serialize for Opcode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = self.clone() as u8;
        serializer.serialize_u8(v)
    }
}
