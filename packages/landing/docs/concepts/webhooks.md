# Webhooks

Webhooks deliver HTTP callbacks to your own endpoints whenever Blazers data
changes — the same idea as PocketBase hooks, but outbound-only and configurable
per event.

## Events

A webhook subscribes to a list of event names. Valid events
(`webhook::validate_events`):

```text
record.create      record.update      record.delete
record.before_create   record.before_update   record.before_delete
collection.create  collection.delete
bucket.create      bucket.delete
secret.create      secret.update      secret.delete
notification.create
log.create
pubsub.topic.create   pubsub.topic.delete   pubsub.message.create
cronjob.create    cronjob.update    cronjob.delete
```

`*` is accepted as a wildcard for "all events".

### Record hooks (`record.before_*`)

The three `record.before_*` events are **synchronous request hooks** — a
JSVM-lite escape hatch. Before a record write, Blazers awaits each matching
webhook (5s timeout) with
`{"event", "data": {"collection", "record", "auth"}}`. The response can:

- return `{"record": {...}}` — fields are shallow-merged into the record's
  `data` before validation and insert/update;
- return `{"cancel": true}` or a non-2xx status — aborts the request with
  `400` (for `before_delete`, nothing is deleted);
- return anything else — the write proceeds unchanged.

Hook deliveries are also written to the webhook log like normal events.

## Delivery

When an event fires, Blazers POSTs a JSON payload to each active webhook whose
`events` list matches:

```http
POST https://your.app/hook
Content-Type: application/json
X-Webhook-ID: <webhook id>
X-Webhook-Event: record.create
X-Webhook-Signature-256: <hex hmac-sha256>   # only if webhook.secret set
```

The signature is `HMAC-SHA256(secret, body)` hex-encoded — verify it to prove
the request came from Blazers.

Payload keys vary by event:

| Event prefix       | Body                                        |
| ------------------ | ------------------------------------------- |
| `record.*`         | `{"collection": "<name>", "record": {...}}` |
| `collection.*`     | `{"collection": {...}}`                     |
| `bucket.*`         | `{"bucket": {...}}`                         |
| `secret.*`         | `{"secret": {...}}`                         |
| `cronjob.*`        | `{"cronjob": {...}}`                        |
| `pubsub.topic.*`   | `{"topic": {...}}`                          |
| `pubsub.message.*` | `{"message": {...}}`                        |
| `notification.*`   | the notification object itself              |
| `log.*`            | the log entry object itself                 |

> `secret.*` payloads contain the **stored (encrypted)** value, not plaintext —
> secrets are never decrypted for webhook delivery.

## Delivery logs

Every attempt is recorded in `webhook_logs`:

```json
{
  "id": "...",
  "webhook_id": "...",
  "event": "record.create",
  "url": "https://your.app/hook",
  "request_body": "{...}",
  "response_status": 200,
  "response_body": "ok",
  "error": "",
  "status": "success",
  "created_at": "..."
}
```

`status` is `success` or `failure`; `error` holds the transport error when the
request never completed. Inspect deliveries with `GET /api/webhooks/{id}/logs`
or the admin dashboard.

## Managing webhooks

```sh
curl -X POST http://localhost:8080/api/webhooks \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{
    "name": "sync-to-crm",
    "url": "https://your.app/hook",
    "events": ["record.create", "record.update"],
    "secret": "whsec_..."
  }'
```

- `PATCH /api/webhooks/{id}` can toggle `is_active`, change URL/events/secret.
- `DELETE` removes the webhook (logs are kept).
- Delivery is fire-and-forget from the request path — a slow endpoint adds
  latency to the triggering API call; keep handlers fast.
