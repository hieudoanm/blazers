# Deploy to AWS

Two realistic paths: **ECS Fargate + EFS** (fully managed, persistent) or a
plain **EC2/ Lightsail VM** running the container (simplest).

## ECS Fargate + EFS (recommended)

Blazers needs a real filesystem for SQLite — EFS provides the persistent `/data`
volume.

1. **Push the image to ECR**

   ```sh
   aws ecr create-repository --repository-name blazers
   aws ecr get-login-password | docker login --username AWS \
     --password-stdin <account>.dkr.ecr.<region>.amazonaws.com
   docker tag ghcr.io/hieudoanm/blazers:latest \
     <account>.dkr.ecr.<region>.amazonaws.com/blazers:latest
   docker push <account>.dkr.ecr.<region>.amazonaws.com/blazers:latest
   ```

   (Or build from source in CI and push — see [Terraform](with-terraform.md).)

2. **EFS** — create a filesystem, mount targets in your VPC subnets, and an
   access point rooted at `/data` with POSIX user `1000` (the `blazers` user's
   UID in the image — verify with `id blazers` in the container if needed).

3. **Task definition** — Fargate task, `awsvpc` network mode, one container:

   ```json
   {
     "family": "blazers",
     "networkMode": "awsvpc",
     "requiresCompatibilities": ["FARGATE"],
     "cpu": "256",
     "memory": "512",
     "containerDefinitions": [
       {
         "name": "blazers",
         "image": "<account>.dkr.ecr.<region>.amazonaws.com/blazers:latest",
         "portMappings": [{ "containerPort": 8080, "protocol": "tcp" }],
         "environment": [
           { "name": "JWT_SECRET", "value": "…" },
           { "name": "BLAZERS_DATA", "value": "/data" },
           { "name": "PORT", "value": "8080" }
         ],
         "mountPoints": [
           {
             "sourceVolume": "data",
             "containerPath": "/data",
             "readOnly": false
           }
         ],
         "healthCheck": {
           "command": [
             "CMD-SHELL",
             "wget -qO- http://localhost:8080/api/health || exit 1"
           ],
           "interval": 30,
           "timeout": 5,
           "retries": 3
         },
         "logConfiguration": {
           "logDriver": "awslogs",
           "options": {
             "awslogs-group": "/ecs/blazers",
             "awslogs-region": "<region>",
             "awslogs-stream-prefix": "blazers"
           }
         }
       }
     ],
     "volumes": [
       {
         "name": "data",
         "efsVolumeConfiguration": {
           "fileSystemId": "fs-…",
           "transitEncryption": "ENABLED",
           "authorizationConfig": {
             "accessPointId": "fsap-…",
             "iam": "ENABLED"
           }
         }
       }
     ]
   }
   ```

   The slim runtime has no `wget`/`curl` — either add a health check via the ALB
   target group (`/api/health`) instead of a container-level check, or install
   `wget` in a derived image.

4. **Service** — Fargate service behind an **ALB** (listener 443 → target group
   port 8080, health check path `/api/health`). WebSocket (`/ws`) and SSE work
   through ALB with no extra config; ensure idle timeout ≥ 60s for SSE.

## EC2 / Lightsail (simplest)

```sh
# on the instance (Docker installed)
docker run -d --name blazers --restart unless-stopped \
  -p 8080:8080 \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  -v blazers-data:/data \
  ghcr.io/hieudoanm/blazers:latest
```

The instance's own disk is the persistent volume — snapshot it for backups, or
schedule `GET /api/backup` to S3. Put Caddy/nginx or an ALB in front for TLS.

## App Runner (quick, but ephemeral)

App Runner deploys the container easily but has **no persistent disk** — same
caveat as Heroku. Fine for evaluation; pair with scheduled `/api/backup` → S3.

## IAM notes

The task role needs `elasticfilesystem:ClientMount`/`ClientWrite` on the access
point when `iam: ENABLED`; otherwise use access-point POSIX enforcement only.
