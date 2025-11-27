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

### Replication

For high availability, deploy multiple replicas with replication enabled:

```yaml
replicas: 3
```

Configure master-replica setup in ConfigMap.

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

## Best Practices

1. **Use StatefulSet**: For persistent storage
2. **Set Resource Limits**: Prevent resource exhaustion
3. **Enable Health Checks**: Liveness and readiness probes
4. **Use ConfigMaps**: For configuration management
5. **Enable Monitoring**: Prometheus metrics
6. **Use Secrets**: For sensitive data
7. **Enable TLS**: For secure communication
8. **Regular Backups**: Automated backup jobs
