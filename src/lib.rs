//! Low level Discord protocol library

#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

#[allow(dead_code)]
pub(crate) const LIB_NAME: &str = "discord-api";

/// Gateway Api version
pub const API_VERSION: u16 = 10;

pub mod error;
#[cfg(feature = "manager")]
pub mod manager;

pub mod proto;
pub use proto::*;

pub use error::Error;
pub use twilight_model as model;
