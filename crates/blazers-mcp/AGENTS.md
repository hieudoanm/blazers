# AGENTS.md — blazers-mcp

MCP stdio server crate. See the root [`AGENTS.md`](../../AGENTS.md).

- Entry point: `blazers_mcp::run(&mut dyn Write)` — called by the `blazers`
  binary for `blazers mcp serve`.
- Framing: one JSON-RPC 2.0 message per line on stdin/stdout; nothing but
  frames on stdout.
- Depends on `blazers-core` (db/models) and `blazers-server` (only for
  `import_export::ExportData`). Do not add HTTP handler deps here.
- Tool names are `blazers_*`; keep them stable — MCP clients cache schemas.
- Tests: `cargo test -p blazers-mcp`.
