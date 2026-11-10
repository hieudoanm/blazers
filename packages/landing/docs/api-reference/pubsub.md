# Pub/Sub API

Concepts: [Realtime](../concepts/realtime.md)

Base: `/api/pubsub` — REST routes require a JWT (handlers call
`extract_claims`); the stream route is unauthenticated.

## Topics

### `GET /api/pubsub/topics`

List topics: `[{id, name, created_at}]`.

### `POST /api/pubsub/topics`

**Body** `{"name": "orders"}` → `201` with the `PubSubTopic`.
Fires `pubsub.topic.create`.

### `GET /api/pubsub/topics/{name}`

Fetch one topic by name. `404` if missing.

### `DELETE /api/pubsub/topics/{name}`

`204`. Fires `pubsub.topic.delete`.

## Messages

### `GET /api/pubsub/topics/{name}/messages`

Message history for a topic (array of `PubSubMessage`, newest first):

```json
{ "id": "...", "topic_id": "...", "body": "...", "created_at": "..." }
```

### `POST /api/pubsub/topics/{name}/messages`

Publish.

**Body** `{"body": "order #123 shipped"}` — `body` required (`400` if empty).

**Response `201`** — the `PubSubMessage`. Broadcasts to every connected
SSE client and fires `pubsub.message.create`.

## `GET /api/pubsub/{name}/stream` — SSE

Unauthenticated server-sent stream. Every published message (any topic)
arrives as:

```text
data: {"id":"...","topic_id":"...","body":"...","created_at":"..."}
```

> The `{name}` path segment is accepted but **not** used for filtering —
> all pub/sub messages fan out to all connected clients. Filter by
> `topic_id` client-side.
