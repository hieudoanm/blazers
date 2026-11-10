# Admin & ops API

Miscellaneous operational endpoints — backup, portability, and docs.

## `GET /api/health`

Liveness probe. **No auth.**

```json
{ "status": "ok" }
```

## `GET /api/backup`

Download a consistent copy of the SQLite database.

- Requires `admin` role on `*` (global admin).
- Response: `application/octet-stream` SQLite file
  (`blazers-backup-<uuid>.db`), suitable for `sqlite3` inspection or
  restoring into `BLAZERS_DATA/data.db` while the server is stopped.

```sh
curl http://localhost:8080/api/backup -H "$AUTH" -o backup.db
```

## `GET /api/export` — portability export

JSON document with **all** collections, records, buckets, and file
metadata:

```json
{
  "collections": [...],
  "records": { "posts": [ ... ], "...": [ ... ] },
  "buckets": [...],
  "files": [...]
}
```

Requires auth. Does **not** include file bytes — pair with a filesystem
backup of `BLAZERS_DATA/files/` for a full export.

## `POST /api/import` — portability import

Accepts the same `ExportData` shape (every field optional — partial
payloads are fine). Query param `?skip_existing=true|false`:

- `skip_existing=true` — conflicting ids are skipped, import continues.
- `skip_existing=false` (**default**) — first conflict aborts with `409`.

```sh
curl -X POST 'http://localhost:8080/api/import?skip_existing=true' \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d @export.json
```

## `GET /api/openapi.json`

OpenAPI 3 document describing the API surface.

## `GET /api/docs` (and `/api/docs/`)

Swagger UI pointed at `/api/openapi.json`.

## `GET /` and static assets

Any unmatched path falls through to the embedded admin dashboard
(`crates/blazers-server/public/`): `index.html`, `styles.css`, and the
HTMX/Mustache assets it references. See
[Admin dashboard](../guides/admin-dashboard.md).
