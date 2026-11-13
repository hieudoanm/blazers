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

## Record rules

On top of RBAC, each collection carries five optional PocketBase-style rule
expressions — `list_rule`, `view_rule`, `create_rule`, `update_rule`,
`delete_rule` — set via `PATCH /api/collections/{name}`:

```json
{
  "list_rule": "status = 'published' || owner = @request.auth.id",
  "create_rule": "@request.auth.verified = true"
}
```

- Empty rule = allow everyone (subject to RBAC).
- `admin` role bypasses rules entirely.
- Expressions use the same syntax as the `?filter=` query param — see
  [Records API](../api-reference/records.md#list-records-with-filter-sort-expand-and-pagination)
  for operators. `@request.auth.*` resolves to the caller's user record (`id`,
  `email`, `verified`, plus custom `data` fields); `@request.data.*` resolves to
  the record being written.
- `list_rule` is ANDed with the client's `?filter=` **before** pagination;
  `view`/`update`/`delete` rules evaluate per record; `create_rule` also runs
  against the merged record on update.

Still missing vs PocketBase: field-level rules, managed/auth collections as
first-class citizens, and request-info beyond `@request.auth`/`@request.data`.

## Limitations vs PocketBase/Supabase

- Rules are row-level only — no field-level or column masking.
- Every protected endpoint still needs a JWT; there is no anonymous flag.
