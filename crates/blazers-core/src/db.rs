//! SQLite storage: schema, migrations and per-domain query helpers.
//!
//! Split by domain under `db/`; every public name is re-exported here so
//! callers keep using `blazers_core::db::*` unchanged.

mod cache;
mod collections;
mod core;
mod cron;
mod files;
mod logs;
mod notifications;
mod pubsub;
mod records;
mod schema;
mod secrets;
mod users;
mod validate;
mod webhooks;
mod ws;

pub use cache::*;
pub use collections::*;
pub use core::*;
pub use cron::*;
pub use files::*;
pub use logs::*;
pub use notifications::*;
pub use pubsub::*;
pub use records::*;
pub use schema::*;
pub use secrets::*;
pub use users::*;
pub use validate::*;
pub use webhooks::*;
pub use ws::*;

#[cfg(test)]
mod tests;
