# MCP server

`blazers mcp serve` runs a
[Model Context Protocol](https://modelcontextprotocol.io/) stdio server that
exposes your Blazers database as agent tools — the same data the HTTP API
serves, without HTTP credentials.

## Run it

```sh
blazers mcp serve
```

It reads the same `BLAZERS_DATA` directory as `blazers serve`, so it can run
alongside a live server (SQLite WAL handles concurrent readers).

## Tools

| Tool                         | Does                                           |
| ---------------------------- | ---------------------------------------------- |
| `blazers_health`             | liveness / status probe                        |
| `blazers_collections_list`   | list collections + schemas                     |
| `blazers_collections_create` | create a collection                            |
| `blazers_collections_delete` | delete a collection                            |
| `blazers_records_list`       | paginated record listing                       |
| `blazers_records_get`        | fetch one record                               |
| `blazers_records_create`     | create a record                                |
| `blazers_records_update`     | update a record                                |
| `blazers_records_delete`     | delete a record                                |
| `blazers_export`             | dump collections/records/buckets/files as JSON |
| `blazers_import`             | import an export payload                       |

Tool names are prefixed `blazers_` (rebranded from the original `backbone_*`).

## Wiring into an MCP client

Example `mcp.json` / client config:

```json
{
  "mcpServers": {
    "blazers": {
      "command": "/path/to/blazers",
      "args": ["mcp", "serve"],
      "env": { "BLAZERS_DATA": "/path/to/data" }
    }
  }
}
```

## Remote KV mode

`blazers mcp serve --kv HOST:PORT` proxies a remote
[`blazers kv`](../api-reference/cache.md#kv-tcp-protocol) listener instead of
opening the local database — the same pattern as kevin's `mcp serve --addr`. The
tool surface becomes `blazers_kv_*` (ping, set, get, del, exists, keys, len,
ttl, expire) and each call speaks the inline TCP protocol, so the MCP server
needs no database of its own. Set `BLAZERS_KV_PASSWORD` when the remote listener
requires `AUTH`.

```json
{
  "mcpServers": {
    "blazers-kv": {
      "command": "/path/to/blazers",
      "args": ["mcp", "serve", "--kv", "cache.internal:6379"]
    }
  }
}
```

## Design notes

- Stdio transport only — no network listener, no auth handshake needed. `--kv`
  mode is the exception: it is a _client_ of the remote KV port.
- The MCP crate (`crates/blazers-mcp`) depends on `blazers-core` and
  `blazers-server` for shared models and handlers; tool implementations live in
  `crates/blazers-mcp/src/tools.rs`.
- Because it shares the database file, MCP writes are immediately visible over
  HTTP (and vice versa) — no sync layer needed.
