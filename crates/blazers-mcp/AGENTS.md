# AGENTS.md — blazers-mcp

MCP stdio server crate. See the root [`AGENTS.md`](../../AGENTS.md).

- Entry point: `blazers_mcp::run(&mut dyn Write)` — called by the `blazers`
  binary for `blazers mcp serve`. `run_remote(addr, ...)` serves `blazers_kv_*`
  tools proxying a remote `blazers kv` listener (`--kv HOST:PORT`).
- Framing: one JSON-RPC 2.0 message per line on stdin/stdout; nothing but
  frames on stdout.
- Depends on `blazers-core` only (db/models, including `ExportData` and
  `db::export_all`/`db::import_all`). Do not add HTTP handler deps here.
- Tool names are `blazers_*`; keep them stable — MCP clients cache schemas.
- Layout: `handlers/` (db/io/kv tool handlers), `tools/` (tool
  registration), `protocol/` (wire types + JSON schema), `server.rs` +
  `server/dispatch.rs` (request dispatch). Tests in `src/tests/`.
- `cargo test -p blazers-mcp`.
