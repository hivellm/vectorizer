# Deployment Guides

Complete deployment guides for Vectorizer in various environments.

## Contents

- [Kubernetes Deployment](./KUBERNETES.md) - Complete Kubernetes deployment guide
- [Docker Compose Production](./docker-compose.production.yml) - Production Docker Compose example
- [Nginx Reverse Proxy](./nginx.conf) - Nginx configuration for reverse proxy

## Quick Links

- [Production Guide](../PRODUCTION_GUIDE.md) - Complete production deployment guide
- [Monitoring Setup](../MONITORING_SETUP.md) - Monitoring and alerting setup
- [Backup & Recovery](../BACKUP_RECOVERY.md) - Backup and recovery procedures
- [Runbooks](../runbooks/) - Operational runbooks

## Deployment Options

### Kubernetes

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/statefulset.yaml
kubectl apply -f k8s/service.yaml
```

See [Kubernetes Deployment Guide](./KUBERNETES.md) for details.

### Docker Compose

```bash
# Deploy with Docker Compose
docker-compose -f docs/deployment/docker-compose.production.yml up -d
```

### Systemd Service

See [Service Management Guide](../../users/operations/SERVICE_MANAGEMENT.md).

## Prerequisites

- Kubernetes cluster (1.20+) or Docker/Docker Compose
- Persistent storage configured
- Network access configured
- SSL/TLS certificates (for production)

## Configuration

All deployment options support configuration via:
- Environment variables
- ConfigMap (Kubernetes)
- Configuration files
- Command-line arguments

See [Configuration Guide](../../users/configuration/CONFIGURATION.md) for details.

