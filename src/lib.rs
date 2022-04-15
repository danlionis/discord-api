//! Low level Discord protocol library

#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

#[allow(dead_code)]
pub(crate) const LIB_NAME: &str = "discord-api";

#[cfg(feature = "cache")]
pub mod cache;
pub mod error;
#[cfg(feature = "manager")]
pub mod manager;
pub mod proto;
pub mod util;

mod snowflake;

pub use error::Error;
pub use snowflake::Snowflake;
