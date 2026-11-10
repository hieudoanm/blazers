# Secrets

The secrets vault stores sensitive values encrypted at rest — API keys,
webhook secrets, third-party credentials — with the same ergonomics as
PocketBase's optional secrets table, but actually encrypted.

## Encryption

- Algorithm: **AES-256-GCM** (`aes-gcm` crate).
- Key: generated on first run by `secrets::get_or_create_secrets_key` and
  stored inside `BLAZERS_DATA`. Losing that file means losing the secrets.
- Only `GET /api/secrets/{id}` returns the **decrypted** value;
  `GET /api/secrets` (list) returns metadata only.

## Model

```json
{
  "id": "...",
  "name": "stripe-secret",
  "value": "sk_live_...", // decrypted on single-get only
  "scope": "payments",
  "created_at": "...",
  "updated_at": "..."
}
```

`scope` is a free-form namespace (`payments`, `email`, `internal`) so you
can group and later filter secrets by purpose.

## REST

| Method   | Endpoint            | Notes                          |
| -------- | ------------------- | ------------------------------ |
| `GET`    | `/api/secrets`      | list metadata (values masked)  |
| `POST`   | `/api/secrets`      | create `{name, value, scope?}` |
| `GET`    | `/api/secrets/{id}` | fetch **with decrypted value** |
| `PATCH`  | `/api/secrets/{id}` | update name/value/scope        |
| `DELETE` | `/api/secrets/{id}` | delete                         |

## Webhooks

`secret.create` / `secret.update` / `secret.delete` events fire webhooks,
but the payload contains the **stored ciphertext**, never plaintext.

## Using secrets in your app

There is no "inject into outgoing request" feature — your app fetches a
secret via `GET /api/secrets/{id}` when it needs it, or you read it once at
startup and cache it in [the cache](cache.md). For cron jobs, put the
secret in the job's `headers` JSON if the target needs it.
