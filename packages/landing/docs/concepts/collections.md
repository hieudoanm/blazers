# Collections & records

Collections are the heart of Blazers — the equivalent of PocketBase
collections and Supabase tables, but with a deliberately small surface.

## Collections

A collection has a name and a JSON **schema**:

```json
{
  "name": "posts",
  "schema": {
    "title": "string",
    "body?": "string",
    "views": "integer",
    "draft": "boolean",
    "meta": "object"
  },
  "created_at": "...",
  "updated_at": "..."
}
```

### Schema format

`schema` is a flat JSON object: **field name → type string**.

| Type      | SQLite column | Validation                   |
| --------- | ------------- | ---------------------------- |
| `string`  | `TEXT`        | must be a JSON string        |
| `email`   | `TEXT`        | string (format not enforced) |
| `url`     | `TEXT`        | string                       |
| `number`  | `REAL`        | must be a JSON number        |
| `integer` | `INTEGER`     | must be an integer           |
| `boolean` | `INTEGER`     | must be `true`/`false`       |
| `array`   | `TEXT`        | JSON-encoded array           |
| `object`  | `TEXT`        | JSON-encoded object          |

Append `?` to the field name to make it optional:

```json
{ "slug?": "string", "tags?": "array" }
```

Fields **not** in the schema are still stored inside the record's `data`
JSON blob — the schema governs typed columns and required-field validation,
not a hard column allowlist. `{}` (or omitting `schema`) means schemaless.

### Updating a schema

`PATCH /api/collections/{name}` with `{"schema": "..."}` runs
`migrate_collection_schema`, which adds new columns and relaxes types.
Changing a field's type on existing data is not a migration tool — treat
schema edits as additive.

## Records

A record is `{ id, data, created_at, updated_at }`. `data` is arbitrary JSON;
typed schema fields are validated on create/update (`validate_data`).

```json
{
  "id": "rec_...",
  "data": { "title": "Hello", "views": 3, "draft": false },
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z"
}
```

You may supply your own `id` on create (`CreateRecordRequest.id`) — useful
for imports and idempotent seeds.

## Listing & search

`GET /api/collections/{name}/records` accepts:

| Param      | Default | Notes                              |
| ---------- | ------- | ---------------------------------- |
| `page`     | `1`     | 1-based                            |
| `per_page` | `20`    | clamped to `1..=100`               |
| `search`   | `""`    | substring match across record data |

Response:

```json
{
  "records": [...],
  "total": 137,
  "page": 1,
  "per_page": 20,
  "total_pages": 7
}
```

## Permissions

Collection access is governed by [RBAC permissions](permissions.md): each
user has a role (`viewer`, `editor`, `admin`) per collection name, or `*`
for global admin. Handlers call `rbac::require_role(conn, user_id, name,
"viewer")` before touching records.
