#![allow(dead_code)]
//! Discord Library
//!
//! Features:
//! - No additional dependencies (eg. async_trait)
//! -

// extern crate chrono;
// extern crate hyper;
// extern crate hyper_tls;
// extern crate log;
// extern crate serde;
// extern crate serde_json;

#[cfg(feature = "cache")]
pub mod cache;

pub mod error;
pub mod gateway;
pub mod model;
pub mod rest;
pub mod traits;
pub mod wrapper;

mod snowflake;

pub use snowflake::Snowflake;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
