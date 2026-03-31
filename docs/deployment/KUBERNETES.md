# Kubernetes Deployment Guide

Complete guide for deploying Vectorizer on Kubernetes.

## Prerequisites

- Kubernetes cluster (1.20+)
- kubectl configured
- PersistentVolume provisioner
- Ingress controller (optional)

## Quick Start

```bash
# Deploy Vectorizer
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/statefulset.yaml
kubectl apply -f k8s/service.yaml

# Check status
kubectl get pods -n vectorizer
kubectl logs -f -n vectorizer vectorizer-0
```

## Manifests

### Namespace

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: vectorizer
```

### ConfigMap

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: vectorizer-config
  namespace: vectorizer
data:
  config.yml: |
    server:
      host: "0.0.0.0"
      port: 15002
      data_dir: "/data"

    logging:
      level: "warn"
      format: "json"

    performance:
      cpu:
        max_threads: 8
        memory_pool_size_mb: 2048
```

### StatefulSet

See [k8s/statefulset.yaml](./k8s/statefulset.yaml)

### Service

See [k8s/service.yaml](./k8s/service.yaml)

## Configuration

### Resource Limits

Adjust based on your workload:

```yaml
resources:
  requests:
    cpu: "4"
    memory: "8Gi"
  limits:
    cpu: "8"
    memory: "16Gi"
```

### Storage

Configure PersistentVolumeClaim:

```yaml
volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: "fast-ssd"
      resources:
        requests:
          storage: 100Gi
```

### High Availability with Raft (v2.5.4+)

For automatic failover, enable Raft consensus. All pods use the **same config template** — Raft elects a leader dynamically. No static master/replica roles needed.

```yaml
# ConfigMap uses __NODE_ID__ placeholder replaced by init container
cluster:
  enabled: true                # Enables Raft leader election
  node_id: "__NODE_ID__"       # Replaced per-pod by init container
  discovery: "dns"
  dns_name: "vectorizer-headless.<namespace>.svc.cluster.local"
  dns_grpc_port: 15003

replication:
  enabled: true
  bind_address: "0.0.0.0:7001"  # Role managed by Raft, not config

file_watcher:
  enabled: false                 # MUST be false in cluster mode
```

The init container injects each pod's hostname as `node_id`:
```yaml
initContainers:
  - name: config-selector
    image: busybox:1.36
    command: ["sh", "-c"]
    args:
      - sed "s/__NODE_ID__/$HOSTNAME/g" /configs/config-template.yml > /active-config/config.yml
```

Required environment variables for cluster DNS:
```yaml
env:
  - name: HOSTNAME
    valueFrom:
      fieldRef:
        fieldPath: metadata.name
  - name: POD_IP
    valueFrom:
      fieldRef:
        fieldPath: status.podIP
  - name: VECTORIZER_SERVICE_NAME
    value: "vectorizer-headless.<namespace>.svc.cluster.local"
```

Required services:
- **Headless service** (`clusterIP: None`) with ports 15002, 7001, 15003
- **ClusterIP service** for external access on port 15002

See `k8s/configmap-ha.yaml` and `k8s/statefulset-ha.yaml` for complete production manifests.

See [HA Cluster Guide](../users/guides/HA_CLUSTER.md) for detailed configuration and failover behavior.

## Scaling

### Horizontal Scaling

```bash
# Scale StatefulSet
kubectl scale statefulset vectorizer --replicas=3 -n vectorizer
```

### Vertical Scaling

Update resource limits in StatefulSet:

```bash
kubectl edit statefulset vectorizer -n vectorizer
```

## Monitoring

### ServiceMonitor (Prometheus Operator)

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: vectorizer
  namespace: vectorizer
spec:
  selector:
    matchLabels:
      app: vectorizer
  endpoints:
    - port: http
      path: /prometheus/metrics
```

## Ingress

### Basic Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: vectorizer
  namespace: vectorizer
spec:
  rules:
    - host: vectorizer.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: vectorizer
                port:
                  number: 15002
```

### TLS Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: vectorizer
  namespace: vectorizer
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
    - hosts:
        - vectorizer.example.com
      secretName: vectorizer-tls
  rules:
    - host: vectorizer.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: vectorizer
                port:
                  number: 15002
```

## Helm Chart

See [helm/vectorizer/](./helm/vectorizer/) for Helm chart.

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod vectorizer-0 -n vectorizer

# Check logs
kubectl logs vectorizer-0 -n vectorizer

# Check events
kubectl get events -n vectorizer --sort-by='.lastTimestamp'
```

### Storage Issues

```bash
# Check PVC status
kubectl get pvc -n vectorizer

# Check PV status
kubectl get pv
```

### Network Issues

```bash
# Check service
kubectl get svc vectorizer -n vectorizer

# Test connectivity
kubectl run -it --rm debug --image=busybox --restart=Never -- curl http://vectorizer:15002/api/status
```

### Cluster / HA Issues

**Replicas stuck on "No route to host" after pod restart:**
- **v2.5.4+**: Fixed. Replicas re-resolve DNS on every reconnect attempt.
- **Older versions**: The master IP was cached at startup. Upgrade to v2.5.4.

**Pods crash with "Path segments must not start with `:`":**
- **v2.5.4+**: Fixed. Cluster router path syntax was corrected for Axum 0.7.

**No automatic failover when leader dies:**
- Ensure `cluster.enabled: true` in your ConfigMap. Without Raft, roles are static and no election occurs.

**File watcher warnings in cluster mode:**
- Set `file_watcher.enabled: false`. It is incompatible with distributed clusters.

See [HA Cluster Guide](../users/guides/HA_CLUSTER.md) for complete troubleshooting.

## Best Practices

1. **Use StatefulSet**: For persistent storage and stable pod identity
2. **Enable Raft** (`cluster.enabled: true`): For automatic failover
3. **Disable file watcher**: `file_watcher.enabled: false` in cluster mode
4. **Use headless service**: Required for pod-to-pod DNS discovery
5. **Set Resource Limits**: Prevent resource exhaustion
6. **Enable Health Checks**: Liveness and readiness probes
7. **Use ConfigMap templates**: With `__NODE_ID__` placeholder and init container
8. **Set `HOSTNAME`, `POD_IP`, `VECTORIZER_SERVICE_NAME`** env vars from K8s downward API
9. **Expose all 3 ports**: REST (15002), replication (7001), gRPC (15003)
10. **Enable Monitoring**: Prometheus metrics
11. **Use Secrets**: For JWT secret and credentials (not in ConfigMap)
