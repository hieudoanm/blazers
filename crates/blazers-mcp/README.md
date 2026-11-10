# blazers-mcp

Model Context Protocol server for Blazers — exposes the database as MCP tools
over stdio.

## How it works

- Newline-delimited JSON-RPC 2.0 on stdin/stdout; diagnostics on stderr.
- Opens the same SQLite database as the HTTP server (`blazers_core::db`).
- `src/lib.rs` — `run()` entrypoint, `Server`, tool registration.
- `src/protocol.rs` — JSON-RPC + MCP types (no I/O).
- `src/transport.rs` — frame reader/writer.
- `src/handlers.rs` — tool implementations against a `DbHandle`
  (`Arc<Mutex<rusqlite::Connection>>`).
- `src/tools.rs` — tool schemas (`blazers_*` names).
- `src/tests.rs` — protocol + handler tests (`#[cfg(test)]`).

## Tools

`blazers_health`, `blazers_collections_list/create/delete`,
`blazers_records_list/get/create/update/delete`, `blazers_export`,
`blazers_import`.

Run: `blazers mcp serve` (handled by the `blazers` binary).

## Rules

- Handlers must do real DB work — no stubbed responses.
- `per_page` is capped at 200.
- `cargo test -p blazers-mcp`.
