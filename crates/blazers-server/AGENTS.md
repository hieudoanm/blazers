# AGENTS.md — blazers-server

HTTP + features crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/lib.rs` declares every feature module `pub`; `src/app/` wires routes
  (`routes.rs`), embedded assets (`assets.rs`) and `serve_http` (`serve.rs`).
  HTTP handlers live in `src/handlers/` split by domain; `src/cache/` and
  `src/kvproto/` hold the typed-cache ops and TCP protocol commands. `kvproto.rs` is the Redis-style inline-protocol
  TCP listener (`blazers kv`), sharing `CacheStore` with the REST cache API.
- `AppState` (in `handlers.rs`) is the shared context: db pool, storage dir,
  http client, secrets key, ws/sse/log/pubsub hubs, rate limiter.
- All routes except health/register/login/openapi require
  `Authorization: Bearer <jwt>`; permissions enforced via `_permissions`.
- Admin dashboard: `public/index.html` embedded by `rust-embed` — HTMX, no
  build step. Prefer htmx + plain CSS over adding a JS framework.
- Realtime: WebSocket hub (`/ws`) + SSE streams (`/api/notifications/stream`,
  `/api/logs/stream`, `/api/pubsub/{name}/stream`).
- `cargo test -p blazers-server`; unit tests live in per-module `tests/`
  dirs; integration tests live in `crates/blazers/tests/api/`.
