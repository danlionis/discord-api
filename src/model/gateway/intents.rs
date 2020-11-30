/// GUILDS (1 << 0)
///   - GUILD_CREATE
///   - GUILD_UPDATE
///   - GUILD_DELETE
///   - GUILD_ROLE_CREATE
///   - GUILD_ROLE_UPDATE
///   - GUILD_ROLE_DELETE
///   - CHANNEL_CREATE
///   - CHANNEL_UPDATE
///   - CHANNEL_DELETE
///   - CHANNEL_PINS_UPDATE
pub const GUILDS: u32 = 1 << 0;

/// GUILD_MEMBERS (1 << 1)
///   - GUILD_MEMBER_ADD
///   - GUILD_MEMBER_UPDATE
///   - GUILD_MEMBER_REMOVE
pub const GUILD_MEMBERS: u32 = 1 << 1;

/// GUILD_BANS (1 << 2)
///   - GUILD_BAN_ADD
///   - GUILD_BAN_REMOVE
pub const GUILD_BANS: u32 = 1 << 2;

/// GUILD_EMOJIS (1 << 3)
///   - GUILD_EMOJIS_UPDATE
pub const GUILD_EMOJIS: u32 = 1 << 3;

/// GUILD_INTEGRATIONS (1 << 4)
///   - GUILD_INTEGRATIONS_UPDATE
pub const GUILD_INTEGRATIONS: u32 = 1 << 4;

/// GUILD_WEBHOOKS (1 << 5)
///   - WEBHOOKS_UPDATE
pub const GUILD_WEBHOOKS: u32 = 1 << 5;

/// GUILD_INVITES (1 << 6)
///   - INVITE_CREATE
///   - INVITE_DELETE
pub const GUILD_INVITES: u32 = 1 << 6;

/// GUILD_VOICE_STATES (1 << 7)
///   - VOICE_STATE_UPDATE
pub const GUILD_VOICE_STATES: u32 = 1 << 7;

/// GUILD_PRESENCES (1 << 8)
///   - PRESENCE_UPDATE
pub const GUILD_PRESENCES: u32 = 1 << 8;

/// GUILD_MESSAGES (1 << 9)
///   - MESSAGE_CREATE
///   - MESSAGE_UPDATE
///   - MESSAGE_DELETE
///   - MESSAGE_DELETE_BULK
pub const GUILD_MESSAGES: u32 = 1 << 9;

/// GUILD_MESSAGE_REACTIONS (1 << 10)
///   - MESSAGE_REACTION_ADD
///   - MESSAGE_REACTION_REMOVE
///   - MESSAGE_REACTION_REMOVE_ALL
///   - MESSAGE_REACTION_REMOVE_EMOJI
pub const GUILD_MESSAGE_REACTIONS: u32 = 1 << 10;

/// GUILD_MESSAGE_TYPING (1 << 11)
///   - TYPING_START
pub const GUILD_MESSAGE_TYPING: u32 = 1 << 11;

/// DIRECT_MESSAGES (1 << 12)
///   - MESSAGE_CREATE
///   - MESSAGE_UPDATE
///   - MESSAGE_DELETE
///   - CHANNEL_PINS_UPDATE
pub const DIRECT_MESSAGES: u32 = 1 << 12;

/// DIRECT_MESSAGE_REACTIONS (1 << 13)
///   - MESSAGE_REACTION_ADD
///   - MESSAGE_REACTION_REMOVE
///   - MESSAGE_REACTION_REMOVE_ALL
///   - MESSAGE_REACTION_REMOVE_EMOJI
pub const DIRECT_MESSAGE_REACTIONS: u32 = 1 << 13;

/// DIRECT_MESSAGE_TYPING (1 << 14)
///   - TYPING_START
pub const DIRECT_MESSAGE_TYPING: u32 = 1 << 14;

pub const PRIVILIGED: u32 = GUILD_PRESENCES | GUILD_MEMBERS;
pub const UNPRIVILIGED: u32 = GUILDS
    | GUILD_BANS
    | GUILD_EMOJIS
    | GUILD_INTEGRATIONS
    | GUILD_WEBHOOKS
    | GUILD_INVITES
    | GUILD_VOICE_STATES
    | GUILD_MESSAGES
    | GUILD_MESSAGE_REACTIONS
    | GUILD_MESSAGE_TYPING
    | DIRECT_MESSAGES
    | DIRECT_MESSAGE_REACTIONS
    | DIRECT_MESSAGE_TYPING;

pub const ALL: u32 = PRIVILIGED | UNPRIVILIGED;
