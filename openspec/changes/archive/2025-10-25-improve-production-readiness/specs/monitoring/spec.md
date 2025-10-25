# Monitoring Capability - Spec Delta

**Change ID**: `improve-production-readiness`  
**Capability**: monitoring (NEW)  
**Type**: ADDED

---

## ADDED Requirements

### Requirement: Prometheus Metrics Export
The system SHALL expose Prometheus-compatible metrics for monitoring and alerting.

#### Scenario: Metrics endpoint available
- **GIVEN** a vectorizer instance running
- **WHEN** GET `/metrics` is called
- **THEN** the response SHALL be in Prometheus text format
- **AND** the Content-Type SHALL be `text/plain; version=0.0.4`
- **AND** metrics SHALL be grouped by subsystem

#### Scenario: Search request metrics
- **GIVEN** the vectorizer is handling search requests
- **WHEN** 100 searches are performed
- **THEN** `vectorizer_search_requests_total` counter SHALL be 100
- **AND** `vectorizer_search_latency_seconds` histogram SHALL have 100 observations
- **AND** p50, p95, p99 percentiles SHALL be available

#### Scenario: Replication lag metrics
- **GIVEN** a replica with replication lag
- **WHEN** lag increases to 500ms
- **THEN** `vectorizer_replication_lag_ms` gauge SHALL be 500
- **AND** alerts MAY be triggered if lag > threshold

---

### Requirement: Distributed Tracing
The system SHALL support OpenTelemetry distributed tracing for request flow visibility.

#### Scenario: Trace API request
- **GIVEN** a search request with trace headers
- **WHEN** the request is processed
- **THEN** a span SHALL be created for the request
- **AND** child spans SHALL be created for each operation
- **AND** spans SHALL include: duration, status, attributes
- **AND** spans SHALL be exported to configured backend

#### Scenario: Trace replication operation
- **GIVEN** a vector insert on master
- **WHEN** the operation replicates to replicas
- **THEN** trace context SHALL propagate to replicas
- **AND** end-to-end latency SHALL be measurable
- **AND** bottlenecks SHALL be identifiable

---

### Requirement: Structured Logging
The system SHALL emit structured logs with correlation IDs for debugging.

#### Scenario: Log with correlation ID
- **GIVEN** an API request with correlation ID header
- **WHEN** the request is processed
- **THEN** all log entries SHALL include the correlation ID
- **AND** logs SHALL be in JSON format
- **AND** logs SHALL include: timestamp, level, message, context

#### Scenario: Log aggregation
- **GIVEN** logs from multiple requests
- **WHEN** filtering by correlation ID
- **THEN** all logs for that request SHALL be retrievable
- **AND** request flow SHALL be reconstructable

---

### Requirement: Health Check Endpoint
The system SHALL provide detailed health status for orchestration platforms.

#### Scenario: Healthy instance
- **GIVEN** a vectorizer instance running normally
- **WHEN** GET `/health` is called
- **THEN** HTTP status SHALL be 200
- **AND** response SHALL include: status=healthy, checks=[database, replication]
- **AND** checks SHALL all be passing

#### Scenario: Unhealthy replication
- **GIVEN** a master unable to connect to any replicas
- **WHEN** GET `/health` is called
- **THEN** HTTP status SHALL be 503
- **AND** response SHALL include: status=degraded, failed_checks=[replication]
- **AND** orchestrator MAY restart instance

---

## Metrics Catalog

### Search Metrics
- `vectorizer_search_requests_total{collection, status}` - Counter
- `vectorizer_search_latency_seconds{collection}` - Histogram
- `vectorizer_search_results_count{collection}` - Histogram

### Indexing Metrics
- `vectorizer_vectors_total{collection}` - Gauge
- `vectorizer_insert_requests_total{collection, status}` - Counter
- `vectorizer_insert_latency_seconds{collection}` - Histogram

### Replication Metrics
- `vectorizer_replication_lag_ms{replica_id}` - Gauge
- `vectorizer_replication_bytes_sent_total` - Counter
- `vectorizer_replication_bytes_received_total` - Counter
- `vectorizer_replication_operations_pending` - Gauge
- `vectorizer_replication_connected_replicas` - Gauge

### Cache Metrics
- `vectorizer_cache_requests_total{cache_type, status}` - Counter (hit/miss)
- `vectorizer_cache_evictions_total{cache_type}` - Counter
- `vectorizer_cache_size_bytes{cache_type}` - Gauge

### System Metrics
- `vectorizer_memory_usage_bytes{type}` - Gauge (heap, index, cache)
- `vectorizer_collections_total` - Gauge
- `vectorizer_api_errors_total{endpoint, error_type}` - Counter

---

## Configuration

### Monitoring Configuration
```yaml
monitoring:
  # Prometheus metrics
  metrics:
    enabled: true
    endpoint: "/metrics"
    
  # OpenTelemetry tracing
  tracing:
    enabled: true
    endpoint: "http://jaeger:14268/api/traces"
    sample_rate: 0.1  # 10% sampling
    
  # Structured logging
  logging:
    format: "json"
    level: "info"
    include_correlation_id: true
    
  # Health checks
  health:
    enabled: true
    endpoint: "/health"
    checks:
      - database
      - replication
      - memory
```

---

## Integration Examples

### Prometheus Scrape Config
```yaml
scrape_configs:
  - job_name: 'vectorizer'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:15002']
    metrics_path: '/metrics'
```

### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "Vectorizer Monitoring",
    "panels": [
      {
        "title": "Search Latency (p95)",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, vectorizer_search_latency_seconds)"
          }
        ]
      },
      {
        "title": "Replication Lag",
        "targets": [
          {
            "expr": "vectorizer_replication_lag_ms"
          }
        ]
      }
    ]
  }
}
```

---

## Performance Requirements

### Requirement: Metrics Collection Overhead
Metrics collection SHALL NOT degrade performance by more than 2%.

#### Scenario: High-throughput metrics
- **GIVEN** a system handling 10,000 requests/second
- **WHEN** metrics collection is enabled
- **THEN** throughput SHALL remain >= 9,800 requests/second
- **AND** latency p95 SHALL increase by <= 0.2ms

---

## Testing Requirements

### Tests Required
- Metrics endpoint returns valid Prometheus format
- All metrics update correctly
- Tracing context propagates through operations
- Correlation IDs appear in all logs
- Health check reflects system state
- Metrics collection performance overhead

---

## Documentation Updates

### Required Documentation
- `docs/MONITORING.md` - Complete monitoring guide
- `docs/METRICS_REFERENCE.md` - All metrics explained
- Grafana dashboard templates
- Alert rules examples
- Troubleshooting guide using metrics

---

**Spec Delta Status**: Complete  
**Review Status**: Pending  
**Implementation Status**: Not started

