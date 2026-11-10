# AGENTS.md — blazers-core

Storage crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/db.rs`: schema + migrations + all SQL helpers. `data_dir()` resolves
  `$BLAZERS_DATA` else `~/.blazers`.
- `src/models.rs`: wire types + `AppError` (`IntoResponse` → JSON + status).
- No HTTP handlers, no feature logic — features live in `blazers-server`.
- Must stay dependency-light: only `blazers-*`-free crates (axum is used solely
  for `AppError`'s `IntoResponse`).
- Tests colocated; run `cargo test -p blazers-core`.
