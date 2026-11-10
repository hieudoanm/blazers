# Cache API

Concepts: [Cache](../concepts/cache.md)

Base: `/api/cache` — admin-level RBAC.

## `GET /api/cache` — list

All entries:

```json
[
  {
    "key": "theme",
    "value": "dark",
    "ttl": 3600,
    "expires_at": "2025-01-01T01:00:00Z",
    "created_at": "...",
    "updated_at": "..."
  }
]
```

## `POST /api/cache` — set

**Body**

```json
{ "key": "theme", "value": "dark", "ttl": 3600 }
```

`key` and `value` required; `ttl` in seconds (`0`/absent = never expires).

**Response `201`** — the `CacheEntry`. `400` if key/value empty.

## `GET /api/cache/{key}` — get

Returns the `CacheEntry` or `404`.

## `DELETE /api/cache/{key}` — delete one

`204`; `404` if the key doesn't exist.

## `DELETE /api/cache` — flush

Wipe all entries. `200` → `{"status": "flushed"}`.

## `GET /api/cache/stats` — stats

```json
{ "total_entries": 12, "expired_entries": 3, "memory": true }
```
