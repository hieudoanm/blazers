# AGENTS.md — blazers-core

Storage crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/db/`: schema + migrations + all SQL helpers, one file per domain
  (`collections.rs`, `records.rs`, `cache.rs`, …). `data_dir()` resolves
  `$BLAZERS_DATA` else `~/.blazers`.
- `src/models/`: wire types + `AppError` (`IntoResponse` → JSON + status),
  one file per domain.
- No HTTP handlers, no feature logic — features live in `blazers-server`.
- Must stay dependency-light: only `blazers-*`-free crates (axum is used solely
  for `AppError`'s `IntoResponse`).
- Tests live in `src/db/tests/` and `src/models/tests/`; run
  `cargo test -p blazers-core`.
