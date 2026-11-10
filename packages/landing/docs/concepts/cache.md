# Cache

An in-memory key/value cache with optional TTL — the equivalent of
PocketBase's cache plus a Redis `SET ... EX` in one endpoint.

## Model

```json
{
  "key": "feature:dark-mode",
  "value": "{\"enabled\":true}",
  "ttl": 300,
  "expires_at": "2025-01-01T00:05:00Z",
  "created_at": "...",
  "updated_at": "..."
}
```

- `value` is an opaque string (JSON-encode if you like).
- `ttl` is in **seconds**; `0`/absent means no expiry.
- Entries persist to SQLite (`cache` table) and reload on boot, so the
  cache survives restarts.

## REST

| Method   | Endpoint           | Notes                                            |
| -------- | ------------------ | ------------------------------------------------ |
| `GET`    | `/api/cache`       | list all entries                                 |
| `POST`   | `/api/cache`       | set `{key, value, ttl?}`                         |
| `GET`    | `/api/cache/stats` | `{"total_entries", "expired_entries", "memory"}` |
| `GET`    | `/api/cache/{key}` | fetch one                                        |
| `DELETE` | `/api/cache/{key}` | delete one                                       |
| `DELETE` | `/api/cache`       | flush everything                                 |

```sh
curl -X POST http://localhost:8080/api/cache \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{"key":"config:theme","value":"dark","ttl":3600}'
```

## Internals

`CacheStore` is an `Arc<RwLock<HashMap>>` in `crates/blazers-server/src/cache.rs`:

- `set` computes `expires_at` from `ttl` and upserts.
- An eviction loop runs periodically to drop expired entries.
- `load_from_db` hydrates the map at startup; writes go through to SQLite.
- `stats` reports `total_entries` and `expired_entries` (no hit/miss
  counters yet).

## When to use it

- Feature flags and runtime config your API reads per request
- Expensive computed results (report snapshots, aggregated counts)
- Rate-limit counters or idempotency keys (short TTLs)

It is **not** a substitute for Redis when you need multi-node coherence —
each Blazers instance has its own cache.
