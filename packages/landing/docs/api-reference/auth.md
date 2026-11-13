# Auth API

Concepts: [Authentication](../concepts/authentication.md)

Base: `/api/auth` — **no token required**.

## `POST /api/auth/register`

Create a user. Does **not** return a token — log in afterwards.

**Body**

```json
{ "email": "user@example.com", "password": "secret" }
```

**Response `200`** — `UserResponse`:

```json
{
  "id": "...",
  "email": "user@example.com",
  "created_at": "...",
  "updated_at": "..."
}
```

## `POST /api/auth/login`

**Body** — same shape as register.

**Response `200`**

```json
{
  "user": {
    "id": "...",
    "email": "...",
    "created_at": "...",
    "updated_at": "..."
  },
  "token": "eyJhbGciOi..."
}
```

**Errors**: `400` missing fields, `401` bad credentials, `409`-style duplicate
email on register.

## Authenticated user endpoints (token required)

### `GET /api/auth/me`

Current user profile: `id`, `email`, `data` (custom fields), `verified`,
timestamps.

### `PATCH /api/auth/me`

Update custom `data` fields and/or password.

**Body**

```json
{ "data": { "name": "Jane", "role": "dev" },
  "password": "newpass123", "current_password": "oldpass" }
```

`current_password` is required when changing `password`.

### `POST /api/auth/verify-email`

Sends a verification email. Response includes `dev_token` when SMTP is not
configured.

### `POST /api/auth/verify-email/confirm`

**Body** `{ "token": "..." }` — marks the user verified. `400` on bad/expired
token.

## Password reset (no token required)

### `POST /api/auth/forgot-password`

**Body** `{ "email": "user@example.com" }` — always `200`. Sends a reset link
(`{app_url}/reset-password?token=...`, 1h expiry). `dev_token` is included in
the response when SMTP is not configured.

### `POST /api/auth/reset-password`

**Body** `{ "token": "...", "password": "newpass123" }` — `400` on
bad/expired token or short password.

## Managed `users` collection

Users are manageable like a PocketBase auth collection:

- `GET /api/auth/users` — paginated list (`page`, `per_page`, `filter`,
  `sort`). Admins see everyone; other callers see only themselves unless a
  `list_rule` allows more.
- `GET /api/auth/users/{id}` — one user; self/admin always, others need a
  matching `view_rule`.
- `GET/PATCH /api/auth/users/rules` — admin-only; the five rule expressions
  (`list_rule`, `view_rule`, `create_rule`, `update_rule`, `delete_rule`)
  stored under the `users_rules` setting. Empty `list_rule`/`view_rule` =
  self+admins only.

`GET /api/collections` also lists a synthetic `users` entry with
`"type": "auth"`.

### `POST /api/auth/users/{id}/impersonate`

Admin-only. Returns a `LoginResponse` (`{user, token}`) with a user-scoped
JWT for the target account — PocketBase `impersonate` parity. `403` for
non-admins, `404` for unknown users.

## Passwordless OTP

### `POST /api/auth/request-otp`

**Body** `{ "email": "user@example.com" }` — emails a 6-digit code (15 min
expiry). Always `200` (no account enumeration); `dev_otp` is included when
SMTP is not configured.

### `POST /api/auth/auth-with-otp`

**Body** `{ "email": "...", "otp": "123456" }` — exchanges the code for a
JWT (`LoginResponse`). Codes are single-use; `401` on bad/expired codes.

## OAuth2

Providers: `google`, `github`, `gitlab`, `discord`. Configure per provider
via settings (`oauth_{provider}_client_id`, `oauth_{provider}_client_secret`)
or env vars (`OAUTH_{PROVIDER}_CLIENT_ID`, `OAUTH_{PROVIDER}_CLIENT_SECRET`).
`app_url` must be the public base URL — providers redirect back to
`{app_url}/api/auth/oauth/{provider}/callback`.

- `GET /api/auth/oauth/{provider}` — `302` to the provider authorize page
  (`400` when unconfigured, `404` unknown provider).
- `GET /api/auth/oauth/{provider}/callback?code=&state=` — verifies the
  signed state, exchanges the code, finds-or-creates the user by email
  (OAuth-only accounts have no password; the provider subject is stored in
  `data.oauth_{provider}`) and returns a `LoginResponse` with a Blazers JWT.

## Token refresh & MFA

### `POST /api/auth/refresh`

Send the current `Authorization: Bearer` token — returns a fresh 72h JWT
(`LoginResponse`). Sliding-session renewal; `401` without a valid token.

### TOTP MFA

- `POST /api/auth/mfa/enable` (auth) → `{secret, otpauth_url}` — enroll a
  TOTP secret (MFA stays off until confirmed).
- `POST /api/auth/mfa/confirm` `{code}` — verifies a code, enables MFA.
- `POST /api/auth/mfa/disable` `{code}` — disables MFA.
- When MFA is on, `POST /api/auth/login` returns `{"mfa_required": true}`
  instead of a token — finish with `POST /api/auth/mfa/login`
  `{email, password, code}` → `LoginResponse`.

## Token usage

```http
Authorization: Bearer <token>
```

There are no refresh or logout endpoints. See
[Authentication](../concepts/authentication.md) for details.
