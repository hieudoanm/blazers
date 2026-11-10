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

## `GET /api/cache/keys` — key names

```json
{ "keys": ["session:abc", "visits", "user:1"] }
```

Sorted names of all unexpired entries (strings, lists and hashes alike).

## `GET /api/cache/exists/{key}` — existence check

```json
{ "key": "theme", "exists": true }
```

## `POST /api/cache/delete` — delete many

**Body** `{ "keys": ["a", "b"] }` → `{ "deleted": 2 }`. `400` on empty `keys`.

## `GET /api/cache/stats` — stats

```json
{ "total_entries": 12, "expired_entries": 3, "memory": true }
```

## Typed values — strings, lists, hashes

`POST /api/cache` always stores a **string**. Two richer types are available
through dedicated endpoints; they share the same key namespace and TTL rules.
Using a typed op on a key of the wrong type returns `400 WRONGTYPE`.

- **List** — stored as a JSON envelope; a plain `GET /api/cache/{key}` returns
  the encoded payload, e.g. `{"__type":"list","items":[...]}`.
- **Hash** — `{"__type":"hash","fields":{...}}`.

### Counters & strings

| Endpoint | Body | Response |
|---|---|---|
| `POST /api/cache/{key}/incr` | `{ "by": 1 }` (`by` optional, default 1) | `{ "key", "value" }` — new integer value |
| `POST /api/cache/{key}/append` | `{ "value": "..." }` | `{ "key", "value" }` — full string after append |
| `POST /api/cache/{key}/expire` | `{ "ttl": 3600 }` (`0` clears expiry) | `{ "key", "ttl" }`; `404` if missing |
| `GET /api/cache/{key}/ttl` | — | `{ "key", "ttl" }` — seconds left, `-1` = no expiry; `404` if missing |

`INCR` on a missing key starts at 0; on a non-integer string it errors.
All mutations are atomic.

### Lists

| Endpoint | Body / query | Response |
|---|---|---|
| `POST /api/cache/{key}/list` | `{ "values": [...], "left": false }` | `{ "key", "length" }` — push right (RPUSH) or left (LPUSH) |
| `POST /api/cache/{key}/list/pop` | `{ "left": false, "count": 1 }` | `{ "key", "value" }` — single item, or array when `count > 1`; `null` when empty |
| `GET /api/cache/{key}/list?start=0&stop=-1` | Redis-style inclusive range, negatives from end | `{ "key", "items": [...] }` |

An emptied list key is deleted, like Redis.

### Hashes

| Endpoint | Body | Response |
|---|---|---|
| `POST /api/cache/{key}/hash` | `{ "field": "f", "value": v }` or `{ "fields": {"f": v, ...} }` | `{ "key", "fields_set" }` |
| `GET /api/cache/{key}/hash` | — | `{ "key", "fields": {...} }` |
| `GET /api/cache/{key}/hash/{field}` | — | `{ "key", "field", "value" }` (`value: null` if absent) |
| `DELETE /api/cache/{key}/hash/{field}` | comma-separated fields allowed | `{ "key", "removed" }` |

An emptied hash key is deleted. Hash field values are arbitrary JSON.

```sh
curl -X POST localhost:8090/api/cache/visits/incr \
  -H "Authorization: Bearer $TOKEN" -d '{"by": 1}'
curl -X POST localhost:8090/api/cache/jobs/list \
  -H "Authorization: Bearer $TOKEN" -d '{"values": ["job-1"]}'
curl -X POST localhost:8090/api/cache/user:42/hash \
  -H "Authorization: Bearer $TOKEN" -d '{"fields": {"plan": "pro"}}'
```

## KV TCP protocol {#kv-tcp-protocol}

`blazers kv [--port 6379] [--bind 127.0.0.1]` serves the same cache over a
Redis-style **inline protocol**: one command per line, one reply per line —
compatible with kevin and usable with `nc`, `telnet`, or any line client.

```sh
$ blazers kv --port 6379 &
$ printf 'PING\nSET k v EX 30\nGET k\nTTL k\n' | nc 127.0.0.1 6379
PONG
OK
v
29
```

| Command | Reply |
|---|---|
| `PING` | `PONG` |
| `SET key value [EX seconds]` | `OK` / `ERR invalid expire time` |
| `GET key` | value, `(nil)`, or the typed JSON envelope |
| `DEL key [key ...]` | count removed |
| `EXISTS key` | `1` / `0` |
| `KEYS [pattern]` | sorted keys; `*` and `?` glob |
| `TYPE key` | `string` / `list` / `hash` / `none` |
| `LEN` | unexpired key count |
| `FLUSHALL` / `FLUSHDB` | `OK` |
| `EXPIRE key seconds` | `1` / `0` |
| `TTL key` | seconds left, `-1` no expiry, `-2` missing |
| `INCR key` / `DECR key` / `INCRBY key n` | new integer |
| `APPEND key value` | new string length |
| `LPUSH`/`RPUSH key v [v ...]` | new list length |
| `LPOP`/`RPOP key [count]` | item(s) or `(nil)` |
| `LRANGE key start stop` | space-separated slice |
| `HSET key f v [f v ...]` | fields written |
| `HGET key f` / `HGETALL key` / `HDEL key f ...` / `HLEN key` | field ops |

Errors are `ERR …` lines; type mismatches are `ERR WRONGTYPE …`.

Set `BLAZERS_KV_PASSWORD` to require `AUTH password` before any other
command (`NOAUTH` until authenticated, `WRONGPASS` on a bad password).
