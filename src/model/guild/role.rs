use crate::model::id::RoleId;
use crate::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct Role {
    /// role id
    id: RoleId,
    /// role name
    name: String,
    /// integer representaion of hexadeciaml color code
    color: i32,
    /// if this role if pinned in the user listing
    hoist: bool,
    /// position of this role
    position: usize,
    /// permission bit set
    // #[serde(rename = "permissions_new")]
    permissions: Snowflake,
    /// whether this role is managed by an integration
    managed: bool,
    /// whether this role is mentionable
    mentionable: bool,
}
