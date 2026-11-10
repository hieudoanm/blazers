# Buckets & files

Buckets are named file containers — the equivalent of PocketBase buckets and
Supabase Storage buckets.

## Buckets

```json
{
  "name": "avatars",
  "is_public": true,
  "created_at": "...",
  "updated_at": "..."
}
```

- `is_public` is stored on the bucket and surfaced in the API. Note that
  `download_file` currently requires a valid JWT for **all** buckets —
  treat `is_public` as metadata until public-download enforcement is added.
- Deleting a bucket deletes its file metadata (verify filesystem cleanup in
  your deployment if you rely on it).

## Uploading

`POST /api/buckets/{name}/files` accepts `multipart/form-data` with a single
file field (any field name; the handler reads the first field):

```sh
curl -X POST http://localhost:8080/api/buckets/avatars/files \
  -H "Authorization: Bearer $TOKEN" \
  -F 'file=@avatar.png'
```

Response `201`:

```json
{
  "id": "...",
  "bucket": "avatars",
  "filename": "avatar.png",
  "mime_type": "image/png",
  "size": 1024,
  "created_at": "...",
  "updated_at": "..."
}
```

Request bodies are capped at 10 MiB by the global body limit.

## Listing & pagination

`GET /api/buckets/{name}/files?page=1&per_page=20` returns:

```json
{
  "files": [...],
  "total": 1, "page": 1, "per_page": 20, "total_pages": 1
}
```

## Download & delete

```http
GET    /api/buckets/{name}/files/{id}   → file bytes (Content-Type from mime_guess)
DELETE /api/buckets/{name}/files/{id}   → 204
```

## Thumbnails

For image files, a JPEG thumbnail can be generated on demand:

```http
GET /api/buckets/{name}/files/{id}/thumb
```

`image::generate_thumbnail` decodes the stored file, scales it to fit a
bounded box, and returns JPEG bytes — handy for avatar grids without a
separate image pipeline.
