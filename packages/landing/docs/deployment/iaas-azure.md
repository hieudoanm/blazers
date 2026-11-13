# Deploy to Azure

**Azure Container Apps** is the best fit: managed containers with native support
for **Azure Files** mounts — persistent storage for `/data` without managing
VMs.

## Azure Container Apps + Azure Files

### 1. Resource group + storage

```sh
az group create -n rg-blazers -l eastus
az storage account create -n stblazers$RANDOM -g rg-blazers \
  --sku Standard_LRS
az storage share create -n blazers-data \
  --account-name <storage-account> --quota 1
```

### 2. Container App

```sh
az containerapp env create -n cae-blazers -g rg-blazers \
  --location eastus

az containerapp create -n blazers -g rg-blazers \
  --environment cae-blazers \
  --image ghcr.io/hieudoanm/blazers:latest \
  --target-port 8080 \
  --ingress external \
  --min-replicas 1 --max-replicas 1 \
  --cpu 0.5 --memory 1Gi \
  --env-vars JWT_SECRET="$(openssl rand -hex 32)" PORT=8080 BLAZERS_DATA=/data \
  --volumes blazers-data=/data \
  --volume-mounts /data=blazers-data
```

For the volume you must first create an **Azure Files-backed volume** on the
environment and grant the environment's managed identity
`Storage File Data Privileged Contributor` on the share — the portal flow
(Container App → Volumes → Azure Files) walks through it; the CLI equivalent
uses `az containerapp env storage set`.

Alternatively deploy from source with the repo `Dockerfile`:

```sh
az containerapp up -n blazers -g rg-blazers \
  --environment cae-blazers --source . \
  --target-port 8080 --ingress external
```

(`az containerapp up` builds the Dockerfile in Azure and can attach storage
interactively.)

## Health & TLS

- Ingress terminates TLS automatically on the `*.azurecontainerapps.io` domain;
  map a custom domain if needed.
- Health probe: set the readiness path to `/api/health` in the portal or via
  `--probes` in Bicep/Terraform.

## AKS / VM alternative

For anything beyond a single instance, run the image on **AKS** with an Azure
Disk (RWO) or **Azure Files** (RWX) PVC mounted at `/data` — standard Kubernetes
`PersistentVolumeClaim` + the `ghcr.io` image.

## Terraform

See [Terraform](with-terraform.md) — the `azurerm` provider covers
`azurerm_container_app`, `azurerm_storage_share`, and the environment in one
apply.
