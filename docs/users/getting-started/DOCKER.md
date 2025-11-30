---
title: Docker Installation
module: installation
id: docker-installation
order: 2
description: Complete Docker installation and deployment guide
tags: [installation, docker, container, deployment]
---

# Docker Installation

Complete guide to installing and deploying Vectorizer using Docker.

## Quick Start

### Basic Docker Run

**Without persistence (data lost on container stop):**

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest
```

**With persistent data:**

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -v $(pwd)/vectorizer-storage:/vectorizer/storage \
  -v $(pwd)/vectorizer-snapshots:/vectorizer/snapshots \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest
```

## Docker Compose

### Basic docker-compose.yml

```yaml
version: "3.8"

services:
  vectorizer:
    image: ghcr.io/hivellm/vectorizer:latest
    container_name: vectorizer
    ports:
      - "15002:15002" # REST API + MCP
      - "15003:15003" # Additional port

    volumes:
      # Persistent data
      - ./vectorizer-data:/vectorizer/data
      - ./vectorizer-storage:/vectorizer/storage
      - ./vectorizer-snapshots:/vectorizer/snapshots
      - ./vectorizer-dashboard:/vectorizer/dashboard

    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - VECTORIZER_LOG_LEVEL=info

    restart: unless-stopped

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:15002/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

    # Resource limits
    deploy:
      resources:
        limits:
          cpus: "4.0"
          memory: 4G
        reservations:
          cpus: "2.0"
          memory: 2G
```

**Start with docker-compose:**

```bash
docker-compose up -d
```

**View logs:**

```bash
docker-compose logs -f vectorizer
```

**Stop:**

```bash
docker-compose down
```

### Production docker-compose.yml

```yaml
version: "3.8"

services:
  vectorizer:
    image: ghcr.io/hivellm/vectorizer:latest
    container_name: vectorizer
    ports:
      - "15002:15002"

    volumes:
      # Use named volumes for better management
      - vectorizer-data:/vectorizer/data
      - vectorizer-storage:/vectorizer/storage
      - vectorizer-snapshots:/vectorizer/snapshots

      # Mount configuration file
      - ./config.yml:/etc/vectorizer/config.yml:ro

    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - VECTORIZER_LOG_LEVEL=info
      - TZ=UTC

    restart: always

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:15002/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

    deploy:
      resources:
        limits:
          cpus: "4.0"
          memory: 4G
        reservations:
          cpus: "2.0"
          memory: 2G

    # Security options
    security_opt:
      - no-new-privileges:true
    read_only: false # Set to true if using tmpfs for writable dirs
    tmpfs:
      - /tmp

volumes:
  vectorizer-data:
    driver: local
  vectorizer-storage:
    driver: local
  vectorizer-snapshots:
    driver: local
```

### Development docker-compose.yml

```yaml
version: "3.8"

services:
  vectorizer-dev:
    build:
      context: .
      dockerfile: Dockerfile
      args:
        - PROFILE=release
        - FEATURES=

    container_name: vectorizer-dev
    ports:
      - "15002:15002"
      - "15003:15003"

    volumes:
      # Persistent data
      - ./vectorizer-data:/vectorizer/data
      - ./vectorizer-storage:/vectorizer/storage
      - ./vectorizer-snapshots:/vectorizer/snapshots

      # Mount workspace for development
      - ../../:/workspace:ro

      # Mount logs for debugging
      - ./logs:/vectorizer/.logs

    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - VECTORIZER_LOG_LEVEL=debug
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
      - RUN_MODE=development

    restart: unless-stopped

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:15002/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

    deploy:
      resources:
        limits:
          cpus: "8.0"
          memory: 8G
        reservations:
          cpus: "4.0"
          memory: 4G
```

## Building Docker Image

### Build from Source

**Basic build:**

```bash
docker build -t vectorizer:latest .
```

**Build with specific features:**

```bash
docker build \
  --build-arg FEATURES="hive-gpu" \
  -t vectorizer:gpu .
```

**Build for specific platform:**

```bash
docker buildx build \
  --platform linux/amd64 \
  -t vectorizer:latest .
```

### Multi-stage Build

The Dockerfile uses multi-stage builds for smaller images:

```dockerfile
# Stage 1: Build
FROM rust:1.90 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/vectorizer /usr/local/bin/
EXPOSE 15002
CMD ["vectorizer"]
```

## Docker Volumes

### Named Volumes

**Create named volumes:**

```bash
docker volume create vectorizer-data
docker volume create vectorizer-storage
docker volume create vectorizer-snapshots
```

**Use in docker-compose:**

```yaml
volumes:
  vectorizer-data:
    external: true
  vectorizer-storage:
    external: true
```

**List volumes:**

```bash
docker volume ls
```

**Inspect volume:**

```bash
docker volume inspect vectorizer-data
```

**Remove volume (⚠️ deletes data):**

```bash
docker volume rm vectorizer-data
```

### Bind Mounts

**Host directory mount:**

```yaml
volumes:
  - ./vectorizer-data:/vectorizer/data
  - /var/lib/vectorizer:/vectorizer/data # Absolute path
```

