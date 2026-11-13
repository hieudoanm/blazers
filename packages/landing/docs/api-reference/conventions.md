# API conventions

Everything under `/api/*` except `/api/health` requires a JWT unless noted.

## Base URL

```text
http://localhost:8080        # default (PORT env var)
```

## Authentication

```http
Authorization: Bearer <jwt>
```

Get a token from `POST /api/auth/login` or `/api/auth/register`. See
[Authentication](../concepts/authentication.md).

## Content types

- Requests that mutate data must send `Content-Type: application/json` (enforced
  by middleware) — **except** multipart uploads (`/files` endpoints) and the SSE
  streams.
- Responses are `application/json` except file downloads and `/api/backup`
  (SQLite bytes) and `/api/docs` (HTML).

## Errors

Errors share one shape via `AppError`:

```json
{ "error": "human readable message" }
```

| Status      | Meaning                                  |
| ----------- | ---------------------------------------- |
| `400`       | bad request — validation, missing fields |
| `401`       | missing/invalid/expired JWT              |
| `403`       | authenticated but RBAC denies the role   |
| `404`       | resource not found                       |
| `405`       | wrong method                             |
| `413`/`400` | body over 10 MiB limit                   |
| `429`       | rate limit exceeded                      |
| `500`       | internal error                           |

## Pagination

List endpoints that page accept `?page=` (1-based) and `?per_page=` (clamped
`1..=100`, default `20`) and return:

```json
{ "items": [...], "total": N, "page": 1, "per_page": 20, "total_pages": M }
```

The key holding the items varies (`records`, `files`, …) — see each endpoint's
page.

## Rate limiting

A per-IP token bucket runs on the whole `/api` router before handlers. Burst
allowance is modest; SSE streams and the admin dashboard's polling are designed
to stay well under it.

## CORS

`CorsLayer::permissive()` — any origin may call the API from a browser. Do not
expose a Blazers instance holding sensitive data to untrusted origins without a
reverse-proxy layer in front.

## Batch requests

`POST /api/batch` runs several API calls in one round-trip (PocketBase
parity). Body:

```json
{ "requests": [
    { "method": "POST", "url": "/api/collections/items/records",
      "body": { "data": { "n": 1 } } },
    { "method": "GET", "url": "/api/collections/items/records" }
]}
```

PocketBase-style `?batch=1` also works on **any** `/api/*` route: send the
same payload as a bare JSON array (or the `requests` envelope) and the
response is the array of results — e.g.
`POST /api/collections?batch=1`.

Each sub-request is dispatched through the real router — auth, RBAC and
validation all apply. Response is an array in request order:

```json
[ { "status": 200, "body": { ... } }, { "status": 404, "body": { "error": "..." } } ]
```

Max 50 sub-requests; `/api/batch` itself and non-`/api/` URLs are rejected
per-item with `400`.

## OpenAPI & docs

- `GET /api/openapi.json` — full OpenAPI 3 spec
- `GET /api/docs` — Swagger UI pointed at that spec

## Request flow

```text
client ──▶ rate limit ──▶ content-type check ──▶ JWT auth ──▶ RBAC ──▶ handler
                │                                                  │
                └── request log (stdout)              SQLite · filesystem · SSE hubs
```
