//! Wire types shared by the HTTP API, MCP tools and tests.
//!
//! Split by domain; every public name is re-exported here so callers keep
//! using `blazers_core::models::*` unchanged.

mod auth;
mod cache;
mod collections;
mod cron;
mod error;
mod files;
mod logs;
mod notifications;
mod pubsub;
mod secrets;
mod webhooks;
mod ws;

pub use auth::*;
pub use cache::*;
pub use collections::*;
pub use cron::*;
pub use error::*;
pub use files::*;
pub use logs::*;
pub use notifications::*;
pub use pubsub::*;
pub use secrets::*;
pub use webhooks::*;
pub use ws::*;

#[cfg(test)]
mod tests;
