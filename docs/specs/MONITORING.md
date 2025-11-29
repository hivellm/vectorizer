# Monitoring & Observability Guide

**Version**: 1.2.0  
**Status**: Production Ready  
**Last Updated**: October 25, 2025

---

## Overview

The Vectorizer monitoring system provides comprehensive observability through:

- **Prometheus Metrics**: 15+ metrics for search, indexing, replication, and system health
- **Correlation IDs**: Request tracing across distributed operations
- **OpenTelemetry**: Optional distributed tracing support
- **System Metrics**: Automatic collection of memory, cache, and resource usage

---

## Quick Start

### 1. Enable Monitoring

Monitoring is enabled by default. Configure in `config.yml`:

```yaml
monitoring:
  prometheus:
    enabled: true
    endpoint: "/prometheus/metrics"
  
  system_metrics:
    enabled: true
    interval_secs: 15
  
  telemetry:
    enabled: false  # Optional - requires OTLP collector
```

### 2. Access Metrics

Metrics are exposed at the `/prometheus/metrics` endpoint:

```bash
curl http://localhost:15002/prometheus/metrics
```

### 3. Configure Prometheus

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'vectorizer'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:15002']
    metrics_path: '/prometheus/metrics'
```

---

## Metrics Reference

### Search Metrics

#### `vectorizer_search_requests_total`
- **Type**: Counter
- **Labels**: `collection`, `search_type`, `status`
- **Description**: Total number of search requests
- **Usage**: Track search volume and success rate

```promql
# Requests per second
rate(vectorizer_search_requests_total[5m])

# Success rate
sum(rate(vectorizer_search_requests_total{status="success"}[5m])) /
sum(rate(vectorizer_search_requests_total[5m]))
```

#### `vectorizer_search_latency_seconds`
- **Type**: Histogram
- **Labels**: `collection`, `search_type`
- **Buckets**: 1ms, 3ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s
- **Description**: Search request latency distribution

```promql
# 95th percentile latency
histogram_quantile(0.95, 
  rate(vectorizer_search_latency_seconds_bucket[5m]))

# Average latency
rate(vectorizer_search_latency_seconds_sum[5m]) /
rate(vectorizer_search_latency_seconds_count[5m])
```

#### `vectorizer_search_results_count`
- **Type**: Histogram
- **Labels**: `collection`, `search_type`
- **Buckets**: 0, 1, 5, 10, 25, 50, 100, 250, 500, 1000
- **Description**: Number of results returned per search

```promql
# Average results per search
rate(vectorizer_search_results_count_sum[5m]) /
rate(vectorizer_search_results_count_count[5m])
```

---

### Indexing Metrics

#### `vectorizer_vectors_total`
- **Type**: Gauge
- **Description**: Total number of vectors stored
- **Updated**: Every 15 seconds by system collector

```promql
# Current vector count
vectorizer_vectors_total

# Vector growth rate
rate(vectorizer_vectors_total[1h])
```

#### `vectorizer_collections_total`
- **Type**: Gauge
- **Description**: Total number of collections

```promql
# Current collections
vectorizer_collections_total
```

#### `vectorizer_insert_requests_total`
- **Type**: Counter
- **Labels**: `collection`, `status`
- **Description**: Total number of insert requests

```promql
# Inserts per second
rate(vectorizer_insert_requests_total[5m])

# Failed inserts
rate(vectorizer_insert_requests_total{status="error"}[5m])
```

#### `vectorizer_insert_latency_seconds`
- **Type**: Histogram
- **Buckets**: 1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s
- **Description**: Insert operation latency

```promql
# 99th percentile insert latency
histogram_quantile(0.99,
  rate(vectorizer_insert_latency_seconds_bucket[5m]))
```

---

### Replication Metrics

#### `vectorizer_replication_lag_ms`
- **Type**: Gauge
- **Description**: Average replication lag in milliseconds

```promql
# Current replication lag
vectorizer_replication_lag_ms

# Alert if lag > 1000ms
vectorizer_replication_lag_ms > 1000
```

#### `vectorizer_replication_bytes_sent_total`
- **Type**: Counter
- **Description**: Total bytes sent via replication

```promql
# Bytes sent per second
rate(vectorizer_replication_bytes_sent_total[5m])

# MB sent per hour
rate(vectorizer_replication_bytes_sent_total[1h]) / 1024 / 1024
```

#### `vectorizer_replication_bytes_received_total`
- **Type**: Counter
- **Description**: Total bytes received via replication (replica only)

```promql
# Bytes received per second
rate(vectorizer_replication_bytes_received_total[5m])
```

#### `vectorizer_replication_operations_pending`
- **Type**: Gauge
- **Description**: Number of operations pending replication

```promql
# Alert if backlog > 10000
vectorizer_replication_operations_pending > 10000
```

---

### System Metrics

#### `vectorizer_memory_usage_bytes`
- **Type**: Gauge
- **Description**: Process memory usage in bytes
- **Updated**: Every 15 seconds

```promql
# Memory usage in MB
vectorizer_memory_usage_bytes / 1024 / 1024

