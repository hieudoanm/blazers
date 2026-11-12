# Buckets API

Concepts: [Buckets & files](../concepts/buckets-and-files.md)

Base: `/api/buckets` — RBAC `viewer` to list/read, `editor`+ to mutate.

## `GET /api/buckets`

List all buckets.

**Response `200`** — array of `Bucket` objects:

```json
{
  "name": "avatars",
  "is_public": true,
  "created_at": "...",
  "updated_at": "..."
}
```

## `POST /api/buckets`

Create a bucket.

**Body**

```json
{ "name": "avatars", "is_public": true }
```

**Response `201`** — the `Bucket`. Fires `bucket.create` webhook.

## `GET /api/buckets/{name}`

Fetch one bucket. `404` if missing.

## `DELETE /api/buckets/{name}`

Delete the bucket. `204`. Fires `bucket.delete` webhook.

## Files under a bucket

File endpoints live under `/api/buckets/{name}/files` — see
[Files API](files.md).
