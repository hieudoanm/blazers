# Authentication

Blazers uses email + password auth with JWT bearer tokens. There are no OAuth
providers or MFA — the model is deliberately minimal, like PocketBase's `users`
auth collection.

## Users

A **user** is a row in the `users` table:

```json
{
  "id": "usr_...",
  "email": "admin@example.com",
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z"
}
```

Passwords are hashed with bcrypt. They are never returned by any endpoint.

## Register

```http
POST /api/auth/register
Content-Type: application/json

{"email": "admin@example.com", "password": "s3cret"}
```

Returns `200` with the `UserResponse` — registration does **not** issue a token;
call `/api/auth/login` next.

## Login

```http
POST /api/auth/login
Content-Type: application/json

{"email": "admin@example.com", "password": "s3cret"}
```

Returns `{ "user": {...}, "token": "eyJ..." }` — the only endpoint that issues
tokens.

## Using the token

Send the token on every protected request:

```http
Authorization: Bearer eyJhbGciOi...
```

Tokens are HS256 JWTs signed with `JWT_SECRET` (default
`dev-secret-change-in-production` — override it in production). See
`extract_claims` in `crates/blazers-server/src/auth.rs` for the exact claim
shape (`sub` = user id).

## Error cases

| Situation               | Response                  |
| ----------------------- | ------------------------- |
| Missing `Authorization` | `401`                     |
| Malformed/expired token | `401`                     |
| Wrong password          | `401`                     |
| Duplicate email         | `400`/`409` (see handler) |

## Sessions

There is no session table and no refresh-token rotation: a token is valid until
it expires or `JWT_SECRET` changes. For long-lived services, store the token and
re-login on `401`.

## User profiles

`GET/PATCH /api/auth/me` exposes the user record: `id`, `email`, `verified`,
custom `data` fields (free-form JSON — Blazers' lightweight take on PocketBase
auth-record fields), and timestamps.

## Password reset & email verification

Both flows are built in:

- `POST /api/auth/forgot-password` → emails a 1-hour reset link
  (`{app_url}/reset-password?token=…`); `POST /api/auth/reset-password`
  exchanges it.
- `POST /api/auth/verify-email` → emails a 24-hour verification link;
  `POST /api/auth/verify-email/confirm` marks `verified: true`.

Delivery goes through SMTP configured in `/api/settings`
(`smtp_host`/`smtp_port`/`smtp_user`/`smtp_from`/`smtp_pass` — the password is
AES-GCM encrypted at rest). When SMTP is not configured the endpoints still
succeed and return a `dev_token` in the response so flows remain testable in
dev.