# Alert if memory > 4GB
vectorizer_memory_usage_bytes > 4294967296
```

#### `vectorizer_cache_requests_total`
- **Type**: Counter
- **Labels**: `cache_type`, `result`
- **Description**: Total cache requests (hits and misses)

```promql
# Cache hit rate
sum(rate(vectorizer_cache_requests_total{result="hit"}[5m])) /
sum(rate(vectorizer_cache_requests_total[5m]))
```

#### `vectorizer_api_errors_total`
- **Type**: Counter
- **Labels**: `endpoint`, `error_type`, `status_code`
- **Description**: Total API errors

```promql
# Error rate
rate(vectorizer_api_errors_total[5m])

# Errors by endpoint
sum by (endpoint) (rate(vectorizer_api_errors_total[5m]))
```

---

## Correlation IDs

Every HTTP request receives a unique correlation ID for distributed tracing.

### Request Header

```bash
curl -H "X-Correlation-ID: my-custom-id-12345" \
  http://localhost:15002/collections
```

### Response Header

```
HTTP/1.1 200 OK
x-correlation-id: my-custom-id-12345
```

### Usage in Logs

All log entries include the correlation ID when available:

```json
{
  "level": "INFO",
  "timestamp": "2025-10-25T10:30:45Z",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Search completed",
  "collection": "my-collection",
  "duration_ms": 2.5
}
```

---

## OpenTelemetry (Optional)

OpenTelemetry provides distributed tracing across services.

### Enable Telemetry

```yaml
monitoring:
  telemetry:
    enabled: true
    otlp_endpoint: "http://localhost:4317"
    service_name: "vectorizer"
```

### Requirements

- OTLP collector running (Jaeger, Zipkin, or OpenTelemetry Collector)
- gRPC port 4317 accessible

### Docker Compose Example

```yaml
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP

  vectorizer:
    image: vectorizer:latest
    environment:
      - OTLP_ENDPOINT=http://jaeger:4317
    ports:
      - "15002:15002"
```

---

## Grafana Dashboard

### Import Dashboard

A pre-configured Grafana dashboard is available in `docs/grafana/vectorizer-dashboard.json`.

**Dashboard Features**:
- Search performance (latency, throughput, success rate)
- Indexing operations (insert rate, vector growth)
- Replication health (lag, throughput, replica status)
- System resources (memory, cache efficiency)
- API errors (by endpoint, error type)

### Import Steps

1. Open Grafana → Dashboards → Import
2. Upload `vectorizer-dashboard.json`
3. Select Prometheus data source
4. Click Import

---

## Alerting Rules

### Prometheus Alert Rules

Create `vectorizer-alerts.yml`:

```yaml
groups:
  - name: vectorizer
    interval: 30s
    rules:
      # High search latency
      - alert: HighSearchLatency
        expr: histogram_quantile(0.95, rate(vectorizer_search_latency_seconds_bucket[5m])) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High search latency detected"
          description: "95th percentile latency is {{ $value }}s"

      # High error rate
      - alert: HighErrorRate
        expr: rate(vectorizer_api_errors_total[5m]) > 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High API error rate"
          description: "{{ $value }} errors/sec"

      # Replication lag
      - alert: HighReplicationLag
        expr: vectorizer_replication_lag_ms > 1000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Replication lag is high"
          description: "Lag is {{ $value }}ms"

      # Memory usage
      - alert: HighMemoryUsage
        expr: vectorizer_memory_usage_bytes > 4294967296  # 4GB
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value | humanize }}B"

      # No recent searches
      - alert: NoSearchActivity
        expr: rate(vectorizer_search_requests_total[10m]) == 0
        for: 30m
        labels:
          severity: info
        annotations:
          summary: "No search activity detected"
```

### Apply Alert Rules

```bash
prometheus --config.file=prometheus.yml \
  --storage.tsdb.path=/prometheus \
  --web.console.libraries=/usr/share/prometheus/console_libraries \
  --web.console.templates=/usr/share/prometheus/consoles \
  --web.enable-lifecycle