**Read-only mount:**

```yaml
volumes:
  - ./config.yml:/etc/vectorizer/config.yml:ro
```

## Environment Variables

### Common Environment Variables

```yaml
environment:
  - VECTORIZER_HOST=0.0.0.0
  - VECTORIZER_PORT=15002
  - VECTORIZER_MCP_PORT=15003
  - VECTORIZER_LOG_LEVEL=info
  - VECTORIZER_DATA_DIR=/vectorizer/data
  - VECTORIZER_WORKERS=8
  - TZ=UTC
  - RUST_LOG=info
```

### Using .env File

**Create .env file:**

```bash
VECTORIZER_HOST=0.0.0.0
VECTORIZER_PORT=15002
VECTORIZER_LOG_LEVEL=info
```

**Use in docker-compose:**

```yaml
services:
  vectorizer:
    env_file:
      - .env
```

## Networking

### Default Bridge Network

**Create custom network:**

```bash
docker network create vectorizer-net
```

**Use in docker-compose:**

```yaml
services:
  vectorizer:
    networks:
      - vectorizer-net

networks:
  vectorizer-net:
    driver: bridge
```

### Port Mapping

**Map single port:**

```yaml
ports:
  - "15002:15002"
```

**Map multiple ports:**

```yaml
ports:
  - "15002:15002"
  - "15003:15003"
```

**Map to different host port:**

```yaml
ports:
  - "8080:15002" # Host:Container
```

**Bind to specific interface:**

```bash
docker run -p 127.0.0.1:15002:15002 vectorizer
```

## Health Checks

### Basic Health Check

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:15002/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

### Advanced Health Check

```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:15002/health || exit 1"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 60s
```

**Check health status:**

```bash
docker ps  # Shows health status
docker inspect vectorizer | grep -A 10 Health
```

## Resource Limits

### CPU Limits

```yaml
deploy:
  resources:
    limits:
      cpus: "4.0" # Maximum 4 CPUs
    reservations:
      cpus: "2.0" # Guaranteed 2 CPUs
```

### Memory Limits

```yaml
deploy:
  resources:
    limits:
      memory: 4G # Maximum 4GB
    reservations:
      memory: 2G # Guaranteed 2GB
```

### Complete Resource Configuration

```yaml
deploy:
  resources:
    limits:
      cpus: "4.0"
      memory: 4G
    reservations:
      cpus: "2.0"
      memory: 2G
```

## Logging

### View Logs

**Follow logs:**

```bash
docker logs -f vectorizer
```

**Last 100 lines:**

```bash
docker logs --tail 100 vectorizer
```

**Since timestamp:**

```bash
docker logs --since 2024-11-16T10:00:00 vectorizer
```

### Log Driver Configuration

**JSON file log driver:**

```yaml
services:
  vectorizer:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

**Syslog driver:**

```yaml
services:
  vectorizer:
    logging:
      driver: "syslog"
      options:
        syslog-address: "tcp://localhost:514"
```

## Backup and Restore

### Backup Container Data

**Backup volumes:**

```bash
# Stop container
docker-compose stop vectorizer

# Backup data directory
docker run --rm \
  -v vectorizer-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/vectorizer-backup-$(date +%Y%m%d).tar.gz -C /data .

# Start container
docker-compose start vectorizer
```

### Restore Container Data

**Restore from backup:**

```bash
# Stop container
docker-compose stop vectorizer

# Restore data
docker run --rm \
  -v vectorizer-data:/data \
  -v $(pwd):/backup \
  alpine sh -c "cd /data && tar xzf /backup/vectorizer-backup-20241116.tar.gz"

# Start container
docker-compose start vectorizer
```

## Troubleshooting

### Container Won't Start

**Check logs:**

```bash
docker logs vectorizer
```

**Check container status:**

```bash
docker ps -a | grep vectorizer
```

**Inspect container:**

```bash
docker inspect vectorizer
```

### Port Already in Use

**Find process using port:**

```bash
# Linux
sudo lsof -i :15002

# Docker
docker ps | grep 15002
```

**Use different port:**

```yaml
ports:
  - "15012:15002" # Use host port 15012
```

### Permission Issues

**Fix volume permissions:**

```bash
docker run --rm \
  -v vectorizer-data:/data \
  alpine chown -R 1000:1000 /data
```

### Out of Memory

**Increase memory limit:**

```yaml
deploy:
  resources:
    limits:
      memory: 8G # Increase from 4G
```

## Best Practices

1. **Use named volumes** for production data
2. **Set resource limits** to prevent resource exhaustion
3. **Enable health checks** for automatic recovery
4. **Use restart policies** (`always` or `unless-stopped`)
5. **Mount config files** as read-only
6. **Use .env files** for environment variables
7. **Regular backups** of volumes
8. **Monitor logs** for issues
9. **Use specific image tags** instead of `latest` in production
10. **Keep images updated** for security patches

## Related Topics

- [Installation Guide](./INSTALLATION.md) - General installation guide
- [Configuration Guide](../configuration/SERVER.md) - Server configuration
- [Service Management](../operations/SERVICE_MANAGEMENT.md) - Service management
