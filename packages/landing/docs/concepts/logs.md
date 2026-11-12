# Logs

The log store is a structured, queryable event log — think PocketBase's `_logs`
table plus aRealtime tail, minus the request-log noise (HTTP access logs go to
stdout, not this table).

## Model

```json
{
  "id": "...",
  "level": "info",
  "message": "cron job finished",
  "meta": "{\"job\":\"cleanup\"}",
  "created_at": "..."
}
```

`level` is one of `debug`, `info`, `warn`, `error` (defaults to `info` when
empty). `meta` is an optional JSON string for structured context.

## REST

| Method   | Endpoint    | Notes                       |
| -------- | ----------- | --------------------------- |
| `GET`    | `/api/logs` | list entries (newest first) |
| `POST`   | `/api/logs` | append; `message` required  |
| `DELETE` | `/api/logs` | clear all                   |

## Realtime stream

```sh
curl -N http://localhost:8080/api/logs/stream
```

Every `POST /api/logs` broadcasts on the SSE stream and fires `log.create`
webhooks — so external systems can tail your app's log stream.

## What writes here

Anything that calls the API — your app, cron jobs reporting results, or the
admin dashboard's "Add log" action. Blazers itself writes HTTP access logs to
stdout only; it does **not** mirror them into this table.
