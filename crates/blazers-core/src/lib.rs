//! Blazers core: the storage layer and shared wire models.
//!
//! Everything persists to a single SQLite file — collections are dynamic
//! tables, records are rows, and the `_`-prefixed tables hold users, buckets,
//! files, webhooks, secrets, cron jobs, logs, notifications, permissions,
//! cache entries, pub/sub topics and websocket messages.

pub mod db;
pub mod models;
