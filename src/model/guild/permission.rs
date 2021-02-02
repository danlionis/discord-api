use crate::Snowflake;
use serde::{Deserialize, Serialize};

/// Channel Permission Overwrites
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct PermissonOverwrite {
    /// user or role id
    pub id: Snowflake,

    /// TODO: manually implement Serialize and Deserialize
    #[serde(rename = "type")]
    pub kind: i32,
    /// allow bit set
    pub allow: String,
    /// deny bit set
    pub deny: String,
}
