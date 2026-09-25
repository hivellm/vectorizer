# Kubernetes Deployment Guide

Complete guide for deploying Vectorizer on Kubernetes.

> **High Availability?** For a 3-pod Raft cluster with automatic failover,
> follow the [HA on Kubernetes — End-to-End Runbook](./HA_KUBERNETES_RUNBOOK.md).
> This page covers the single-node deployment and general Kubernetes topics.

## Prerequisites

- Kubernetes cluster (1.20+)
- kubectl configured
- PersistentVolume provisioner
- Ingress controller (optional)

## Image

| Tag | Base | Embeddings |
|---|---|---|
| `ghcr.io/hivellm/vectorizer:3.8.2` | `scratch`, no shell, non-root (UID 65532) | BM25 only (default) |
| `ghcr.io/hivellm/vectorizer:3.8.2-fastembed` | `gcr.io/distroless/cc-debian13` (glibc), non-root (UID 65532) | BM25 + fastembed dense/multilingual models, `multilingual-e5-small` baked in — see [runbook §11](HA_KUBERNETES_RUNBOOK.md#11-multilingual-embeddings-optional) |

The GHCR package is public — no `imagePullSecrets` needed. Pin an exact tag
(never `latest`); tags are unprefixed (`3.8.0`, not `v3.8.0`). The default
image has no shell, so `kubectl exec ... sh` does not work — use
`kubectl logs` and the REST API. See the
[Embedding Providers Guide](../users/guides/EMBEDDINGS.md) for multilingual
models.

## Quick Start

```bash
# Namespace and credentials (binding 0.0.0.0 requires authentication)
kubectl apply -f deploy/k8s/namespace.yaml
kubectl create secret generic vectorizer-credentials -n vectorizer \
  --from-literal=VECTORIZER_USERNAME='admin' \
  --from-literal=VECTORIZER_PASSWORD="$(openssl rand -base64 24 | tr -d '=+/')" \
  --from-literal=VECTORIZER_API_KEY="$(openssl rand -hex 64)"

# Deploy Vectorizer
kubectl apply -f deploy/k8s/configmap.yaml
kubectl apply -f deploy/k8s/statefulset.yaml
kubectl apply -f deploy/k8s/service.yaml

# Check status
kubectl get pods -n vectorizer
kubectl logs -f -n vectorizer vectorizer-0
```

## Manifests

### Namespace

```yaml
# deploy/k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: vectorizer
```

### ConfigMap

```yaml
# deploy/k8s/configmap.yaml
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
      mcp_port: 15002

    # jwt_secret is overridden by VECTORIZER_JWT_SECRET (from the Secret)
    auth:
      enabled: true
      jwt_secret: "placeholder-overridden-by-env-VECTORIZER_JWT_SECRET"
      jwt_expiration: 3600

    logging:
      level: "warn"
      format: "json"

    performance:
      cpu:
        max_threads: 8
        memory_pool_size_mb: 2048
```

### StatefulSet

See [deploy/k8s/statefulset.yaml](../../deploy/k8s/statefulset.yaml). It runs
`ghcr.io/hivellm/vectorizer:3.8.2`, mounts the ConfigMap at
`/vectorizer/config.yml` (the server reads `config.yml` from its working
directory), keeps data on the PVC via `VECTORIZER_DATA_DIR=/data`, reads the
admin password and JWT secret from the `vectorizer-credentials` Secret, and
sets `fsGroup: 65532` so the non-root image can write the PVC.

Probes: liveness `GET /health` (200 as soon as the HTTP server is up) and
readiness `GET /ready` (503 with `Retry-After` until the startup collection
load completes, 200 after). Both are anonymous.

### Service

See [deploy/k8s/service.yaml](../../deploy/k8s/service.yaml)

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

### High Availability (Raft)

For automatic failover, run three pods with Raft consensus
(`cluster.enabled: true`). HA needs more than `replicas: 3`: a headless
Service with `publishNotReadyAddresses: true`, `podManagementPolicy: Parallel`,
a config template whose `__NODE_ID__` an init container replaces with the pod
hostname, `cluster.servers[].id` equal to the pod hostnames, the data dir on
the PVC, shared auth secrets, `RUST_LOG=info`, and ports 15002, 15003, 7001
and 15503.

Writes must go to the leader: a follower answers a write with HTTP 307 and
the leader's in-cluster URL instead of forwarding it. Reads are served by any
pod.

The complete, validated procedure — manifests, validation, failover test,
rolling updates and the one-time all-pods restart when upgrading from
≤ 3.7.2 — is the
[HA on Kubernetes — End-to-End Runbook](./HA_KUBERNETES_RUNBOOK.md). The
matching manifests are `deploy/k8s/service-ha.yaml`,
`deploy/k8s/configmap-ha.yaml` and `deploy/k8s/statefulset-ha.yaml`
(namespace `vectorizer-ha`).

## Scaling

### Horizontal Scaling

The single-node manifests run one standalone server; scaling that
StatefulSet does not create a replicated cluster. For multiple replicas use
the [HA runbook](./HA_KUBERNETES_RUNBOOK.md).

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

See [deploy/helm/vectorizer/](../../deploy/helm/vectorizer/) and the
[Helm guide](./HELM.md). The chart covers single-node deployments; it does
not render the Raft cluster configuration.

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
kubectl run -it --rm debug -n vectorizer --image=busybox:1.36 --restart=Never -- \
  wget -qO- http://vectorizer:15002/health
```

### Cluster / HA Issues

See the [HA runbook troubleshooting table](./HA_KUBERNETES_RUNBOOK.md#12-troubleshooting).

## Best Practices

1. **Use StatefulSet**: For persistent storage and stable pod identity
2. **Pin the image**: `ghcr.io/hivellm/vectorizer:3.8.2`, never `latest`
3. **Keep data on the PVC**: `VECTORIZER_DATA_DIR` must point inside the volume mount
4. **Probe `/health` (liveness) and `/ready` (readiness)**
5. **Set Resource Limits**: Prevent resource exhaustion
6. **Use Secrets**: For JWT secret and credentials (not in ConfigMap)
7. **Enable Monitoring**: Prometheus metrics
8. **For HA**: follow the [HA runbook](./HA_KUBERNETES_RUNBOOK.md) — Raft (`cluster.enabled: true`), headless Service, `file_watcher.enabled: false`, `RUST_LOG=info`
