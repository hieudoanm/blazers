# Secrets API

Concepts: [Secrets](../concepts/secrets.md)

Base: `/api/secrets` — admin-level RBAC.

## `GET /api/secrets` — list

Metadata only — returns `SecretListItem` (`id`, `name`, `scope`,
`created_at`, `updated_at`); `value` is never included.

## `POST /api/secrets` — create

**Body**

```json
{ "name": "stripe-key", "value": "sk_live_...", "scope": "payments" }
```

Value is AES-256-GCM encrypted before storage. `scope` optional —
defaults to `"general"`. `name` required (`400` if empty).

**Response `201`** — the `Secret`. Fires `secret.create` webhook
(payload contains ciphertext, not plaintext).

## `GET /api/secrets/{id}` — reveal

Returns the `Secret` **with the decrypted `value`**. This is the only
endpoint that exposes plaintext.

## `PATCH /api/secrets/{id}` — update

**Body** — any subset of `{name, value, scope}`. A new `value` is
re-encrypted; omitting `value` keeps the existing secret.

## `DELETE /api/secrets/{id}`

`204`. Fires `secret.delete`.
