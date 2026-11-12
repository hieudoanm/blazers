# Realtime

Blazers has three realtime channels, all in-process and all optional — pick per
use case.

## WebSocket — `/ws`

A persistent bidirectional socket. The server tracks every connection in the
`ws_connections` table and every frame in `ws_messages`, so you can audit who
was connected and what was said.

```js
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onmessage = (e) => console.log('got', e.data);
ws.send('hello'); // logged as direction="received"
```

Server-side you can:

- send to one connection — `POST /api/websockets/{id}/send`
- broadcast to all — `POST /api/websockets/broadcast`
- list/audit connections and messages via the REST endpoints

Useful for chat, live cursors, game state — anything needing client→server push.

## Server-Sent Events — three streams

SSE is one-way (server→client) but works over plain HTTP/1.1 and through most
proxies. Blazers exposes:

| Endpoint                    | Stream contents                       |
| --------------------------- | ------------------------------------- |
| `/api/notifications/stream` | new notifications as they are created |
| `/api/logs/stream`          | new log entries                       |
| `/api/pubsub/{name}/stream` | pub/sub messages                      |

```sh
curl -N http://localhost:8080/api/logs/stream
# data: {"id":"...","level":"info","message":"...","created_at":"..."}
```

Each stream keeps the connection alive with SSE `keep-alive` comments and holds
a 1-hour server-side lease.

## Pub/Sub topics

On top of the message log, pub/sub gives you named channels:

1. `POST /api/pubsub/topics` — create a topic (`{"name": "orders"}`)
2. `POST /api/pubsub/topics/orders/messages` — publish (`{"body": "..."}`)
3. Every connected client on `/api/pubsub/*/stream` receives the message as an
   SSE `data:` event containing the `PubSubMessage` JSON
   (`{id, topic_id, body, created_at}`).

Messages are also persisted in `pubsub_messages`, so late joiners can read
history via `GET /api/pubsub/topics/{name}/messages`.

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 240" font-family="ui-sans-serif, system-ui, sans-serif">
  <rect x="230" y="90" width="180" height="60" rx="10" fill="#1e1b4b" stroke="#6366f1" stroke-width="1.5"/>
  <text x="320" y="115" text-anchor="middle" fill="#a5b4fc" font-size="12" font-weight="700">blazers</text>
  <text x="320" y="132" text-anchor="middle" fill="#9ca3af" font-size="10">pubsub hub + SQLite</text>

  <rect x="30" y="30" width="140" height="44" rx="8" fill="#111827" stroke="#374151"/>
  <text x="100" y="56" text-anchor="middle" fill="#d1d5db" font-size="11">Publisher (REST)</text>
  <rect x="30" y="166" width="140" height="44" rx="8" fill="#111827" stroke="#374151"/>
  <text x="100" y="192" text-anchor="middle" fill="#d1d5db" font-size="11">SSE client A</text>
  <rect x="470" y="166" width="140" height="44" rx="8" fill="#111827" stroke="#374151"/>
  <text x="540" y="192" text-anchor="middle" fill="#d1d5db" font-size="11">SSE client B</text>

  <line x1="170" y1="52" x2="230" y2="105" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr2)"/>
  <line x1="410" y1="105" x2="470" y2="180" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr2)"/>
  <line x1="410" y1="135" x2="470" y2="200" stroke="#6b7280" stroke-width="1.5" marker-end="url(#arr2)"/>
  <defs>
    <marker id="arr2" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M0,0 L10,5 L0,10 z" fill="#6b7280"/>
    </marker>
  </defs>
  <text x="320" y="225" text-anchor="middle" fill="#6b7280" font-size="10">POST /api/pubsub/topics/{name}/messages fans out to every stream client</text>
</svg>

> The stream endpoint accepts a topic name in the path
> (`/api/pubsub/{name}/stream`) but currently fans out **all** pub/sub messages
> to every connected client — filter by `topic_id` client-side until per-topic
> streams land.

## Choosing a channel

| Need                           | Use                    |
| ------------------------------ | ---------------------- |
| Browser dashboard live updates | SSE                    |
| Client→server messages         | WebSocket              |
| Fan-out to many subscribers    | Pub/Sub + SSE          |
| Audit trail of messages        | REST history endpoints |
