# blazers-core

The storage layer and shared wire models for Blazers.

- `src/db.rs` — SQLite schema (`_collections`, `_users`, `_buckets`, `_files`,
  `_webhooks`, `_secrets`, `_cron_jobs`, `_logs`, `_notifications`,
  `_permissions`, `_cache`, `_pubsub_*`, `_ws_*`), migrations, and every SQL
  function. Collections are dynamic tables created from their JSON schema.
- `src/models.rs` — serde wire types (`Collection`, `Record`, `Bucket`,
  `FileRecord`, `User`, …) and `AppError`, which maps to HTTP status codes.

## Rules

- No axum handlers here — this crate is HTTP-agnostic (it links `axum` only
  because `models.rs` implements `IntoResponse` for `AppError`).
- All persistence goes through `rusqlite::Connection`; the pool type is
  `deadpool::managed::Pool<db::ConnectionManager>`.
- Data directory: `db::data_dir()` → `$BLAZERS_DATA` or `~/.blazers`.
- `cargo test -p blazers-core` runs the db/models unit tests.
