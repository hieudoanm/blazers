# Deploy to GCP

**Compute Engine** is the straightforward fit (persistent disk + Docker).
**Cloud Run** is the quickest but has no persistent filesystem — same caveat as
Heroku/App Runner.

## Compute Engine (recommended)

```sh
# e2-small is plenty for a single-node BaaS
gcloud compute instances create blazers \
  --zone=us-central1-a --machine-type=e2-small \
  --image-family=debian-12 --image-project=debian-cloud \
  --boot-disk-size=20GB

gcloud compute ssh blazers --zone=us-central1-a
# on the VM:
sudo apt-get update && sudo apt-get install -y docker.io
sudo usermod -aG docker $USER   # re-login

docker run -d --name blazers --restart unless-stopped \
  -p 8080:8080 \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  -v blazers-data:/data \
  ghcr.io/hieudoanm/blazers:latest
```

The boot disk is persistent — snapshot it (`gcloud compute snapshots create`) or
schedule `GET /api/backup` uploads to GCS for backups. Add a managed
certificate + HTTPS load balancer (or Caddy on the VM) for TLS.

For higher availability, put the VM in an instance group behind a load balancer
and use Filestore NFS for `/data` — though SQLite on NFS is workable, a single
node is the intended deployment model.

## Cloud Run (quick, ephemeral)

```sh
gcloud run deploy blazers \
  --image ghcr.io/hieudoanm/blazers:latest \
  --region us-central1 \
  --port 8080 \
  --set-env-vars JWT_SECRET="$(openssl rand -hex 32)" \
  --allow-unauthenticated
```

Cloud Run's filesystem is in-memory and reset per instance — **data does not
survive**. Acceptable for evaluation only; pair with scheduled `/api/backup` →
GCS bucket + restore-on-boot if you must.

You can mount a GCS bucket via Cloud Storage FUSE, but **do not** put SQLite on
it — FUSE lacks the locking/mmap semantics SQLite needs.

## GKE alternative

Standard Kubernetes: the `ghcr.io` image + a `PersistentVolumeClaim` (`pd-ssd`
or Filestore) mounted at `/data`, `readinessProbe` on `/api/health`. See
[Terraform](with-terraform.md) for the `google` provider.
