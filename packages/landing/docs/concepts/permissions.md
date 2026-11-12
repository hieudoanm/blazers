# Permissions (RBAC)

Blazers has a small role-based access layer over collections — simpler than
PocketBase's per-collection API rules, closer to Supabase RLS in spirit but
without row-level policies.

## Model

```json
{
  "id": "...",
  "user_id": "usr_...",
  "collection": "posts",
  "role": "editor",
  "created_at": "...",
  "updated_at": "..."
}
```

`collection` may be a specific collection name **or `*`** for a global grant.

## Roles

| Role     | Meaning                                       |
| -------- | --------------------------------------------- |
| `admin`  | satisfies **any** required role on that scope |
| `editor` | write-level access (exact-match checks only)  |
| `viewer` | read-level access (exact-match checks only)   |

> **No role hierarchy.** `require_role` passes when the stored role is `admin`
> **or exactly equals** the required role. An `editor` grant does **not** imply
> `viewer` — grant both if a user needs read+write on a collection, or use
> `admin`.

## Enforcement

`rbac::require_role(conn, user_id, collection, required)` runs inside handlers
before touching data:

- record/bucket reads require `viewer`
- record writes, webhook/cronjob/secret management, backup, and permission
  management require `admin` (or `editor` where noted)

Scope resolution order: a permission for the exact collection name wins; then
`collection = '*'` global permissions are consulted; `admin` anywhere in scope
satisfies everything.

## REST

| Method   | Endpoint                | Notes                               |
| -------- | ----------------------- | ----------------------------------- |
| `GET`    | `/api/permissions`      | list all grants                     |
| `POST`   | `/api/permissions`      | grant `{user_id, collection, role}` |
| `DELETE` | `/api/permissions/{id}` | revoke                              |

```sh
curl -X POST http://localhost:8080/api/permissions \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{"user_id":"usr_...","collection":"posts","role":"editor"}'
```

There is no self-service "my permissions" endpoint — list permissions with an
admin token.

## Limitations vs PocketBase/Supabase

- No row-level security: a `viewer` on `posts` sees **all** posts.
- No field-level rules.
- No per-request rule expressions (PocketBase filters).
- No anonymous/public access flag — every protected endpoint needs a JWT.
