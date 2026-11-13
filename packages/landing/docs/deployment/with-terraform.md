# Deploy with Terraform

Blazers is a container + a persistent volume + env vars — a small Terraform
surface on every cloud. Below is a complete **AWS ECS Fargate + EFS** example;
the same shape maps to Azure Container Apps and GCE.

## AWS: ECS Fargate + EFS

```hcl
terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

provider "aws" { region = "us-east-1" }

resource "aws_ecr_repository" "blazers" {
  name = "blazers"
}

# Push ghcr.io/hieudoanm/blazers:latest to ECR once (see aws.md), or build
# in CI and reference the image URI directly.

resource "aws_efs_file_system" "data" {
  creation_token = "blazers-data"
  encrypted      = true
}

resource "aws_efs_mount_target" "data" {
  count           = length(var.subnet_ids)
  file_system_id  = aws_efs_file_system.data.id
  subnet_id       = var.subnet_ids[count.index]
  security_groups = [aws_security_group.efs.id]
}

resource "aws_ecs_cluster" "blazers" { name = "blazers" }

resource "aws_iam_role" "task" {
  name = "blazers-task"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ecs-tasks.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "task_exec" {
  role       = aws_iam_role.task.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_security_group" "efs" {
  name   = "blazers-efs"
  vpc_id = var.vpc_id
  ingress {
    from_port   = 2049
    to_port     = 2049
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"] # your VPC CIDR
  }
}

resource "aws_ecs_task_definition" "blazers" {
  family                   = "blazers"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.task.arn
  task_role_arn            = aws_iam_role.task.arn

  volume {
    name = "data"
    efs_volume_configuration {
      file_system_id     = aws_efs_file_system.data.id
      transit_encryption = "ENABLED"
    }
  }

  container_definitions = jsonencode([{
    name      = "blazers"
    image     = "${aws_ecr_repository.blazers.repository_url}:latest"
    essential = true
    portMappings = [{ containerPort = 8080 }]
    environment = [
      { name = "JWT_SECRET", value = var.jwt_secret },
      { name = "BLAZERS_DATA", value = "/data" },
      { name = "PORT", value = "8080" }
    ]
    mountPoints = [{
      sourceVolume  = "data"
      containerPath = "/data"
    }]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/blazers"
        "awslogs-region"        = "us-east-1"
        "awslogs-stream-prefix" = "blazers"
      }
    }
  }])
}

resource "aws_ecs_service" "blazers" {
  name            = "blazers"
  cluster         = aws_ecs_cluster.blazers.id
  task_definition = aws_ecs_task_definition.blazers.arn
  desired_count   = 1
  launch_type     = "FARGATE"

  network_configuration {
    subnets         = var.subnet_ids
    security_groups = [aws_security_group.service.id]
    assign_public_ip = true
  }
}
```

Variables: `vpc_id`, `subnet_ids`, `jwt_secret` (`TF_VAR_jwt_secret`, mark it
`sensitive`). Add an ALB + target group with health check `/api/health` in front
— see [aws.md](iaas-aws.md) for the task-definition details.

## Azure: Container Apps

```hcl
resource "azurerm_resource_group" "blazers" {
  name     = "rg-blazers"
  location = "East US"
}

resource "azurerm_storage_account" "data" {
  name                     = "stblazers${random_string.suffix.result}"
  resource_group_name      = azurerm_resource_group.blazers.name
  location                 = azurerm_resource_group.blazers.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
}

resource "azurerm_storage_share" "data" {
  name                 = "blazers-data"
  storage_account_name = azurerm_storage_account.data.name
  quota                = 1
}

resource "azurerm_container_app_environment" "blazers" {
  name                       = "cae-blazers"
  location                   = azurerm_resource_group.blazers.location
  resource_group_name        = azurerm_resource_group.blazers.name
  infrastructure_subnet_id   = azurerm_subnet.apps.id
  internal_load_balancer_enabled = false
}

resource "azurerm_container_app" "blazers" {
  name                         = "blazers"
  container_app_environment_id = azurerm_container_app_environment.blazers.id
  resource_group_name          = azurerm_resource_group.blazers.name
  revision_mode                = "Single"

  template {
    min_replicas = 1
    max_replicas = 1

    container {
      name   = "blazers"
      image  = "ghcr.io/hieudoanm/blazers:latest"
      cpu    = 0.5
      memory = "1Gi"

      env {
        name  = "JWT_SECRET"
        value = var.jwt_secret
      }
      env { name = "BLAZERS_DATA"; value = "/data" }
      env { name = "PORT";        value = "8080" }

      volume_mounts {
        name = "data"
        path = "/data"
      }
    }

    volume {
      name         = "data"
      storage_type = "AzureFile"
      azure_file {
        share_name     = azurerm_storage_share.data.name
        account_name   = azurerm_storage_account.data.name
        account_key    = azurerm_storage_account.data.primary_access_key
        access_mode    = "ReadWrite"
      }
    }
  }

  ingress {
    external_enabled = true
    target_port      = 8080
    transport        = "http"
  }
}
```

## GCP: Compute Engine

The simplest GCP path is a VM + Docker — Terraform is a
`google_compute_instance` with a startup script:

```hcl
resource "google_compute_instance" "blazers" {
  name         = "blazers"
  machine_type = "e2-small"
  zone         = "us-central1-a"

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
      size  = 20
    }
  }

  network_interface {
    network = "default"
    access_config {} # ephemeral external IP; use Cloud NAT for private
  }

  metadata_startup_script = <<-EOT
    #!/bin/bash
    apt-get update && apt-get install -y docker.io
    systemctl enable --now docker
    docker run -d --name blazers --restart unless-stopped \
      -p 8080:8080 \
      -e JWT_SECRET="${var.jwt_secret}" \
      -v blazers-data:/data \
      ghcr.io/hieudoanm/blazers:latest
  EOT
}
```

For Cloud Run there is no persistent disk — see [GCP](iaas-gcp.md) for why that
path is eval-only.

## General notes

- **One instance.** Blazers is single-node by design (SQLite + in-memory
  realtime hubs). `min_replicas`/`desired_count` stay at 1 everywhere.
- **Secrets**: pass `JWT_SECRET`/`BLAZERS_SECRETS_KEY` via the platform's secret
  store (SSM/Key Vault/Secret Manager), never commit them.
- **Health check**: `/api/health` on the load balancer/ingress.
