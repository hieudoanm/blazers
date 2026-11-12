# Records API

Concepts: [Collections & records](../concepts/collections.md)

Base: `/api/collections/{name}/records` — RBAC `viewer` to read, `editor` to
write (`admin` satisfies both via the role rules).

## `GET /api/collections/{name}/records`

List records with pagination and search.

**Query**

| Param      | Default | Range                          |
| ---------- | ------- | ------------------------------ |
| `page`     | `1`     | ≥ 1                            |
| `per_page` | `20`    | `1..=100`                      |
| `search`   | `""`    | substring match on record data |

**Response `200`**

```json
{
  "records": [ { "id": "...", "data": {...},
                 "created_at": "...", "updated_at": "..." } ],
  "total": 42, "page": 1, "per_page": 20, "total_pages": 3
}
```

## `POST /api/collections/{name}/records`

Create a record.

**Body**

```json
{ "id": "optional-custom-id", "data": { "title": "Hello", "views": 0 } }
```

`data` is validated against the collection schema (required fields, types). `id`
may be omitted (UUID generated).

**Response `201`** — the created `Record`.

Fires `record.create` webhook.

## `GET /api/collections/{name}/records/{id}`

Fetch one record. `404` if missing.

## `PATCH /api/collections/{name}/records/{id}`

**Replaces** the record's `data` object wholesale (not a merge) and re-validates
against the full schema — send the complete object.

**Body**

```json
{ "data": { "views": 10 } }
```

Fires `record.update` webhook.

## `DELETE /api/collections/{name}/records/{id}`

Delete a record. `204`. Fires `record.delete` webhook.

## Errors

- `400` — schema validation failure (missing required field, wrong type)
- `404` — collection or record not found
- `403` — RBAC denies the role