```

---

## Performance Targets

### Production SLOs

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Search Latency (p95) | < 50ms | > 100ms |
| Insert Latency (p95) | < 100ms | > 250ms |
| API Success Rate | > 99.9% | < 99% |
| Replication Lag | < 100ms | > 1000ms |
| Memory Usage | < 2GB | > 4GB |
| Cache Hit Rate | > 80% | < 60% |

---

## Troubleshooting

### No Metrics Appearing

**Issue**: `/prometheus/metrics` returns empty or no data

**Solutions**:
1. Check monitoring initialization:
   ```bash
   grep "Prometheus metrics registry initialized" logs
   ```

2. Verify endpoint is registered:
   ```bash
   curl http://localhost:15002/prometheus/metrics
   ```

3. Generate some traffic:
   ```bash
   # Trigger search metrics
   curl -X POST http://localhost:15002/collections/test/search/text \
     -H "Content-Type: application/json" \
     -d '{"query": "test", "limit": 10}'
   ```

### High Memory Usage

**Issue**: `vectorizer_memory_usage_bytes` growing continuously

**Solutions**:
1. Check vector count growth:
   ```promql
   rate(vectorizer_vectors_total[1h])
   ```

2. Review collection configurations (quantization enabled?)

3. Check for memory leaks in logs

### Replication Lag

**Issue**: `vectorizer_replication_lag_ms` > 1000

**Solutions**:
1. Check network latency between master and replica
2. Review `vectorizer_replication_operations_pending`
3. Check master and replica logs for errors
4. Consider scaling replicas or optimizing operations

---

## Integration Examples

### Prometheus + Grafana + Alert Manager

Full stack deployment:

```yaml
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./vectorizer-alerts.yml:/etc/prometheus/alerts.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:latest
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - ./grafana/vectorizer-dashboard.json:/etc/grafana/provisioning/dashboards/vectorizer.json
    ports:
      - "3000:3000"

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"

  vectorizer:
    image: vectorizer:latest
    ports:
      - "15002:15002"
```

### Accessing Dashboards

- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)
- **Alert Manager**: http://localhost:9093
- **Vectorizer Metrics**: http://localhost:15002/prometheus/metrics

---

## Best Practices

### 1. Metric Collection

- **Keep cardinality low**: Avoid high-cardinality labels (user IDs, timestamps)
- **Use meaningful labels**: `collection`, `search_type`, `status`
- **Pre-aggregate**: Use counters and histograms, not gauges for everything

### 2. Alert Configuration

- **Set appropriate thresholds**: Based on actual SLOs
- **Use `for` duration**: Avoid alert flapping
- **Include context**: Add helpful annotations with links

### 3. Dashboard Design

- **Group by subsystem**: Search, Indexing, Replication, System
- **Use appropriate visualizations**: Time series for trends, gauges for current state
- **Add SLO markers**: Show target lines on graphs

### 4. Performance Impact

- **Metrics overhead**: < 1% CPU, < 10MB memory
- **Collection interval**: 15 seconds balances freshness and overhead
- **Histogram resolution**: 10 buckets per metric

---

## Advanced Configuration

### Custom Metrics Collection Interval

```yaml
monitoring:
  system_metrics:
    enabled: true
    interval_secs: 30  # Reduce frequency for lower overhead
```

### Enable OpenTelemetry

```yaml
monitoring:
  telemetry:
    enabled: true
    otlp_endpoint: "http://otel-collector:4317"
    service_name: "vectorizer-prod"
    sampler: "parent_based"  # Inherit sampling decision from parent
```

### Correlation ID Configuration

```yaml
logging:
  correlation_id_enabled: true
```

Correlation IDs are automatically:
- Generated for new requests (UUID v4)
- Extracted from `X-Correlation-ID` header if present
- Added to all response headers
- Included in structured logs

---

## Monitoring Checklist

### Production Deployment

- [ ] Prometheus scraping configured (15s interval)
- [ ] Grafana dashboard imported
- [ ] Alert rules configured
- [ ] Alert channels configured (email, Slack, PagerDuty)
- [ ] Correlation IDs enabled
- [ ] Log aggregation configured (ELK, Loki, CloudWatch)
- [ ] Metrics retention configured (default: 15 days)
- [ ] SLO targets documented
- [ ] Runbook created for common alerts

### Day 2 Operations

- [ ] Review metrics weekly for trends
- [ ] Adjust alert thresholds based on actual performance
- [ ] Archive historical data (>90 days)
- [ ] Update dashboards as features are added
- [ ] Test alert notifications quarterly

---

## Metrics Export Format

### Prometheus Text Format

```
# HELP vectorizer_search_requests_total Total number of search requests
# TYPE vectorizer_search_requests_total counter
vectorizer_search_requests_total{collection="my-collection",search_type="text",status="success"} 1234

# HELP vectorizer_search_latency_seconds Search request latency in seconds
# TYPE vectorizer_search_latency_seconds histogram
vectorizer_search_latency_seconds_bucket{collection="my-collection",search_type="text",le="0.001"} 50
vectorizer_search_latency_seconds_bucket{collection="my-collection",search_type="text",le="0.003"} 120
vectorizer_search_latency_seconds_bucket{collection="my-collection",search_type="text",le="+Inf"} 150
vectorizer_search_latency_seconds_sum{collection="my-collection",search_type="text"} 0.450
vectorizer_search_latency_seconds_count{collection="my-collection",search_type="text"} 150
```

---

## See Also

- `METRICS_REFERENCE.md` - Complete metrics reference
- `REPLICATION.md` - Replication configuration and monitoring
- `config.yml` - Full configuration options
- Grafana dashboard: `docs/grafana/vectorizer-dashboard.json`
- Alert rules: `docs/prometheus/vectorizer-alerts.yml`

---

**Note**: OpenTelemetry tracing is optional and requires an external OTLP collector.
The system will continue to function normally if the collector is not available.

