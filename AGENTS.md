# AGENTS.md — Blazers

Guidance for AI agents working in this repository.

## What this is

Blazers is a single-binary Back-end as a Service written in Rust — auth, a
schema-less document database on SQLite, file buckets, realtime (WebSocket +
SSE), webhooks, cron jobs, an encrypted secrets vault, cache, pub/sub,
notifications, an embedded admin dashboard, OpenAPI docs and an MCP server.
Inspired by PocketBase and Supabase; see the feature comparison in
[`README.md`](./README.md).

## Repository layout

| Path                        | What it is                                                                                                                                      |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/blazers-core`       | Storage layer: `db/` (schema, migrations, all SQL, split by domain) + `models/` (wire types, `AppError`, split by domain). See `crates/blazers-core/AGENTS.md`.               |
| `crates/blazers-server`     | Everything HTTP: feature modules + `app.rs` (router, embedded dashboard via `rust-embed`, `serve_http`). See `crates/blazers-server/AGENTS.md`. |
| `crates/blazers-mcp`        | MCP stdio server (newline-delimited JSON-RPC 2.0) exposing the DB as tools. See `crates/blazers-mcp/AGENTS.md`.                                 |
| `crates/blazers`            | Thin binary: `blazers` / `blazers serve` → HTTP, `blazers mcp serve` → MCP.                                                                     |
| `packages/landing`          | Next.js + Tailwind landing page **and** docs site (`/docs/*`) → GitHub Pages. See `packages/landing/AGENTS.md`.                                |
| `.github/workflows/ci.yaml` | Pages deploy + release binary jobs.                                                                                                             |

## Commands

```bash
cargo build --workspace          # debug build
cargo test --workspace           # all tests (unit + integration)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
make all                         # fmt-check + clippy + test + release build
```

Run locally: `cargo run -p blazers -- serve` (or `./target/release/blazers`).
Data lives in `$BLAZERS_DATA` (default `~/.blazers`): `data.db`, `secrets.key`,
`uploads/`.

## Conventions

- **Rust 2024 edition**, `rust-version = 1.91`-era toolchain; let-chains are used.
- Dependency versions are pinned with `=` in crate manifests — match that style
  when adding deps; prefer versions published >7 days ago.
- Errors are explicit: libraries return typed errors (`models::AppError` maps to
  HTTP statuses); no silent `unwrap()` on I/O or external input.
- `blazers-core` must not depend on `blazers-server`; `blazers-server` owns all
  axum handlers; `blazers-mcp` may depend on both. Keep the dependency graph
  acyclic.
- New HTTP features: add the module under `crates/blazers-server/src/`, declare
  it `pub mod` in `lib.rs`, register routes in `app.rs`.
- Unit tests live in per-module `tests/` directories next to the code
  (`src/db/tests/`, `src/app/tests/`, …); the `crates/blazers/tests/api/`
  suite builds the real binary end-to-end. Keep files ≲200 lines.
- Env vars: `PORT`, `BLAZERS_DATA`, `JWT_SECRET`, `BLAZERS_SECRETS_KEY`.
  Rebrand-safe: never reintroduce `BACKBONE_*` names.
- Do not commit secrets, `data.db`, or `target/`.
