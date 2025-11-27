# Helm Chart Deployment Guide

Complete guide for deploying Vectorizer using Helm charts.

## Overview

The Vectorizer Helm chart provides a production-ready deployment for Kubernetes with:

- StatefulSet for persistent storage
- ConfigMap for configuration management
- Service for load balancing
- Ingress support
- Prometheus ServiceMonitor support
- Horizontal Pod Autoscaler support

## Quick Start

```bash
# Install from local chart
helm install vectorizer ./helm/vectorizer

# Install with custom values
helm install vectorizer ./helm/vectorizer -f my-values.yaml

# Install with persistence
helm install vectorizer ./helm/vectorizer \
  --set persistence.enabled=true \
  --set persistence.size=100Gi
```

## Configuration

### Basic Values

```yaml
# values.yaml
replicaCount: 1

image:
  repository: vectorizer
  tag: "1.3.0"

resources:
  limits:
    cpu: 4
    memory: 8Gi
  requests:
    cpu: 2
    memory: 4Gi
```

### Production Values

```yaml
# production-values.yaml
replicaCount: 3

image:
  repository: vectorizer
  tag: "1.3.0"

resources:
  limits:
    cpu: 8
    memory: 16Gi
  requests:
    cpu: 4
    memory: 8Gi

persistence:
  enabled: true
  storageClass: "fast-ssd"
  size: 500Gi

config:
  logging:
    level: "warn"
  performance:
    cpu:
      max_threads: 8
      memory_pool_size_mb: 4096

replication:
  enabled: true
  role: "master"

monitoring:
  enabled: true
  serviceMonitor:
    enabled: true

ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: vectorizer.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: vectorizer-tls
      hosts:
        - vectorizer.example.com
```

## Advanced Configuration

### Multi-Replica with Replication

```yaml
# Master
replicaCount: 1
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"

# Replicas
replicaCount: 2
replication:
  enabled: true
  role: "replica"
  master_address: "vectorizer-master:7001"
```

### Autoscaling

```yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
  targetMemoryUtilizationPercentage: 80
```

### Node Affinity

```yaml
affinity:
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
        - matchExpressions:
            - key: node-type
              operator: In
              values:
                - vector-db
```

## Deployment Examples

### Development

```bash
helm install vectorizer-dev ./helm/vectorizer \
  --set replicaCount=1 \
  --set resources.limits.cpu=2 \
  --set resources.limits.memory=4Gi \
  --set persistence.enabled=false \
  --set config.logging.level=debug
```

### Production

```bash
helm install vectorizer-prod ./helm/vectorizer \
  -f production-values.yaml \
  --namespace vectorizer \
  --create-namespace
```

### High Availability

```bash
# Master
helm install vectorizer-master ./helm/vectorizer \
  --set replicaCount=1 \
  --set replication.enabled=true \
  --set replication.role=master

# Replicas
helm install vectorizer-replica ./helm/vectorizer \
  --set replicaCount=3 \
  --set replication.enabled=true \
  --set replication.role=replica \
  --set replication.master_address=vectorizer-master:7001
```

## Upgrading

```bash
# Upgrade to new version
helm upgrade vectorizer ./helm/vectorizer

# Upgrade with new values
helm upgrade vectorizer ./helm/vectorizer -f new-values.yaml

# Rollback
helm rollback vectorizer
```

## Uninstalling

```bash
# Uninstall (keeps PVCs)
helm uninstall vectorizer

# Uninstall with PVC cleanup
helm uninstall vectorizer
kubectl delete pvc -l app.kubernetes.io/name=vectorizer
```

## Troubleshooting

### Check Release Status

```bash
helm status vectorizer
```

### View Values

```bash
helm get values vectorizer
```

### Debug Template

```bash
helm template vectorizer ./helm/vectorizer --debug
```

### Check Resources

```bash
# Pods
kubectl get pods -l app.kubernetes.io/name=vectorizer

# Services
kubectl get svc -l app.kubernetes.io/name=vectorizer

# ConfigMaps
kubectl get configmap -l app.kubernetes.io/name=vectorizer

# PVCs
kubectl get pvc -l app.kubernetes.io/name=vectorizer
```

## Best Practices

1. **Use StatefulSet**: Enable persistence for production
2. **Set Resource Limits**: Prevent resource exhaustion
3. **Enable Monitoring**: Use ServiceMonitor for Prometheus
4. **Use Ingress**: For external access with TLS
5. **Enable Autoscaling**: For variable workloads
6. **Configure Replication**: For high availability
7. **Use Values Files**: Separate dev/staging/prod values
8. **Version Control**: Track values files in Git

## Related Documentation

- [Kubernetes Deployment Guide](./KUBERNETES.md)
- [Production Guide](../PRODUCTION_GUIDE.md)
- [Helm Chart README](../../helm/vectorizer/README.md)
