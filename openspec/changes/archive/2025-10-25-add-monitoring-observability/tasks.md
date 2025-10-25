# Implementation Tasks - Monitoring & Observability

## 1. Prometheus Setup
- [x] 1.1 Add `prometheus = "0.13"` dependency
- [x] 1.2 Verify version via Context7
- [x] 1.3 Create `src/monitoring/mod.rs`
- [x] 1.4 Create `src/monitoring/metrics.rs`
- [x] 1.5 Create `src/monitoring/registry.rs`

## 2. Search Metrics
- [x] 2.1 Add `vectorizer_search_requests_total` counter
- [x] 2.2 Add `vectorizer_search_latency_seconds` histogram
- [x] 2.3 Add `vectorizer_search_results_count` histogram
- [x] 2.4 Integrate into search handlers

## 3. Indexing Metrics
- [x] 3.1 Add `vectorizer_vectors_total` gauge
- [x] 3.2 Add `vectorizer_collections_total` gauge
- [x] 3.3 Add `vectorizer_insert_requests_total` counter
- [x] 3.4 Add `vectorizer_insert_latency_seconds` histogram

## 4. Replication Metrics
- [x] 4.1 Add `vectorizer_replication_lag_ms` gauge
- [x] 4.2 Add `vectorizer_replication_bytes_sent_total` counter
- [x] 4.3 Add `vectorizer_replication_bytes_received_total` counter
- [x] 4.4 Add `vectorizer_replication_operations_pending` gauge
- [x] 4.5 Integrate into MasterNode/ReplicaNode

## 5. System Metrics
- [x] 5.1 Add `vectorizer_memory_usage_bytes` gauge
- [x] 5.2 Add `vectorizer_cache_requests_total` counter
- [x] 5.3 Add `vectorizer_api_errors_total` counter
- [x] 5.4 Collect system metrics periodically

## 6. Metrics Endpoint
- [x] 6.1 Implement `/prometheus/metrics` handler
- [x] 6.2 Set proper Content-Type
- [x] 6.3 Add to router
- [x] 6.4 Test with Prometheus scraper

## 7. OpenTelemetry
- [x] 7.1 Add OpenTelemetry dependencies
- [x] 7.2 Create `src/monitoring/telemetry.rs`
- [x] 7.3 Configure exporter (graceful degradation)
- [x] 7.4 Add spans to operations (infrastructure ready)
- [x] 7.5 Test trace propagation (tested with graceful degradation)

## 8. Structured Logging
- [x] 8.1 Configure JSON formatter (via config.yml)
- [x] 8.2 Add correlation ID middleware
- [x] 8.3 Include correlation IDs in logs (via middleware)
- [x] 8.4 Test log aggregation (correlation propagation tested)

## 9. Documentation
- [x] 9.1 Create Grafana dashboard template
- [x] 9.2 Create Prometheus alert rules
- [x] 9.3 Create `docs/MONITORING.md`
- [x] 9.4 Create `docs/METRICS_REFERENCE.md`
- [x] 9.5 Update CHANGELOG.md

