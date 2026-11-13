//! SQLite storage: schema, migrations and per-domain query helpers.
//!
//! Split by domain under `db/`; every public name is re-exported here so
//! callers keep using `blazers_core::db::*` unchanged.

mod cache;
mod collections;
mod core;
mod cron;
mod files;
mod functions;
mod logs;
mod migrate;
mod notifications;
mod pubsub;
mod records;
mod records_query;
mod schema;
mod secrets;
mod settings;
mod transfer;
mod users;
mod validate;
mod webhooks;
mod ws;

pub use cache::*;
pub use collections::*;
pub use core::*;
pub use cron::*;
pub use files::*;
pub use functions::*;
pub use logs::*;
pub use migrate::*;
pub use notifications::*;
pub use pubsub::*;
pub use records::*;
pub use records_query::*;
pub use schema::*;
pub use secrets::*;
pub use settings::*;
pub use transfer::*;
pub use users::*;
pub use validate::*;
pub use webhooks::*;
pub use ws::*;

#[cfg(test)]
mod tests;
