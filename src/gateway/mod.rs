//! gateway allows for recieving events from the discord api in realtime
//!
//! # Example
//!
//! ```no_run
//! #[tokio::main]
//! async fn main() -> Result<(), discord::Error> {
//!     let (mut shard, conn) = discord::gateway::new("--your_token--");
//!     let conn = tokio::spawn(conn);
//!
//!     let event = shard.recv_event().await.unwrap();
//!     assert_eq!(event.kind(), "READY");
//!     Ok(())
//! }
//!
//! ```

mod shard;
mod socket;

pub use shard::*;
