use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    id::{
        ApplicationCommandId, ApplicationId, AttachmentId, ChannelId, GuildId, InteractionId,
        MessageId, RoleId, UserId,
    },
    Attachment, Channel, GuildMember, Message, Role, User,
};

/// <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-structure>
///
/// `data` always present on application command, message component, and modal submit interaction types. It is optional for future-proofing against new interaction types
///
/// `member` is sent when the interaction is invoked in a guild, and `user` is sent when invoked in a DM
///
/// `locale` is available on all interaction types except PING
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Interaction {
    pub id: InteractionId,
    pub application_id: ApplicationId,
    pub r#type: InteractionType,
    pub data: Option<InteractionData>,
    pub guild_id: Option<GuildId>,
    pub channel_id: Option<ChannelId>,
    pub member: Option<GuildMember>,
    pub user: Option<User>,
    pub token: String,
    pub version: i32,
    pub message: Option<Message>,
    pub locale: Option<String>,
    pub guild_locale: Option<String>,
}

#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutocomplete = 4,
    ModalllSubmit = 5,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct InteractionData {
    pub id: ApplicationCommandId,
    name: String,
    r#type: i32,
    resolved: Option<ResolvedData>,
    options: Option<Vec<ApplicationCommandInteractionDataOption>>,
    guild_id: Option<GuildId>,
    custom_id: Option<String>,
    component_type: i32,
    values: Option<SelectOptionValues>,
    target_id: Option<Snowflake>,
    components: Option<Vec<MessageComponent>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct ResolvedData {
    users: Option<HashMap<UserId, User>>,
    members: Option<HashMap<UserId, GuildMember>>,
    roles: Option<HashMap<RoleId, Role>>,
    channels: Option<HashMap<ChannelId, Channel>>,
    messages: Option<HashMap<MessageId, Message>>,
    attachments: Option<HashMap<AttachmentId, Attachment>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ApplicationCommandInteractionDataOption {
    name: String,
    r#type: ApplicationCommandOptionType,
    // value: String, Int or double TODO:
    options: Option<Vec<ApplicationCommandInteractionDataOption>>,
    focused: Option<bool>,
}

#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub enum ApplicationCommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
    Mentionable = 9,
    Number = 10,
    Attachment = 11,
}
