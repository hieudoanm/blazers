# WebSockets API

Concepts: [Realtime](../concepts/realtime.md)

The socket itself is **not** under `/api` — it upgrades at `GET /ws` and does
not require a JWT (it is exempt from the API middleware stack).
Connection/message audit endpoints below are REST.

## `GET /ws` — connect

Standard WebSocket upgrade. No auth handshake — treat the socket as public and
do your own auth inside the first message if needed.

```js
const ws = new WebSocket('ws://localhost:8080/ws');
```

The server records the connection in `ws_connections` and every inbound text
frame in `ws_messages` (`direction: "received"`). Outbound sends via the REST
endpoints are logged as `direction: "sent"`.

## Subscriptions & record events

Send a JSON text frame to subscribe:

```json
{ "action": "subscribe", "collection": "posts" }
{ "action": "subscribe", "collection": "*" }
{ "action": "subscribe", "collection": "posts", "filter": "views > 50" }
{ "action": "unsubscribe", "collection": "posts" }
```

`"*"` subscribes to every collection. An optional `filter` applies a
PocketBase-style expression to each record event — the event is only
delivered when it matches (e.g. `"filter": "status = 'published'"`).
`@request.auth.*` resolves to null since WS connections are anonymous. Whenever a record is created, updated or
deleted through the REST API, subscribers receive:

```json
{ "type": "record", "action": "create",
  "collection": "posts", "record": { "id": "...", "data": {...} } }
```

`action` is one of `create`, `update`, `delete`. `update` and `delete` events
also carry `oldRecord` — the record as it was before the change.

## Presence

Channels let clients announce availability (chat "typing…", live cursors,
who's online):

```json
{ "action": "presence_join", "channel": "room-1" }
{ "action": "presence", "channel": "room-1", "state": { "cursor": {"x": 10} } }
{ "action": "presence_leave", "channel": "room-1" }
```

`presence` also auto-joins. Members receive
`{ "type": "presence", "action": "join"|"state"|"leave", "channel": "...",
"client": "<id>", "state": {...} }`; disconnects broadcast `leave`
automatically.

## `GET /api/websockets` — list connections

Active and recently-closed connections:

```json
[
  {
    "id": "...",
    "remote_addr": "...",
    "path": "/ws",
    "user_agent": "...",
    "connected_at": "...",
    "disconnected_at": "",
    "is_active": true,
    "created_at": "..."
  }
]
```

## `GET /api/websockets/{id}`

Fetch one connection. `404` if missing.

## `DELETE /api/websockets/{id}`

Disconnect and remove the connection record. `204`.

## `POST /api/websockets/{id}/send` — send to one

**Body**

```json
{ "content": "hello client" }
```

Pushes a text frame to that connection if still active.

## `POST /api/websockets/broadcast` — send to all

Same body; fans out to every connected client.

## `GET /api/websockets/{id}/messages`

Message history for one connection.

## `GET /api/websockets/messages`

Global message history across all connections (for the admin dashboard's message
viewer).
