use crate::model::id::{EmojiId, RoleId};
use crate::model::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct Emoji {
    /// emoji id
    pub id: Option<EmojiId>,
    /// emoji name
    pub name: Option<String>,
    /// roles this emoji is whitelisted to
    #[serde(default)]
    pub roles: Vec<RoleId>,
    /// user that created this emoji
    pub user: Option<User>,
    /// whether this emoji must be wrapped in colons
    #[serde(default)]
    pub require_colons: bool,
    /// whether this emoji is managed
    #[serde(default)]
    pub managed: bool,
    /// whether this emoji is animated
    #[serde(default)]
    pub animated: bool,
    /// whether this emoji may be used, may be fale due to loss of server boosts
    #[serde(default)]
    pub available: bool,
}
