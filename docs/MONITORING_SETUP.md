# Monitoring Setup Guide

Complete guide for setting up monitoring for Vectorizer in production.

## Overview

Vectorizer exposes Prometheus metrics at `/prometheus/metrics` endpoint. This guide covers:
- Prometheus configuration
- Grafana dashboards
- Alert rules
- Log aggregation

## Prometheus Configuration

### Basic Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'vectorizer'
    static_configs:
      - targets: ['vectorizer:15002']
    metrics_path: '/prometheus/metrics'
    scrape_interval: 10s
```

### Kubernetes ServiceMonitor

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
      interval: 10s
```

## Grafana Setup

### Import Dashboard

Import the pre-built dashboard from `docs/grafana/vectorizer-dashboard.json`:

1. Open Grafana
2. Go to Dashboards → Import
3. Upload `vectorizer-dashboard.json`
4. Select Prometheus data source
5. Import

### Dashboard Panels

The dashboard includes:
- **Overview**: Uptime, version, collections count
- **Performance**: Query latency, throughput, CPU usage
- **Capacity**: Memory usage, disk usage, vector count
- **Reliability**: Error rate, replication lag, health status
- **Operations**: Request rate, connection count, queue depth

## Alert Rules

### Prometheus Alert Rules

```yaml
# prometheus/alerts.yml
groups:
  - name: vectorizer
    interval: 30s
    rules:
      # High CPU usage
      - alert: VectorizerHighCPU
        expr: rate(process_cpu_seconds_total{job="vectorizer"}[5m]) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer CPU usage is high"
          description: "CPU usage is {{ $value }}%"

      # High memory usage
      - alert: VectorizerHighMemory
        expr: (process_resident_memory_bytes{job="vectorizer"} / 1024 / 1024 / 1024) > 14
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer memory usage is high"
          description: "Memory usage is {{ $value }}GB"

      # Slow queries
      - alert: VectorizerSlowQueries
        expr: histogram_quantile(0.95, rate(vectorizer_query_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer queries are slow"
          description: "95th percentile query time is {{ $value }}s"

      # High error rate
      - alert: VectorizerHighErrorRate
        expr: rate(vectorizer_errors_total[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Vectorizer error rate is high"
          description: "Error rate is {{ $value }} errors/sec"

      # Replication lag
      - alert: VectorizerReplicationLag
        expr: vectorizer_replication_lag_seconds > 30
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer replication lag is high"
          description: "Replication lag is {{ $value }}s"

      # Disk space
      - alert: VectorizerLowDiskSpace
        expr: (node_filesystem_avail_bytes{mountpoint="/data"} / node_filesystem_size_bytes{mountpoint="/data"}) < 0.2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer disk space is low"
          description: "Disk space is {{ $value }}%"

      # Service down
      - alert: VectorizerDown
        expr: up{job="vectorizer"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Vectorizer is down"
          description: "Vectorizer service is not responding"
```

### Alertmanager Configuration

```yaml
# alertmanager.yml
route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
    - match:
        severity: warning
      receiver: 'warning-alerts'

receivers:
  - name: 'default'
    webhook_configs:
      - url: 'http://webhook:5001/'

  - name: 'critical-alerts'
    email_configs:
      - to: 'oncall@example.com'
        from: 'alerts@example.com'
        smarthost: 'smtp.example.com:587'
        auth_username: 'alerts'
        auth_password: 'password'

  - name: 'warning-alerts'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/...'
        channel: '#alerts'
```

## Key Metrics

### Performance Metrics

- `vectorizer_query_duration_seconds` - Query latency histogram
- `vectorizer_queries_total` - Total query count
- `vectorizer_queries_per_second` - Query throughput
- `process_cpu_seconds_total` - CPU usage
- `process_resident_memory_bytes` - Memory usage

### Capacity Metrics

- `vectorizer_collections_total` - Number of collections
- `vectorizer_vectors_total` - Total vector count
- `vectorizer_memory_usage_bytes` - Memory usage
- `node_filesystem_avail_bytes` - Available disk space

### Reliability Metrics

- `vectorizer_errors_total` - Error count
- `vectorizer_replication_lag_seconds` - Replication lag
- `up` - Service availability
- `vectorizer_health_status` - Health check status

## Log Aggregation

### Loki Configuration

```yaml
# loki-config.yml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/index
    cache_location: /loki/cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h
```

### Promtail Configuration

```yaml
# promtail-config.yml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: vectorizer
    static_configs:
      - targets:
          - localhost
        labels:
          job: vectorizer
          __path__: /var/log/vectorizer/*.log
```

## Best Practices

1. **Monitor Key Metrics**: Focus on latency, throughput, errors
2. **Set Appropriate Thresholds**: Based on your SLA
3. **Use Alerting**: But avoid alert fatigue
4. **Regular Review**: Review and tune alerts regularly
5. **Document Runbooks**: For each alert
6. **Test Alerts**: Regularly test alert delivery
7. **Dashboard Organization**: Group related metrics
8. **Retention Policy**: Set appropriate retention for metrics

## Troubleshooting

### Metrics Not Appearing

1. Check Prometheus can reach Vectorizer
2. Verify `/prometheus/metrics` endpoint is accessible
3. Check Prometheus scrape configuration
4. Verify service discovery is working

### High Cardinality

1. Limit label cardinality
2. Use recording rules for aggregations
3. Filter unnecessary metrics
4. Use metric relabeling

### Alert Fatigue

1. Tune alert thresholds
2. Use alert grouping
3. Implement alert deduplication
4. Review and disable unnecessary alerts

