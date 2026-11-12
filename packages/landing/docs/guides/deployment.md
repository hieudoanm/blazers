# Deployment

Blazers is designed to run anywhere a single static binary runs — a VPS, a
container, a Raspberry Pi, or a CI job. There is no external service to
provision.

## Minimal deploy

```sh
export JWT_SECRET="$(openssl rand -hex 32)"   # required in production
export BLAZERS_DATA=/var/lib/blazers
export PORT=8080
blazers serve
```

That's it. The server:

- creates `BLAZERS_DATA` and `data.db` on first run (migrations automatic)
- generates the secrets-vault key inside `BLAZERS_DATA`
- serves the API, admin dashboard (`/`), OpenAPI (`/api/docs`), and all realtime
  streams on `PORT`

## Docker

```dockerfile
FROM debian:bookworm-slim
COPY blazers /usr/local/bin/blazers
ENV BLAZERS_DATA=/data JWT_SECRET=change-me PORT=8080
VOLUME /data
EXPOSE 8080
CMD ["blazers", "serve"]
```

Mount `/data` as a volume — it holds the database, uploaded files, and the
secrets key.

## systemd unit

```ini
[Unit]
Description=Blazers BaaS
After=network.target

[Service]
Environment=JWT_SECRET=... BLAZERS_DATA=/var/lib/blazers PORT=8080
ExecStart=/usr/local/bin/blazers serve
Restart=always
User=blazers

[Install]
WantedBy=multi-user.target
```

## Behind a reverse proxy

Blazers is plain HTTP/1.1 — put Caddy/nginx in front for TLS:

```caddyfile
api.example.com {
    reverse_proxy localhost:8080
}
```

WebSocket (`/ws`) and SSE (`/api/*/stream`) both work through standard reverse
proxies — SSE needs no special config beyond disabling response buffering if
your proxy buffers by default.

## Data backup

Three options, simplest first:

1. **File copy** — stop the server, copy `BLAZERS_DATA/` (SQLite + files +
   secrets key). To back up live, use option 2.
2. **`GET /api/backup`** — admin-only SQLite snapshot download; restore by
   placing the file as `data.db`.
3. **`GET /api/export`** — portable JSON (collections, records, buckets, file
   metadata); pair with a copy of `BLAZERS_DATA/files/` for bytes.

## Security checklist

- [ ] `JWT_SECRET` set to a long random value (not the default)
- [ ] `BLAZERS_DATA` on encrypted storage; secrets key backed up separately
- [ ] Rate limiting is per-IP — front with a proxy if you need global limits
- [ ] SSE streams (`/api/logs/stream`, `/api/pubsub/*/stream`,
      `/api/notifications/stream`) are **unauthenticated** — restrict them at
      the proxy if the data is sensitive
- [ ] `/api/backup`, `/api/import`, `/api/export` need an `admin` token — keep
      admin credentials out of client-side code
- [ ] CORS is permissive — add origin restrictions at the proxy for
      browser-facing deployments

## Scaling notes

Blazers is single-node by design: one process, one SQLite file, in-memory cache
and realtime hubs. It comfortably handles thousands of concurrent SSE/WS
connections and moderate write throughput. For multi-node you would need
external Postgres/Redis — at that point you're past the "one binary" value
proposition; run one instance per tenant instead.
