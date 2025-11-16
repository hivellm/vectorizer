---
title: Monitoring and Observability
module: monitoring
id: monitoring-guide
order: 1
description: Monitor Vectorizer performance and health
tags: [monitoring, metrics, observability, prometheus, health]
---

# Monitoring and Observability

Complete guide to monitoring Vectorizer performance, health, and metrics.

## Health Checks

### Basic Health Check

Check if Vectorizer is running and healthy:

```bash
curl http://localhost:15002/health
```

**Response:**

```json
{
  "status": "healthy",
  "version": "1.3.0",
  "uptime_seconds": 3600
}
```

### Health Check Status Codes

- **200 OK**: Service is healthy
- **503 Service Unavailable**: Service is unhealthy or starting up

### Using Health Checks

**Load balancer health checks:**

```nginx
location /health {
    proxy_pass http://localhost:15002/health;
    access_log off;
}
```

**Kubernetes liveness probe:**

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 15002
  initialDelaySeconds: 30
  periodSeconds: 10
```

## Prometheus Metrics

Vectorizer exposes Prometheus metrics at `/metrics`.

### Accessing Metrics

```bash
curl http://localhost:15002/metrics
```

### Key Metrics

#### Search Metrics

- **`vectorizer_search_requests_total`**: Total number of search requests
  - Labels: `collection`, `search_type` (basic, intelligent, semantic, hybrid)
- **`vectorizer_search_latency_seconds`**: Search operation latency
  - Labels: `collection`, `search_type`
- **`vectorizer_search_results_count`**: Number of results returned
  - Labels: `collection`, `search_type`

#### Collection Metrics

- **`vectorizer_collections_total`**: Total number of collections
- **`vectorizer_vectors_total`**: Total number of vectors across all collections
  - Labels: `collection`
- **`vectorizer_collection_memory_bytes`**: Memory usage per collection
  - Labels: `collection`

#### Index Metrics

- **`vectorizer_index_build_duration_seconds`**: Time to build HNSW index
  - Labels: `collection`
- **`vectorizer_index_size_bytes`**: Size of HNSW index
  - Labels: `collection`

#### Operation Metrics

- **`vectorizer_insert_operations_total`**: Total insert operations
  - Labels: `collection`, `operation_type` (single, batch)
- **`vectorizer_insert_latency_seconds`**: Insert operation latency
  - Labels: `collection`, `operation_type`
- **`vectorizer_update_operations_total`**: Total update operations
- **`vectorizer_delete_operations_total`**: Total delete operations

### Example Prometheus Queries

**Average search latency:**

```promql
rate(vectorizer_search_latency_seconds_sum[5m]) / rate(vectorizer_search_requests_total[5m])
```

**Search requests per second:**

```promql
rate(vectorizer_search_requests_total[5m])
```

**Memory usage by collection:**

```promql
vectorizer_collection_memory_bytes
```

**P95 search latency:**

```promql
histogram_quantile(0.95, rate(vectorizer_search_latency_seconds_bucket[5m]))
```

## Grafana Dashboards

### Recommended Dashboards

1. **Search Performance Dashboard**:

   - Search latency (P50, P95, P99)
   - Search throughput (QPS)
   - Search results count

2. **Collection Dashboard**:

   - Collection count
   - Vectors per collection
   - Memory usage per collection

3. **Operations Dashboard**:
   - Insert/update/delete rates
   - Operation latencies
   - Error rates

### Example Dashboard Configuration

```json
{
  "dashboard": {
    "title": "Vectorizer Performance",
    "panels": [
      {
        "title": "Search Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(vectorizer_search_latency_seconds_bucket[5m]))"
          }
        ]
      },
      {
        "title": "Search QPS",
        "targets": [
          {
            "expr": "rate(vectorizer_search_requests_total[5m])"
          }
        ]
      }
    ]
  }
}
```

## Logging

### Log Levels

- **trace**: Very detailed (development only)
- **debug**: Debug information
- **info**: General information (default)
- **warn**: Warnings
- **error**: Errors only

### Structured Logging

Vectorizer uses structured logging with JSON format:

```json
{
  "timestamp": "2025-11-16T10:00:00Z",
  "level": "info",
  "message": "Collection created",
  "collection": "my_collection",
  "dimension": 384
}
```

### Log Aggregation

**ELK Stack (Elasticsearch, Logstash, Kibana):**

```yaml
# Logstash configuration
input {
file {
path => "/var/log/vectorizer/*.log"
codec => json
}
}
```

**Loki (Grafana):**

```yaml
# Promtail configuration
scrape_configs:
  - job_name: vectorizer
    static_configs:
      - targets:
          - localhost
        labels:
          job: vectorizer
          __path__: /var/log/vectorizer/*.log
```

## Alerting

### Recommended Alerts

**High Search Latency:**

```yaml
alert: HighSearchLatency
expr: histogram_quantile(0.95, rate(vectorizer_search_latency_seconds_bucket[5m])) > 0.05
for: 5m
annotations:
  summary: "Search latency is high"
  description: "P95 search latency is {{ $value }}s"
```

**High Memory Usage:**

```yaml
alert: HighMemoryUsage
expr: vectorizer_collection_memory_bytes > 2147483648 # 2GB
for: 10m
annotations:
  summary: "Collection memory usage is high"
  description: "Collection {{ $labels.collection }} is using {{ $value }} bytes"
```

**Service Down:**

```yaml
alert: VectorizerDown
expr: up{job="vectorizer"} == 0
for: 1m
annotations:
  summary: "Vectorizer service is down"
```

**High Error Rate:**

```yaml
alert: HighErrorRate
expr: rate(vectorizer_errors_total[5m]) > 10
for: 5m
annotations:
  summary: "High error rate detected"
```

## Performance Monitoring

### Key Performance Indicators (KPIs)

1. **Search Latency**: Target < 10ms for small collections
2. **Search Throughput**: Target > 1000 QPS
3. **Memory Usage**: Monitor per collection
4. **Index Build Time**: Track index construction duration
5. **Error Rate**: Should be < 0.1%

### Monitoring Tools

**Prometheus + Grafana:**

- Real-time metrics
- Historical trends
- Custom dashboards
- Alerting

**Custom Monitoring Script:**

```python
import requests
import time

def monitor_vectorizer(base_url="http://localhost:15002"):
    """Monitor Vectorizer health and performance."""
    while True:
        # Health check
        health = requests.get(f"{base_url}/health").json()
        print(f"Status: {health['status']}, Uptime: {health['uptime_seconds']}s")

        # Get metrics
        metrics = requests.get(f"{base_url}/metrics").text
        # Parse and display key metrics

        time.sleep(60)  # Check every minute
```

## Related Topics

- [Performance Guide](../performance/PERFORMANCE.md) - Performance optimization
- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration
- [Troubleshooting Guide](../troubleshooting/TROUBLESHOOTING.md) - Debugging issues
