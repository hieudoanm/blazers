# Collections API

Concepts: [Collections & records](../concepts/collections.md)

All endpoints require `Authorization: Bearer <token>` and RBAC `viewer` (or
higher) on the collection.

## `GET /api/collections`

List all collections.

**Response `200`** — array of:

```json
{ "name": "posts", "schema": "{...}", "created_at": "...", "updated_at": "..." }
```

## `POST /api/collections`

Create a collection.

**Body**

```json
{
  "name": "posts",
  "schema": "{\"title\":\"string\",\"views\":\"integer\",\"draft\":\"boolean\"}"
}
```

`schema` may be a JSON **string** or object (serialized to string by the
handler). Omit for schemaless. The schema is validated: it must be a JSON object
mapping field names to known types (`string`, `number`, `integer`, `boolean`,
`array`, `object`, `email`, `url`, optional `?` suffix) — anything else returns
`400`.

**Response `201`** — the created `Collection`.

Fires `collection.create` webhook.

## `GET /api/collections/{name}`

Fetch one collection. `404` if missing.

## `PATCH /api/collections/{name}`

Update the schema.

**Body**

```json
{ "schema": "{\"title\":\"string\",\"slug?\":\"string\"}" }
```

Runs `migrate_collection_schema` — adds columns for new fields; does not
transform existing data.

## `DELETE /api/collections/{name}`

Delete the collection and its records. `204`.

Fires `collection.delete` webhook.
