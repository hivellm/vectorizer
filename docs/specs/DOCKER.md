# Docker Deployment Guide

This guide covers deploying Vectorizer using Docker containers for both production and development environments.

## Quick Start

### Production Deployment

```bash
# Clone the repository
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer

# Build and start with Docker Compose
docker-compose up --build

# The service will be available at:
# - Unified Server: http://localhost:15002
# - REST API: http://localhost:15002
# - MCP Server: http://localhost:15002/mcp/sse
```

### Development Environment

```bash
# Start development container
docker-compose up vectorizer-dev

# This gives you a bash shell with development tools:
# - cargo-watch (for auto-rebuilds)
# - cargo-outdated (for dependency updates)
# - cargo-audit (for security checks)
# - vim, htop (for debugging)
```

## Docker Images

### Production Image (`Dockerfile`)

- **Base**: `rust:1.75-slim` (builder) → `debian:bookworm-slim` (runtime)
- **Size**: ~50MB (optimized multi-stage build)
- **Security**: Non-root user, minimal dependencies
- **Health Check**: Built-in health monitoring
- **Features**: All production features enabled

### Development Image (`Dockerfile.dev`)

- **Base**: `rust:1.75-slim`
- **Size**: ~200MB (includes development tools)
- **Tools**: cargo-watch, cargo-outdated, cargo-audit, vim, htop
- **User**: Interactive development user with sudo access
- **Features**: Full development environment

## Configuration

### Production Configuration (`config.docker.yml`)

```yaml
server:
  host: "0.0.0.0"  # Listen on all interfaces
  port: 15001

database:
  persistence:
    enabled: true
    path: "/app/data"  # Docker volume mount
    compression: true

auth:
  jwt:
    secret: "your-jwt-secret-key-change-in-production"
  api_keys:
    enabled: true
    rate_limit_per_minute: 100

mcp:
  enabled: true
  port: 15003
  max_connections: 50
```

### Environment Variables

You can override configuration using environment variables:

```bash
docker run -e RUST_LOG=debug -e SERVER_PORT=8080 vectorizer
```

## Docker Compose Services

### `vectorizer` (Production)

- **Purpose**: Production-ready service
- **Ports**: 15001, 15002, 15003
- **Volumes**: `./data:/app/data`, `./config.docker.yml:/app/config.yml`
- **Restart**: `unless-stopped`
- **Health Check**: Built-in monitoring

### `vectorizer-dev` (Development)

- **Purpose**: Development environment
- **Ports**: 15001, 15002, 15003
- **Volumes**: `.:/app` (live code reload), cargo cache
- **Interactive**: `stdin_open: true`, `tty: true`
- **Command**: `bash` (interactive shell)

## Building Custom Images

### Production Build

```bash
# Build with custom tag
docker build -t vectorizer:v0.6.0 .

# Build with specific features
docker build --build-arg FEATURES="onnx-models,real-models" -t vectorizer:full .
```

### Development Build

```bash
# Build development image
docker build -f Dockerfile.dev -t vectorizer-dev:latest .
```

## Volume Management

### Data Persistence

```bash
# Create named volume for data
docker volume create vectorizer-data

# Use named volume in docker-compose
volumes:
  - vectorizer-data:/app/data
```

### Configuration Management

```bash
# Mount custom config
docker run -v /path/to/config.yml:/app/config.yml vectorizer

# Use environment variables
docker run -e SERVER_HOST=0.0.0.0 -e SERVER_PORT=8080 vectorizer
```

## Health Monitoring

### Built-in Health Check

The production image includes a health check:

```dockerfile
HEALTHCHECK --interval=30s --timeout=15s --start-period=10s --retries=5 \
    CMD curl -f http://localhost:15001/health || exit 1
```

### External Monitoring

```bash
# Check container health
docker ps  # Shows health status

# View health check logs
docker inspect vectorizer | grep -A 10 Health

# Custom health check
docker run --health-cmd="curl -f http://localhost:15001/health" vectorizer
```

## Security Considerations

### Production Security

- **Non-root user**: Container runs as `vectorizer` user
- **Minimal dependencies**: Only essential packages installed
- **No shell access**: Production container has no interactive shell
- **Read-only filesystem**: Consider using `--read-only` flag

### Development Security

- **Sudo access**: Development user has sudo for debugging
- **Interactive shell**: Full shell access for development
- **Volume mounts**: Live code reloading enabled

## Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports 15001-15003 are available
2. **Permission issues**: Check volume mount permissions
3. **Memory limits**: Ensure sufficient memory for large datasets
4. **Network issues**: Verify Docker network configuration

### Debugging Commands

```bash
# View container logs
docker logs vectorizer

# Execute commands in running container
docker exec -it vectorizer bash

# Check resource usage
docker stats vectorizer

# Inspect container configuration
docker inspect vectorizer
```

### Performance Tuning

```bash
# Increase memory limit
docker run --memory=4g vectorizer

# Set CPU limits
docker run --cpus=2 vectorizer

# Enable swap
docker run --memory-swap=8g vectorizer
```

## Advanced Usage

### Multi-container Deployment

```yaml
version: '3.8'
services:
  vectorizer-api:
    build: .
    ports:
      - "15001:15001"
  
  vectorizer-mcp:
    build: .
    command: ["./vectorizer-server", "--mcp-only"]
    ports:
      - "15003:15003"
  
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    depends_on:
      - vectorizer-api
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vectorizer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: vectorizer
  template:
    metadata:
      labels:
        app: vectorizer
    spec:
      containers:
      - name: vectorizer
        image: vectorizer:latest
        ports:
        - containerPort: 15001
        env:
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: data
          mountPath: /app/data
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: vectorizer-pvc
```

## Best Practices

1. **Use specific tags**: Avoid `latest` in production
2. **Regular updates**: Keep base images updated
3. **Resource limits**: Set appropriate memory/CPU limits
4. **Health checks**: Monitor container health
5. **Logging**: Configure proper log levels
6. **Backups**: Regular data volume backups
7. **Security**: Regular security scans with `cargo audit`
