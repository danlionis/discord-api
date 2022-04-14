use crate::model::id::UserId;
use serde::{Deserialize, Serialize};

/// Base [Discord user] object
///
/// [Discord user]: https://discord.com/developers/docs/resources/user#user-object
#[derive(Clone, Hash, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct User {
    /// the user's id
    pub id: UserId,
    /// the user's username, not unique across the platform
    pub username: String,
    /// the user's 4-digit discord-tag
    pub discriminator: String,
    /// the user's [avatar hash]
    ///
    /// [avatar hash]: https://discord.com/developers/docs/reference#image-formatting
    pub avatar: Option<String>,
    /// whether the user belongs to an OAuth2 application
    #[serde(default)]
    pub bot: bool,
    /// whether the user is an Official Discord System user (part of the urgent message system)
    #[serde(default)]
    pub system: bool,
    /// whether the user has two factor enabled on their account
    #[serde(default)]
    pub mfa_enabled: bool,
    /// the user's chosen language option
    pub locale: Option<String>,
    /// whether the email on this account has been verified
    #[serde(default)]
    pub verified: bool,
    /// the user's email
    pub email: Option<String>,
    /// the [flags] on a user's account
    ///
    /// [flags]: https://discord.com/developers/docs/resources/user#user-object-user-flags
    #[serde(default)]
    pub flags: u64,
    /// the [type of Nitro subscription] on a user's account
    ///
    /// [type of Nitro subscription]: https://discord.com/developers/docs/resources/user#user-object-premium-types
    #[serde(default)]
    pub premium_type: u8,
    /// the public [flags] on a user's account
    ///
    /// [flags]: https://discord.com/developers/docs/resources/user#user-object-user-flags
    #[serde(default)]
    pub public_flags: u64,
}

impl User {
    /// Combination of username and discriminator
    ///
    /// e.g. username#1234
    pub fn tag(&self) -> String {
        return format!("{}#{}", self.username, self.discriminator);
    }
}
