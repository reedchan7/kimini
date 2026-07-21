//! Discovery and lifecycle access for the shared local Kimi Code server.
//!
//! Native (`kimini`) and Web (`kimini-web`) share this path: probe recorded
//! instances / legacy lock / default loopback, then start `kimi web --no-open`
//! when nothing is healthy. The spawned server is detached so either app can
//! exit without tearing down the peer.

mod connection;
mod discovery;
mod health;
mod process;
mod source;

pub use connection::Connection;
pub use discovery::{Status, discover_connection};

use std::sync::atomic::AtomicBool;

/// Compatibility entry point used by the Web app.
pub fn discover(stop: &AtomicBool, notify: &dyn Fn(Status)) -> Option<String> {
    discover_connection(stop, notify).map(|connection| connection.web_url())
}
