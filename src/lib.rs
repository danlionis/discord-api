//! Low level Discord protocol library

// extern crate chrono;
// extern crate hyper;
// extern crate hyper_tls;
// extern crate log;
// extern crate serde;
// extern crate serde_json;

#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]

pub(crate) const LIB_NAME: &str = "discord-api";

#[cfg(feature = "cache")]
pub mod cache;
pub mod proto;

pub mod error;
pub mod model;
pub mod rest;
pub mod util;

mod snowflake;

pub use error::Error;
pub use snowflake::Snowflake;
