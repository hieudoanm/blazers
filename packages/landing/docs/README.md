# Blazers documentation

Blazers is a single-binary Backend-as-a-Service: SQLite database, auth, file
storage, realtime, webhooks, cache, notifications, logs, cron, a secrets vault,
an admin dashboard, and an MCP server — all in one Rust executable.

This documentation is organized like the
[PocketBase docs](https://pocketbase.io/docs/):

- **Getting started** — install, configure, and make your first API call.
- **Concepts** — how each subsystem works and when to use it.
- **API reference** — every REST endpoint, request/response shapes, and errors.
- **Guides** — the admin dashboard, MCP server, and deployment.

The machine-readable spec is always available from a running server:

- `GET /api/openapi.json` — OpenAPI 3 document
- `GET /api/docs` — Swagger UI

## Contents

### Getting started

- [Installation](getting-started/installation.md)
- [Quickstart](getting-started/quickstart.md)

### Concepts

- [Architecture](concepts/architecture.md)
- [Authentication](concepts/authentication.md)
- [Collections & records](concepts/collections.md)
- [Buckets & files](concepts/buckets-and-files.md)
- [Realtime](concepts/realtime.md)
- [Webhooks](concepts/webhooks.md)
- [Notifications](concepts/notifications.md)
- [Logs](concepts/logs.md)
- [Cache](concepts/cache.md)
- [Cron jobs](concepts/cron-jobs.md)
- [Secrets](concepts/secrets.md)
- [Permissions (RBAC)](concepts/permissions.md)

### API reference

- [Conventions](api-reference/conventions.md)
- [Auth](api-reference/auth.md)
- [Collections](api-reference/collections.md)
- [Records](api-reference/records.md)
- [Buckets](api-reference/buckets.md)
- [Files](api-reference/files.md)
- [Webhooks](api-reference/webhooks.md)
- [WebSockets](api-reference/websockets.md)
- [Cache](api-reference/cache.md)
- [Notifications](api-reference/notifications.md)
- [Logs](api-reference/logs.md)
- [Pub/Sub](api-reference/pubsub.md)
- [Cron jobs](api-reference/cronjobs.md)
- [Secrets](api-reference/secrets.md)
- [Permissions](api-reference/permissions.md)
- [Admin & ops](api-reference/admin.md)

### Guides

- [Admin dashboard](guides/admin-dashboard.md)
- [MCP server](guides/mcp-server.md)
- [Deployment](guides/deployment.md)
