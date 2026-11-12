# Quickstart

This guide takes you from zero to a working Blazers backend in a few minutes.
All examples use `curl` against the default `http://localhost:8080`.

## 1. Start the server

```sh
blazers serve
```

## 2. Register a user, then log in

```sh
curl -X POST http://localhost:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"s3cret"}'
```

Response — just the user record (register does **not** return a token):

```json
{
  "id": "...",
  "email": "admin@example.com",
  "created_at": "...",
  "updated_at": "..."
}
```

Now log in to get a token:

```sh
curl -X POST http://localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"s3cret"}'
```

Response:

```json
{
  "user": {
    "id": "...",
    "email": "admin@example.com",
    "created_at": "...",
    "updated_at": "..."
  },
  "token": "eyJhbGciOi..."
}
```

```sh
TOKEN=eyJhbGciOi...
AUTH="Authorization: Bearer $TOKEN"
```

## 3. Create a collection

A collection is a table. Its `schema` maps field names to types; append `?` to a
field name to make it optional.

```sh
curl -X POST http://localhost:8080/api/collections \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{
    "name": "posts",
    "schema": {
      "title": "string",
      "body?": "string",
      "views": "integer",
      "draft": "boolean"
    }
  }'
```

## 4. Create and read records

```sh
# create
curl -X POST http://localhost:8080/api/collections/posts/records \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{"data": {"title": "Hello Blazers", "body": "First post", "views": 0, "draft": false}}'

# list (paginated, searchable)
curl "http://localhost:8080/api/collections/posts/records?page=1&per_page=20&search=Hello" \
  -H "$AUTH"

# get one
curl http://localhost:8080/api/collections/posts/records/<id> -H "$AUTH"

# update
curl -X PATCH http://localhost:8080/api/collections/posts/records/<id> \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{"data": {"views": 42}}'
```

Record list responses are paginated:

```json
{
  "records": [ { "id": "...", "data": { "title": "Hello Blazers", ... },
                 "created_at": "...", "updated_at": "..." } ],
  "total": 1, "page": 1, "per_page": 20, "total_pages": 1
}
```

## 5. Upload a file

```sh
# create a bucket first
curl -X POST http://localhost:8080/api/buckets \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{"name": "avatars", "is_public": true}'

# upload (multipart, single file field)
curl -X POST http://localhost:8080/api/buckets/avatars/files \
  -H "$AUTH" \
  -F 'file=@avatar.png'

# download
curl http://localhost:8080/api/buckets/avatars/files/<file_id> -H "$AUTH" -o avatar.png
```

## 6. Open the admin dashboard

Browse to <http://localhost:8080/> and sign in with the same email/password. The
dashboard gives you a UI over collections, records, buckets, webhooks, secrets,
WebSockets, cache, notifications, logs, and pub/sub.

## 7. Try realtime

```sh
# server-sent events for logs
curl -N http://localhost:8080/api/logs/stream

# pub/sub topic stream
curl -N http://localhost:8080/api/pubsub/<topic-name>/stream
```

See [Realtime](../concepts/realtime.md) for WebSockets and pub/sub details.

## Next steps

- [API conventions](../api-reference/conventions.md) — auth header, errors,
  pagination
- [Webhooks](../concepts/webhooks.md) — get HTTP callbacks on data changes
- [MCP server](../guides/mcp-server.md) — let AI agents operate your backend
- [PocketBase comparison](../../README.md#pocketbase-comparison) — feature
  matrix
