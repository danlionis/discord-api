//!
//!
//!

use serde::{self, de::Visitor, Deserialize, Serialize};
use std::convert::AsRef;
use std::ops::Deref;
use std::str::FromStr;

/// The `Snowflake` type is used for uniqely identifiable descriptors (IDs) across Discord
///
/// A `Snowflake` is represented by a `u64` and will always be serialized as a String to prevent
/// integer overflows on some languages
///
/// [Reference]
///
/// [Reference]: https://discord.com/developers/docs/reference#snowflakes
#[derive(Default, PartialEq, Eq, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct Snowflake(u64);

impl Snowflake {
    /// Create a new Snoflake
    pub fn new(v: u64) -> Self {
        Snowflake(v)
    }

    /// Milliseconds since Discord Epoch
    pub fn timestamp(&self) -> u64 {
        (self.0 >> 22) + 1420070400000
    }

    /// WorkerId this Snowflake was generated on
    pub fn internal_worker_id(&self) -> u64 {
        (self.0 & 0x3E0000) >> 17
    }

    /// ProcessId this Snowflake was generated on
    pub fn internal_process_id(&self) -> u64 {
        (self.0 & 0x1F000) >> 12
    }

    /// For every ID that is generated on that process, this number is incremented
    pub fn increment(&self) -> u64 {
        self.0 & 0xFFF
    }

    /// Whether this Snowflake is a safe JavaScript integer
    pub fn is_safe(&self) -> bool {
        self.0 <= MAX_SAFE_INTEGER
    }
}

impl std::fmt::Debug for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SnowflakeVisitor)
    }
}

const MAX_SAFE_INTEGER: u64 = (2 << 52) - 1;

impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl FromStr for Snowflake {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse()?;
        Ok(Snowflake(id))
    }
}

impl From<u64> for Snowflake {
    fn from(id: u64) -> Self {
        Snowflake(id)
    }
}

impl From<Snowflake> for u64 {
    fn from(snowflake: Snowflake) -> u64 {
        snowflake.0
    }
}

impl Deref for Snowflake {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<u64> for Snowflake {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

struct SnowflakeVisitor;

impl<'de> Visitor<'de> for SnowflakeVisitor {
    type Value = Snowflake;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str("a u64 snowflake")
    }

    // fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
    //     Ok(Snowflake(v))
    // }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse::<u64>()
            .map(Snowflake)
            .map_err(|_| serde::de::Error::custom("unknown value"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    // #[test]
    // fn safe_integer() {
    //     let safe_int = Snowflake::from(123);
    //     assert_tokens(&safe_int, &[Token::U64(123)]);
    // }

    // #[test]
    // fn max_safe_integer() {
    //     let max_safe_int = Snowflake::from(MAX_SAFE_INTEGER);
    //     assert_tokens(&max_safe_int, &[Token::U64(9007199254740991)]);
    // }

    // #[test]
    // fn unsafe_integer() {
    //     let unsafe_int = Snowflake::from(MAX_SAFE_INTEGER + 1);
    //     assert_tokens(&unsafe_int, &[Token::String("9007199254740992")]);
    // }
}
