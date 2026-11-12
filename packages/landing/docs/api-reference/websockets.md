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
