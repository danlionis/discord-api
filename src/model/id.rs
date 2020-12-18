use crate::Snowflake;
use serde::{Deserialize, Serialize};

/// implements traits for types that have an inner Snowflake
macro_rules! impl_snowflake {
    ($($name:ident),*) => {
        $(
        #[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub struct $name(Snowflake);

        impl std::ops::Deref for $name {
            type Target = Snowflake;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl std::convert::From<$name> for Snowflake {
            fn from(id: $name) -> Snowflake {
                id.0
            }
        }

        impl std::convert::From<$name> for u64 {
            fn from(id: $name) -> u64 {
                u64::from(id.0)
            }
        }

        impl std::convert::From<Snowflake> for $name {
            fn from(snowflake: Snowflake) -> Self {
                Self(snowflake)
            }
        }

        impl std::convert::From<u64> for $name {
            fn from(id: u64) -> Self {
                Self(Snowflake::new(id))
            }
        }

        impl std::convert::AsRef<u64> for $name {
            fn as_ref(&self) -> &u64 {
                &self
            }
        }
        )*
    };
}

impl_snowflake!(
    ApplicationId,
    UserId,
    GuildId,
    ChannelId,
    MessageId,
    RoleId,
    EmojiId,
    AttachmentId,
    StickerId,
    PackId,
    WebhookId
);
