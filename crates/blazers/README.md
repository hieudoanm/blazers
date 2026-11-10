# blazers

The Blazers binary — a thin argument-parsing entrypoint over the library
crates.

- `src/main.rs` — `blazers` / `blazers serve` → `blazers_server::app::serve_http()`;
  `blazers mcp serve` → `blazers_mcp::run(stderr)`; `--help` / `--version`.
- `tests/api.rs` — end-to-end suite: builds the real binary
  (`CARGO_BIN_EXE_blazers`), boots it on a random port with a temp
  `BLAZERS_DATA` dir, and exercises health/auth/CRUD over HTTP.

Keep `main.rs` thin — all logic lives in `blazers-server` / `blazers-mcp`.
