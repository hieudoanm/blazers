# Webhooks API

Concepts: [Webhooks](../concepts/webhooks.md)

Base: `/api/webhooks` — admin-level RBAC.

## `GET /api/webhooks`

List all webhooks (array of `Webhook`).

## `POST /api/webhooks`

**Body**

```json
{
  "name": "sync",
  "url": "https://example.com/hook",
  "events": ["record.create", "*"],
  "secret": "whsec_..."
}
```

`events` must be valid event names or `*` — see
[webhook events](../concepts/webhooks.md#events). `secret` enables
`X-Webhook-Signature-256` HMAC signing.

**Response `201`** — the `Webhook`.

## `GET /api/webhooks/{id}`

Fetch one. `404` if missing.

## `PATCH /api/webhooks/{id}`

**Body** — any subset:

```json
{ "url": "...", "events": [...], "is_active": false, "secret": "..." }
```

## `DELETE /api/webhooks/{id}`

`204`. Delivery logs are retained.

## `GET /api/webhooks/{id}/logs`

List delivery attempts for this webhook (newest first).

**Response `200`** — array of `WebhookLog`:

```json
{
  "id": "...",
  "webhook_id": "...",
  "event": "record.create",
  "url": "...",
  "request_body": "{...}",
  "response_status": 200,
  "response_body": "ok",
  "error": "",
  "status": "success",
  "created_at": "..."
}
```

`status` ∈ `success` | `failure`; `error` carries transport errors.
