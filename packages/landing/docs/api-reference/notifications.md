# Notifications API

Concepts: [Notifications](../concepts/notifications.md)

Base: `/api/notifications` — authenticated.

## `GET /api/notifications` — list

All notifications, newest first (array).

## `POST /api/notifications` — create

**Body**

```json
{ "title": "Deploy done", "body": "...", "type": "success" }
```

All fields optional in the request struct; `type` free-form. Broadcasts on
`/api/notifications/stream` and fires `notification.create` webhooks.

**Response `201`** — the `Notification`.

## `GET /api/notifications/{id}`

Fetch one. `404` if missing.

## `PATCH /api/notifications/{id}` — mark read

No body required — sets `is_read = true`. Returns the updated `Notification`.

## `DELETE /api/notifications/{id}`

Delete one. `204`; `404` if missing.

## `DELETE /api/notifications` — clear all

`200` → `{"status": "cleared"}`.

## `GET /api/notifications/stream` — SSE

Server-sent events; each new notification arrives as:

```text
data: {"id":"...","title":"...","type":"success","is_read":false,"created_at":"..."}
```

Keep-alive comments are sent periodically. **No authentication** — the stream
routes live outside the authenticated `/api` middleware stack; put a reverse
proxy in front if the stream must be private.
