//! RestClient connections to the Discord API
// TODO: remove hyper, only create the requests
// let the appliction handle sending

mod gateway;
mod message;
mod user;

pub use gateway::*;
pub use message::*;
pub use user::*;

mod routes;
pub use routes::Route;

#[cfg(feature = "rest")]
pub mod client;
