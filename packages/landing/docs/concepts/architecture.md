# Architecture

Blazers is a single Rust binary. One process serves the REST API, the admin
dashboard, WebSockets, and three server-sent-event streams; a second mode
(`blazers mcp serve`) exposes the same data over the Model Context Protocol.

## System overview

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 720 380" font-family="ui-sans-serif, system-ui, sans-serif">
  <defs>
    <marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M0,0 L10,5 L0,10 z" fill="#6b7280"/>
    </marker>
  </defs>
  <rect x="20" y="30" width="150" height="60" rx="10" fill="#111827" stroke="#374151"/>
  <text x="95" y="55" text-anchor="middle" fill="#f3f4f6" font-size="13" font-weight="600">Clients</text>
  <text x="95" y="72" text-anchor="middle" fill="#9ca3af" font-size="11">curl · SDK · browser</text>

  <rect x="230" y="20" width="260" height="240" rx="12" fill="#1e1b4b" stroke="#6366f1" stroke-width="1.5"/>
  <text x="360" y="42" text-anchor="middle" fill="#a5b4fc" font-size="13" font-weight="700">blazers serve</text>

  <rect x="250" y="58" width="100" height="34" rx="7" fill="#111827" stroke="#374151"/>
  <text x="300" y="79" text-anchor="middle" fill="#d1d5db" font-size="11">REST API</text>
  <rect x="370" y="58" width="100" height="34" rx="7" fill="#111827" stroke="#374151"/>
  <text x="420" y="79" text-anchor="middle" fill="#d1d5db" font-size="11">/ws upgrade</text>
  <rect x="250" y="102" width="100" height="34" rx="7" fill="#111827" stroke="#374151"/>
  <text x="300" y="123" text-anchor="middle" fill="#d1d5db" font-size="11">SSE streams</text>
  <rect x="370" y="102" width="100" height="34" rx="7" fill="#111827" stroke="#374151"/>
  <text x="420" y="123" text-anchor="middle" fill="#d1d5db" font-size="11">Admin UI</text>
  <rect x="250" y="146" width="220" height="34" rx="7" fill="#111827" stroke="#374151"/>
  <text x="360" y="167" text-anchor="middle" fill="#d1d5db" font-size="11">Rate limit · auth · RBAC middleware</text>
  <rect x="250" y="196" width="220" height="48" rx="7" fill="#0b1120" stroke="#6366f1" stroke-dasharray="4 3"/>
  <text x="360" y="216" text-anchor="middle" fill="#a5b4fc" font-size="11" font-weight="600">Feature modules</text>
  <text x="360" y="234" text-anchor="middle" fill="#9ca3af" font-size="10">collections · files · webhooks · cache · cron · secrets</text>

  <rect x="560" y="80" width="140" height="60" rx="10" fill="#111827" stroke="#374151"/>
  <text x="630" y="105" text-anchor="middle" fill="#f3f4f6" font-size="12" font-weight="600">SQLite</text>
  <text x="630" y="122" text-anchor="middle" fill="#9ca3af" font-size="10">data.db (WAL)</text>

  <rect x="560" y="180" width="140" height="60" rx="10" fill="#111827" stroke="#374151"/>
  <text x="630" y="205" text-anchor="middle" fill="#f3f4f6" font-size="12" font-weight="600">Filesystem</text>
  <text x="630" y="222" text-anchor="middle" fill="#9ca3af" font-size="10">uploads + secrets key</text>

  <rect x="230" y="300" width="260" height="60" rx="10" fill="#052e16" stroke="#34d399" stroke-width="1.5"/>
  <text x="360" y="324" text-anchor="middle" fill="#34d399" font-size="12" font-weight="600">External services</text>
  <text x="360" y="342" text-anchor="middle" fill="#9ca3af" font-size="10">webhook endpoints · cron HTTP calls · MCP clients</text>

  <line x1="170" y1="60" x2="230" y2="140" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr)"/>
  <line x1="490" y1="140" x2="560" y2="110" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr)"/>
  <line x1="490" y1="180" x2="560" y2="210" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr)"/>
  <line x1="360" y1="260" x2="360" y2="300" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr)"/>
</svg>

## Crate layout

```text
crates/
  blazers-core/    # SQLite schema, migrations, models, wire types
  blazers-server/  # HTTP API, realtime, admin dashboard, all features
  blazers-mcp/     # MCP stdio server (blazers_* tools)
  blazers/         # thin binary: `blazers serve` / `blazers kv` / `blazers mcp serve`
```

`blazers-core` owns persistence and knows nothing about HTTP.
`blazers-server` owns every feature and embeds the dashboard assets
(`public/`) into the binary via `rust-embed`. `blazers-mcp` talks to the
same SQLite database directly, so MCP tools and the HTTP API always agree.

## Request lifecycle

Every `/api/*` request passes through the same middleware stack:

1. **Request logging** — method, path, status, duration.
2. **CORS** — permissive, so browser apps on any origin can call the API.
3. **Rate limiting** — per-IP token bucket.
4. **Content-Type check** — mutating requests must send `application/json`
   (multipart uploads are exempt).
5. **Body limit** — 10 MiB maximum request body.
6. **Handler** — JWT auth (`Authorization: Bearer <token>`), then RBAC
   role checks against the target collection/bucket.

## Persistence model

Everything lives in one SQLite file inside `BLAZERS_DATA`:

- `collections` + dynamically-created record tables — schemaless-ish
  records validated against the collection's JSON schema.
- `buckets` + `files` — metadata in SQLite, bytes on the filesystem under
  `BLAZERS_DATA/files/`.
- `users`, `permissions` — auth and RBAC.
- `webhooks`, `webhook_logs`, `secrets`, `cache`, `notifications`,
  `logs`, `cron_jobs`, `cron_job_logs`, `ws_connections`,
  `ws_messages`, `pubsub_topics`, `pubsub_messages`.

Because it is one file, [backup](api-reference/admin.md) is a file copy and
[export/import](api-reference/admin.md) is a JSON document.

## Realtime fan-out

Three independent channels share the same in-process hub pattern:

- **WebSocket** (`/ws`) — persistent bidirectional connections; the server
  can push to one client or broadcast to all.
- **SSE** (`/api/notifications/stream`, `/api/logs/stream`,
  `/api/pubsub/{name}/stream`) — one-way server→client streams over plain
  HTTP, ideal for dashboards and browsers that cannot hold WebSockets.

## MCP mode

`blazers mcp serve` starts a stdio JSON-RPC server exposing tools such as
`blazers_list_collections`, `blazers_list_records`, `blazers_export`, and
`blazers_import`. It reads the same `BLAZERS_DATA` database, so an agent can
inspect and modify a running Blazers backend without HTTP credentials.
`blazers mcp serve --kv HOST:PORT` instead proxies a remote `blazers kv`
listener and exposes `blazers_kv_*` tools over the inline TCP protocol.
