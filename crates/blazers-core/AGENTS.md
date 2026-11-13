# AGENTS.md — blazers-core

Storage crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/db/`: schema + migrations (`migrate.rs`, `core.rs`) + all SQL helpers,
  one file per domain (`collections.rs`, `records.rs` + `records_query.rs`
  for filter/sort listing, `cache.rs`, `settings.rs`, …). `data_dir()`
  resolves `$BLAZERS_DATA` else `~/.blazers`.
- `src/filter/`: PocketBase-style filter expression engine (tokenizer,
  parser, evaluator, sort keys) shared by `?filter=` and collection record
  rules.
- `src/models/`: wire types + `AppError` (`IntoResponse` → JSON + status),
  one file per domain.
- No HTTP handlers, no feature logic — features live in `blazers-server`.
- Must stay dependency-light: only `blazers-*`-free crates (axum is used solely
  for `AppError`'s `IntoResponse`).
- Tests live in `src/db/tests/` and `src/models/tests/`; run
  `cargo test -p blazers-core`.
