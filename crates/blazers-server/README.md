# blazers-server

The Blazers HTTP server: REST API, realtime streams, feature modules and the
embedded admin dashboard.

## Modules

| Module | Feature |
| ------ | ------- |
| `app.rs` | Router assembly, `serve_http` bootstrap, `rust-embed` admin UI at `/`, request logging, and the large `#[cfg(test)]` suite |
| `handlers.rs` | `AppState`, auth extractors, collections/records CRUD, buckets/files, cache, notifications, logs, cron job handlers, backup |
| `auth.rs` / `rbac.rs` | JWT issue/verify (bcrypt passwords), `_permissions` enforcement |
| `websocket.rs` | `/ws` hub: connections, messages, broadcast |
| `notification.rs` / `log.rs` / `pubsub.rs` | SSE hubs + `/api/*/stream` endpoints |
| `cache.rs` | In-memory KV cache persisted to `_cache` with TTL eviction |
| `cronjobs.rs` | Cron parser + scheduler + job CRUD handlers |
| `webhook.rs` | Webhook CRUD, event payloads, signed delivery (HMAC-SHA256) |
| `secrets.rs` | AES-256-GCM encrypted `_secrets` vault (`$BLAZERS_SECRETS_KEY` or `secrets.key`) |
| `image.rs` | Upload sniffing (`infer`), thumbnails (`image` crate) |
| `import_export.rs` | `/api/export`, `/api/import` JSON snapshots (`ExportData`) |
| `rate_limit.rs` | Token-bucket middleware (200 rps / 100 burst, per-IP) |
| `content_type.rs` | JSON content-type enforcement middleware |
| `openapi.rs` | Runtime-generated `/api/openapi.json` + Swagger UI at `/api/docs` |

`public/` is embedded into the binary via `rust-embed` — edit it and rebuild;
there is no separate frontend build.

## Rules

- Handlers take `State<Arc<AppState>>`; `AppState` lives in `handlers.rs`.
- Feature modules may not import each other cyclically — shared helpers go in
  `handlers.rs` or `blazers-core`.
- Admin dashboard is HTMX + CSS in `public/index.html`; keep it framework-free.
- `cargo test -p blazers-server` runs ~250 tests.
