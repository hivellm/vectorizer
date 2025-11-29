# Vectorizer Helm Chart

Helm chart for deploying Vectorizer vector database on Kubernetes.

## Prerequisites

- Kubernetes 1.20+
- Helm 3.0+
- PersistentVolume provisioner

## Installation

### Add Repository

```bash
helm repo add vectorizer https://charts.vectorizer.io
helm repo update
```

### Install Chart

```bash
# Basic installation
helm install vectorizer vectorizer/vectorizer

# With custom values
helm install vectorizer vectorizer/vectorizer -f my-values.yaml

# With persistence
helm install vectorizer vectorizer/vectorizer \
  --set persistence.enabled=true \
  --set persistence.size=100Gi
```

## Configuration

### Basic Configuration

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

### With Replication

```yaml
# Master node
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"

# Replica node
replication:
  enabled: true
  role: "replica"
  master_address: "master.vectorizer.svc.cluster.local:7001"
```

### With Monitoring

```yaml
monitoring:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 10s
```

### With Ingress

```yaml
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

## Values Reference

See [values.yaml](./values.yaml) for all available configuration options.

## Upgrading

```bash
# Upgrade to new version
helm upgrade vectorizer vectorizer/vectorizer

# Upgrade with new values
helm upgrade vectorizer vectorizer/vectorizer -f new-values.yaml
```

## Uninstalling

```bash
helm uninstall vectorizer
```

## Troubleshooting

### Check Pod Status

```bash
kubectl get pods -l app.kubernetes.io/name=vectorizer
```

### View Logs

```bash
kubectl logs -l app.kubernetes.io/name=vectorizer
```

### Check ConfigMap

```bash
kubectl get configmap vectorizer-config -o yaml
```

## Support

For issues and questions:
- GitHub: https://github.com/hivellm/vectorizer
- Documentation: https://docs.vectorizer.io

