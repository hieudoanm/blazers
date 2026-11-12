# Permissions API

Concepts: [Permissions (RBAC)](../concepts/permissions.md)

Base: `/api/permissions` — admin-level RBAC (managing grants requires admin).

## `GET /api/permissions`

List all grants:

```json
[
  {
    "id": "...",
    "user_id": "usr_...",
    "collection": "posts",
    "role": "editor",
    "created_at": "...",
    "updated_at": "..."
  }
]
```

## `POST /api/permissions` — grant

**Body**

```json
{ "user_id": "usr_...", "collection": "posts", "role": "editor" }
```

All three fields required (`400` if empty). `role` must be one of `admin`,
`editor`, `viewer` (`400` otherwise). `collection` may be `*` for a global
grant.

**Response `201`** — the `Permission`.

## `DELETE /api/permissions/{id}`

Revoke a grant. `204`; `404` if missing.
