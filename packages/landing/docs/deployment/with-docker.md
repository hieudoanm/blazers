# Deploy with Docker

The repo ships a multi-stage `Dockerfile`: a Rust builder stage compiles the
release binary, then a slim Debian runtime runs it as a non-root `blazers` user.
`ca-certificates` is included so outbound HTTPS (webhooks, OAuth, SMTP) works
out of the box.

The image sets `BLAZERS_DATA=/data` and `PORT=8080` by default. Mount a volume
at `/data` — it holds `data.db`, uploaded files and the secrets key.

## Prebuilt image

Published to GHCR on every push to `master`:

```sh
docker run -d --name blazers -p 8080:8080 \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  -v blazers-data:/data \
  ghcr.io/hieudoanm/blazers:latest
# → http://localhost:8080 (admin UI at /, API docs at /api/docs)
```

## Build from source

```sh
docker build -t blazers .
docker run -d --name blazers -p 8080:8080 \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  -v blazers-data:/data \
  blazers
```

## Docker Compose

```yaml
services:
  blazers:
    image: ghcr.io/hieudoanm/blazers:latest
    ports: ['8080:8080']
    environment:
      JWT_SECRET: change-me
    volumes:
      - blazers-data:/data
volumes:
  blazers-data:
```

`docker compose up -d` → http://localhost:8080

## Environment variables

| Variable              | Default      | Purpose                                       |
| --------------------- | ------------ | --------------------------------------------- |
| `PORT`                | `8080`       | HTTP listen port                              |
| `BLAZERS_DATA`        | `~/.blazers` | Data dir (`data.db`, `files/`, `secrets.key`) |
| `JWT_SECRET`          | generated    | JWT signing secret — **set in production**    |
| `BLAZERS_SECRETS_KEY` | generated    | Vault encryption key — back it up             |
| `OAUTH_*`             | —            | OAuth provider client id/secret pairs         |

## Health checks

`GET /api/health` returns `200` when the server is up — use it for Docker
`HEALTHCHECK`, load-balancer health checks, or platform readiness probes.

## Notes

- The container runs as the non-root `blazers` user; the named volume handles
  permissions automatically.
- SQLite lives on the mounted volume — do **not** put `/data` on a network
  filesystem (NFS/SMB); use a local volume or block storage.
- Backups: `GET /api/backup` (admin token) downloads a `data.db` snapshot, or
  simply copy the volume while the container is stopped.
