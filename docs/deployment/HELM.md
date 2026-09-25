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

It deploys `ghcr.io/hivellm/vectorizer:3.8.2` by default (public GHCR
package, no pull secret needed) and probes `/health` (liveness) and `/ready`
(readiness).

> **High Availability:** the chart runs standalone servers — it does not
> render the Raft cluster configuration (node ids, peer list, config
> template init container, auth secrets). For a 3-pod Raft cluster with
> automatic failover, use the manifests in `deploy/k8s/` and the
> [HA on Kubernetes — End-to-End Runbook](./HA_KUBERNETES_RUNBOOK.md).

## Quick Start

```bash
# Install from local chart
helm install vectorizer ./deploy/helm/vectorizer

# Install with custom values
helm install vectorizer ./deploy/helm/vectorizer -f my-values.yaml

# Install with persistence
helm install vectorizer ./deploy/helm/vectorizer \
  --set persistence.enabled=true \
  --set persistence.size=100Gi
```

## Configuration

> **Known limitation (chart 1.6.0):** the chart mounts its rendered
> `config.yml` under `/etc/vectorizer/`, but the server reads `config.yml`
> from its working directory (`/vectorizer/config.yml`). Of the `config.*`
> values, only `config.logging.level` (passed as `RUST_LOG`) and
> `config.server.data_dir` (the PVC mount path; keep it at `/data`, the
> image's `VECTORIZER_DATA_DIR`) take effect today.

### Basic Values

```yaml
# values.yaml
replicaCount: 1

image:
  repository: ghcr.io/hivellm/vectorizer
  tag: "3.8.2"            # -fastembed (dense/multilingual): use tag "3.8.2-fastembed"

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
replicaCount: 1           # one standalone server; see the HA runbook for a cluster

image:
  repository: ghcr.io/hivellm/vectorizer
  tag: "3.8.2"            # -fastembed (dense/multilingual): use tag "3.8.2-fastembed"

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
helm install vectorizer-dev ./deploy/helm/vectorizer \
  --set replicaCount=1 \
  --set resources.limits.cpu=2 \
  --set resources.limits.memory=4Gi \
  --set persistence.enabled=false \
  --set config.logging.level=debug
```

### Production

```bash
helm install vectorizer-prod ./deploy/helm/vectorizer \
  -f production-values.yaml \
  --namespace vectorizer \
  --create-namespace
```

## Upgrading

Bump the image by setting `image.tag` (e.g. `--set image.tag=3.8.0`).

```bash
# Upgrade to new version
helm upgrade vectorizer ./deploy/helm/vectorizer

# Upgrade with new values
helm upgrade vectorizer ./deploy/helm/vectorizer -f new-values.yaml

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
helm template vectorizer ./deploy/helm/vectorizer --debug
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

1. **Pin the image tag**: `image.tag: "3.8.2"`, never `latest`
2. **Use StatefulSet**: Enable persistence for production
3. **Set Resource Limits**: Prevent resource exhaustion
4. **Enable Monitoring**: Use ServiceMonitor for Prometheus
5. **Use Ingress**: For external access with TLS
6. **Use Values Files**: Separate dev/staging/prod values
7. **Version Control**: Track values files in Git
8. **For HA**: use the [HA runbook](./HA_KUBERNETES_RUNBOOK.md) manifests instead of the chart

## Related Documentation

- [Kubernetes Deployment Guide](./KUBERNETES.md)
- [HA on Kubernetes — End-to-End Runbook](./HA_KUBERNETES_RUNBOOK.md)
- [Production Guide](./PRODUCTION_GUIDE.md)
- [Helm Chart README](../../deploy/helm/vectorizer/README.md)
