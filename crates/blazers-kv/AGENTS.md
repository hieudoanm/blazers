# AGENTS.md — blazers-kv

Key-value crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/cache/` — `CacheStore`: in-memory + SQLite-backed cache with typed ops
  (strings, hashes, lists) shared by the REST cache API in `blazers-server`.
- `src/kvproto/` — Redis-style inline-protocol TCP listener (`blazers kv`
  command), sharing the same `CacheStore`.
- Depends on `blazers-core` (db/models) only; no HTTP handlers — the REST
  surface lives in `blazers-server/src/handlers/cache*.rs`.
- Tests live in per-module `tests/` dirs; `cargo test -p blazers-kv`.
