# Files API

Concepts: [Buckets & files](../concepts/buckets-and-files.md)

Base: `/api/buckets/{name}/files` — RBAC `viewer` to read, `editor`+ to
upload/delete. All requests need a JWT.

## `POST /api/buckets/{name}/files` — upload

`multipart/form-data` with a single file field (field name is ignored — the
first field wins):

```sh
curl -X POST http://localhost:8080/api/buckets/avatars/files \
  -H "Authorization: Bearer $TOKEN" \
  -F 'file=@photo.png'
```

**Response `201`**

```json
{
  "id": "...",
  "bucket": "avatars",
  "filename": "photo.png",
  "mime_type": "image/png",
  "size": 12345,
  "created_at": "...",
  "updated_at": "..."
}
```

- Max body size: 10 MiB (global limit).
- `filename` is taken from the multipart filename.
- `mime_type` from the part's content type, falling back to `octet-stream`.

## `GET /api/buckets/{name}/files` — list

**Query**: `page` (default 1), `per_page` (default 20, max 100).

**Response `200`**

```json
{
  "files": [ {FileRecord...} ],
  "total": 5, "page": 1, "per_page": 20, "total_pages": 1
}
```

## `GET /api/buckets/{name}/files/{id}` — download

Streams the file bytes with the stored `Content-Type`. `404` if the file or
bucket is missing.

## `DELETE /api/buckets/{name}/files/{id}`

Delete metadata + stored bytes. `204`.

## `GET /api/buckets/{name}/files/{id}/thumb` — thumbnail

Returns a JPEG thumbnail for image files (decoded, scaled, re-encoded).
`400`/`500` for non-images or undecodable content.
