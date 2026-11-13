# Deploy to Render

Render runs Docker images directly and supports persistent disks — a good fit
for Blazers since SQLite needs a real filesystem.

## Option A: Blueprint (recommended)

Add `render.yaml` to your repo (or create one in the dashboard):

```yaml
services:
  - type: web
    name: blazers
    runtime: docker
    dockerfilePath: ./Dockerfile
    plan: starter # disks require a paid instance type
    disk:
      name: blazers-data
      mountPath: /data
      sizeGB: 1
    healthCheckPath: /api/health
    envVars:
      - key: JWT_SECRET
        generateValue: true
      - key: PORT
        value: '8080'
```

Then **New → Blueprint** in the Render dashboard and pick the repo.

## Option B: Dashboard

1. **New → Web Service → Build from a Docker image** (or "Existing image"
   `ghcr.io/hieudoanm/blazers:latest` — make the package public or connect GHCR)
2. Or **Build from repo** with runtime `Docker` if you want Render to build the
   `Dockerfile` itself.
3. Under **Disks**, add a disk mounted at `/data` (min 1 GB, paid plans only).
4. Env vars: `JWT_SECRET` (generate), `PORT=8080`.
5. Health check path: `/api/health`.

Render injects `PORT` automatically — Blazers reads it, so no extra config is
needed.

## Persistent data

The disk at `/data` survives deploys and restarts. Back up via `GET /api/backup`
(admin token) or Render's disk snapshots.

## Caveats

- Disks are single-instance — don't scale beyond 1 instance.
- Free/starter-without-disk instances lose all data on redeploy; the disk is
  required for anything real.
