# Deployment

Blazers is designed to run anywhere a single static binary or container runs — a
VPS, a container platform, a Raspberry Pi, or a CI job. There is no external
service to provision.

## Platform guides

Step-by-step setup for each target:

**with-\*** — run it yourself

- [Docker](deployment/with-docker.md) — image, Compose, volumes, env vars
- [Terraform](deployment/with-terraform.md) — AWS, Azure and GCP as code

**paas-\*** — platform-as-a-service

- [Render](deployment/paas-render.md) — blueprint + persistent disks
- [Heroku](deployment/paas-heroku.md) — container stack (ephemeral disk caveat)

**iaas-\*** — infrastructure-as-a-service

- [AWS](deployment/iaas-aws.md) — ECS Fargate + EFS, or EC2
- [Azure](deployment/iaas-azure.md) — Container Apps + Azure Files
- [GCP](deployment/iaas-gcp.md) — Compute Engine, or Cloud Run (ephemeral)

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
