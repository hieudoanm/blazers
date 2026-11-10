# Notifications

Notifications are user-facing messages stored in SQLite and pushed to
subscribers in real time — the equivalent of Supabase Realtime broadcast
channels or PocketBase's optional notification hooks.

## Model

```json
{
  "id": "...",
  "title": "Deploy finished",
  "body": "Production deploy completed successfully.",
  "type": "success",
  "is_read": false,
  "created_at": "..."
}
```

`type` is a free-form string; the admin dashboard color-codes
`info` / `success` / `warning` / `error`.

## REST

| Method   | Endpoint                  | Notes                                  |
| -------- | ------------------------- | -------------------------------------- |
| `GET`    | `/api/notifications`      | list all (unread first not guaranteed) |
| `POST`   | `/api/notifications`      | create; body `{title, body, type}`     |
| `GET`    | `/api/notifications/{id}` | fetch one                              |
| `PATCH`  | `/api/notifications/{id}` | mark as read (no body needed)          |
| `DELETE` | `/api/notifications/{id}` | delete one                             |
| `DELETE` | `/api/notifications`      | clear all                              |

## Realtime stream

```sh
curl -N http://localhost:8080/api/notifications/stream
# data: {"id":"...","title":"...","type":"success",...}
```

Every `POST /api/notifications` also:

1. broadcasts the entry on the SSE stream (`notification::broadcast_notification`), and
2. fires `notification.create` webhooks.

The admin dashboard uses this stream to bump the sidebar badge live.

## Typical uses

- In-app alert center for your own app's users
- Ops alerts written by cron jobs or webhooks
- Audit feed (pair with `type` values like `audit`)
