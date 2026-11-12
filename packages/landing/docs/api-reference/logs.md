# Logs API

Concepts: [Logs](../concepts/logs.md)

Base: `/api/logs` — authenticated.

## `GET /api/logs` — list

All entries, newest first (array of `LogEntry`).

## `POST /api/logs` — append

**Body**

```json
{ "level": "warn", "message": "disk 80% full", "meta": "{\"host\":\"web-1\"}" }
```

`message` required (`400` if empty); `level` defaults to `info` when empty;
`meta` is an optional JSON string.

**Response `201`** — the `LogEntry`. Broadcasts on `/api/logs/stream` and fires
`log.create` webhooks.

## `DELETE /api/logs` — clear

Wipe all entries. `200` → `{"status": "cleared"}` (same shape as notifications).

## `GET /api/logs/stream` — SSE

Unauthenticated server-sent stream; each new entry arrives as a `data:` event
with the `LogEntry` JSON. See
[Notifications API](notifications.md#api-notificationsstream--sse) for the same
pattern.
