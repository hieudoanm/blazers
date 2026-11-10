# AGENTS.md — blazers (binary)

Binary crate for Blazers. See the root [`AGENTS.md`](../../AGENTS.md).

- `src/main.rs` only parses args and delegates: HTTP →
  `blazers_server::app::serve_http()`, MCP → `blazers_mcp::run()`.
- `tests/api.rs` is the authoritative end-to-end gate — it spawns the compiled
  binary, so run `cargo test -p blazers` after changing routes or startup.
- The binary name is `blazers`; env vars are `BLAZERS_*` (never `BACKBONE_*`).
