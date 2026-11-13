# Functions API

Concepts: [Edge functions](../concepts/functions.md)

Base: `/api/functions`. Management endpoints require
`Authorization: Bearer <token>` with the `admin` role; invocation is open (send
a token to expose the caller to the function).

## `GET /api/functions`

List stored functions. **Admin only.**

**Response `200`** — array of `{ "name", "code", "created_at", "updated_at" }`.

## `POST /api/functions`

Create or replace a function (upsert by `name`). **Admin only.**

**Body**

```json
{
  "name": "double",
  "code": "function handler(ctx) { return { value: ctx.input.n * 2 }; }"
}
```

**Response `200`** — the stored function object. **Errors** — `400` missing
`name`/`code`; `403` not admin.

## `POST /api/functions/{name}` — invoke

**Body**

```json
{ "input": { "n": 21 } }
```

The handler receives `{ "input": <body.input>, "auth": {...} | null }`.

**Response `200`**

```json
{ "result": { "value": 42 } }
```

**Errors** — `400` JS exception / missing handler / timeout abort; `404` unknown
function.

## `DELETE /api/functions/{name}`

Delete a function. **Admin only.**

**Response** — `204`; `404` unknown function; `403` not admin.
