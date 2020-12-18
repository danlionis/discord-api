use crate::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct PermissonOverwrite {
    /// user or role id
    id: Snowflake,

    /// TODO: manually implement Serialize and Deserialize
    #[serde(rename = "type")]
    kind: i32,
    allow: String,
    deny: String,
}
