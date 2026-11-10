# AGENTS.md — blazers-server

HTTP + features crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/lib.rs` declares every feature module `pub`; `src/app.rs` wires routes,
  middleware and `serve_http`.
- `AppState` (in `handlers.rs`) is the shared context: db pool, storage dir,
  http client, secrets key, ws/sse/log/pubsub hubs, rate limiter.
- All routes except health/register/login/openapi require
  `Authorization: Bearer <jwt>`; permissions enforced via `_permissions`.
- Admin dashboard: `public/index.html` embedded by `rust-embed` — HTMX, no
  build step. Prefer htmx + plain CSS over adding a JS framework.
- Realtime: WebSocket hub (`/ws`) + SSE streams (`/api/notifications/stream`,
  `/api/logs/stream`, `/api/pubsub/{name}/stream`).
- `cargo test -p blazers-server`; integration tests live in
  `crates/blazers/tests/api.rs`.
