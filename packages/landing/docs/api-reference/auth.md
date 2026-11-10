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
{ "id": "...", "email": "user@example.com", "created_at": "...", "updated_at": "..." }
```

## `POST /api/auth/login`

**Body** — same shape as register.

**Response `200`**

```json
{
  "user": { "id": "...", "email": "...", "created_at": "...", "updated_at": "..." },
  "token": "eyJhbGciOi..."
}
```

**Errors**: `400` missing fields, `401` bad credentials, `409`-style
duplicate email on register.

## Token usage

```http
Authorization: Bearer <token>
```

There are no refresh, logout, or password-reset endpoints. See
[Authentication](../concepts/authentication.md) for details.
