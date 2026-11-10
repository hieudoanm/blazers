# Cache

An in-memory key/value cache with optional TTL and Redis-style data types —
strings, lists, and hashes with atomic operations.

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

- `value` is an opaque string for `POST /api/cache`. Lists and hashes use
  dedicated endpoints and are stored as typed JSON envelopes
  (`{"__type":"list","items":[...]}` / `{"__type":"hash","fields":{...}}`),
  so they share the same keyspace, TTL, and persistence.
- `ttl` is in **seconds**; `0`/absent means no expiry. `EXPIRE`/`TTL`
  endpoints manage it after creation.
- Entries persist to SQLite (`cache` table) and reload on boot, so the
  cache survives restarts.
- All mutating operations (`INCR`, `APPEND`, list/hash ops) are atomic —
  they run under the store's write lock.

## REST

| Method   | Endpoint                        | Notes                                            |
| -------- | ------------------------------- | ------------------------------------------------ |
| `GET`    | `/api/cache`                    | list all entries                                 |
| `POST`   | `/api/cache`                    | set `{key, value, ttl?}` (string)                |
| `GET`    | `/api/cache/stats`              | `{"total_entries", "expired_entries", "memory"}` |
| `GET`    | `/api/cache/{key}`              | fetch one                                        |
| `DELETE` | `/api/cache/{key}`              | delete one                                       |
| `DELETE` | `/api/cache`                    | flush everything                                 |
| `GET`    | `/api/cache/keys`               | sorted key names                               |
| `GET`    | `/api/cache/exists/{key}`       | `{"exists": bool}`                             |
| `POST`   | `/api/cache/delete`             | variadic delete `{keys: [...]}` → count        |
| `POST`   | `/api/cache/{key}/incr`         | atomic `{by?}` increment → new value             |
| `POST`   | `/api/cache/{key}/append`       | atomic string append → full value                |
| `POST`   | `/api/cache/{key}/expire`       | set TTL `{ttl}` (`0` clears)                     |
| `GET`    | `/api/cache/{key}/ttl`          | seconds left, `-1` = none                        |
| `POST`   | `/api/cache/{key}/list`         | RPUSH/LPUSH `{values, left?}` → length           |
| `GET`    | `/api/cache/{key}/list`         | LRANGE `?start&stop`                             |
| `POST`   | `/api/cache/{key}/list/pop`     | LPOP/RPOP `{left?, count?}`                      |
| `POST`   | `/api/cache/{key}/hash`         | HSET `{field,value}` or `{fields}`               |
| `GET`    | `/api/cache/{key}/hash`         | HGETALL                                           |
| `GET`    | `/api/cache/{key}/hash/{field}` | HGET                                             |
| `DELETE` | `/api/cache/{key}/hash/{field}` | HDEL (comma-separated fields)                    |

See [Cache API](../api-reference/cache.md) for request/response details and
`WRONGTYPE` semantics.

## KV TCP listener

`blazers kv` exposes the same `CacheStore` over a Redis-style inline TCP
protocol (port 6379 by default) — `SET`/`GET`/`DEL`/`KEYS`/`EXPIRE`/`TTL`
plus `INCR`, list and hash commands. It shares the SQLite `_cache` table, so
REST and TCP clients see one dataset. `BLAZERS_KV_PASSWORD` enables `AUTH`.

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
