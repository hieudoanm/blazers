# AGENTS.md — blazers-server

HTTP + features crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/lib.rs` declares every feature module `pub`; `src/app/` wires routes
  (`routes.rs`), embedded assets (`assets.rs`) and `serve_http` (`serve.rs`).
  HTTP handlers live in `src/handlers/` split by domain; The cache store and `blazers kv` TCP listener live in the
  `blazers-kv` crate (`blazers_kv::cache::CacheStore`, `blazers_kv::kvproto`).
- `AppState` (in `handlers.rs`) is the shared context: db pool, storage dir,
  http client, secrets key, ws/sse/log/pubsub hubs, rate limiter.
- `mailer.rs` (SMTP via lettre, config in `_settings`), `backups.rs`
  (scheduled `VACUUM INTO` snapshots from `backup_schedule` setting).
- Record rules: five per-collection expressions (`*_rule`) evaluated by
  `rbac::check_rule_view`/`check_record_rule` against `blazers_core::filter`
  with an `@request.auth` context; `?filter=`/`?sort=`/`?expand=` on record
  lists; WS record-change broadcasts via `websocket::broadcast_record`.
- All routes except health/register/login/openapi require
  `Authorization: Bearer <jwt>`; permissions enforced via `_permissions`.
- Admin dashboard: `public/index.html` embedded by `rust-embed` — HTMX, no
  build step. Prefer htmx + plain CSS over adding a JS framework.
- Realtime: WebSocket hub (`/ws`) + SSE streams (`/api/notifications/stream`,
  `/api/logs/stream`, `/api/pubsub/{name}/stream`).
- `cargo test -p blazers-server`; unit tests live in per-module `tests/`
  dirs; integration tests live in `crates/blazers/tests/api/`.
