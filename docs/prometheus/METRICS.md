# Metrics & Observability

## Overview

Vectorizer exposes runtime telemetry in two complementary shapes:

1.  **Prometheus text exposition** at `GET /prometheus/metrics`, powered by the
    `prometheus` crate registry in
    `crates/vectorizer/src/monitoring/`. Every metric family listed in this
    document is registered there and gathered via
    `vectorizer::monitoring::export_metrics()`.
2.  **File-watcher JSON dashboard feed** at `GET /metrics`, served by
    `crates/vectorizer-server/src/server/core/helpers.rs:165` and backed by
    `FileWatcherMetrics` (`crates/vectorizer/src/file_watcher/metrics.rs`).
    This endpoint returns a structured JSON document consumed by the built-in
    dashboard; it is **not** in Prometheus text format.

This document catalogs every emitted metric with its type, labels, unit, and
semantics; shows how to scrape, alert on, and visualize them; and documents
where the data comes from in the source tree so you can correlate a metric back
to the code that emits it.

## Endpoints

| Path                   | Format               | Auth (prod)           | Notes                                                                                                                  |
| ---------------------- | -------------------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `GET /prometheus/metrics` | Prometheus text 0.0.4 | **Public, never gated** | Registered on `public_routes` in `crates/vectorizer-server/src/server/core/routing.rs:147-150`; bypasses auth + HiveHub tenant middleware. |
| `GET /metrics`         | `application/json`   | Subject to auth       | File-watcher dashboard feed. JSON body shape = `FileWatcherMetrics` (see [File-watcher metrics](#file-watcher-metrics-get-metrics)). |
| `GET /health`          | `application/json`   | Public                | Liveness + query-cache summary; not a metrics feed but useful as a scrape-side availability check.                    |

The `/prometheus/metrics` endpoint is intentionally carved out of the
production auth middleware (`routing.rs:824-835`) so Prometheus can scrape
unauthenticated. If you are exposing the server on a public network, run it
behind an ingress that restricts `/prometheus/metrics` to your Prometheus
subnet.

### Correlation IDs

Every request is tagged with an `X-Correlation-ID` header (generated if absent)
by `crates/vectorizer/src/monitoring/correlation.rs`. Correlation IDs are
propagated in task-local storage and echoed on the response — use them to join
structured logs with alerts that fire on these metrics.

### OpenTelemetry (optional)

`crates/vectorizer/src/monitoring/telemetry.rs` exposes a best-effort
`try_init()` that is **prepared but disabled by default**. It will log a
warning and continue if no OTLP collector is reachable. No spans are exported
unless you wire an exporter via `OTLP_ENDPOINT` / `config.yml`.

## Scraping setup

### Prometheus

```yaml
# prometheus.yml
scrape_configs:
  - job_name: "vectorizer"
    scrape_interval: 15s
    scrape_timeout: 10s
    metrics_path: /prometheus/metrics
    static_configs:
      - targets: ["vectorizer.example.com:15002"]
        labels:
          service: vectorizer
          env: prod
```

The server runs a background `SystemCollector`
(`crates/vectorizer/src/monitoring/system_collector.rs`) on a 15-second tick
that refreshes `vectorizer_memory_usage_bytes`, `vectorizer_collections_total`,
and `vectorizer_vectors_total`. A scrape interval shorter than 15 s therefore
only adds HTTP overhead without new data for those three gauges.

### Alert rules

Load `docs/prometheus/vectorizer-alerts.yml` into Prometheus:

```yaml
rule_files:
  - /etc/prometheus/rules/vectorizer-alerts.yml
```

See [Alert rules](#alert-rules) below for the full list.

### Kubernetes `ServiceMonitor`

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: vectorizer
spec:
  selector:
    matchLabels:
      app: vectorizer
  endpoints:
    - port: http
      path: /prometheus/metrics
      interval: 15s
```

## Metric catalog

All metrics are defined in `crates/vectorizer/src/monitoring/metrics.rs` (see
`Metrics::new` for the authoritative list, `Metrics::register` for the
registry wire-up). Histogram buckets are specified at construction time and
mirrored below.

### Search metrics

| Metric                                      | Type            | Labels                                       | Unit    | Source                                    |
| ------------------------------------------- | --------------- | -------------------------------------------- | ------- | ----------------------------------------- |
| `vectorizer_search_requests_total`          | Counter vector  | `collection`, `search_type`, `status`        | 1       | `monitoring/metrics.rs:147`               |
| `vectorizer_search_latency_seconds`         | Histogram vector| `collection`, `search_type`                  | seconds | `monitoring/metrics.rs:156` (buckets: 1 ms, 3 ms, 5 ms, 10 ms, 25 ms, 50 ms, 100 ms, 250 ms, 500 ms, 1 s) |
| `vectorizer_search_results_count`           | Histogram vector| `collection`, `search_type`                  | 1       | `monitoring/metrics.rs:168` (buckets: 0, 1, 5, 10, 25, 50, 100, 250, 500, 1000) |

`search_type` is populated by the calling handler with values such as `basic`,
`text`, `hybrid`, `semantic`, `recommend`, or `intelligent`. `status` is
`success` or `error`.

### Indexing metrics

| Metric                                      | Type            | Labels                          | Unit    | Source                                    |
| ------------------------------------------- | --------------- | ------------------------------- | ------- | ----------------------------------------- |
| `vectorizer_vectors_total`                  | Gauge           | —                               | 1       | `monitoring/metrics.rs:181`; set by `SystemCollector::collect_vector_store_metrics` every 15 s |
| `vectorizer_collections_total`              | Gauge           | —                               | 1       | `monitoring/metrics.rs:184`; set by `SystemCollector` |
| `vectorizer_alias_operations_total`         | Counter vector  | `operation`, `status`           | 1       | `monitoring/metrics.rs:190` — `operation` ∈ `create`, `delete`, `rename`; `status` ∈ `success`, `error` |
| `vectorizer_insert_requests_total`          | Counter vector  | `collection`, `status`          | 1       | `monitoring/metrics.rs:199`               |
| `vectorizer_insert_latency_seconds`         | Histogram       | —                               | seconds | `monitoring/metrics.rs:208` (buckets: 1 ms, 5 ms, 10 ms, 25 ms, 50 ms, 100 ms, 250 ms, 500 ms, 1 s, 2.5 s) |

### Replication metrics

Emitted only on master nodes (and replicas for `*_received_total`) —
`Replication` is a BETA subsystem. See
`crates/vectorizer/src/replication/master.rs` and `replica.rs`.

| Metric                                        | Type    | Labels | Unit                    | Source                                                                                                                                                                     |
| --------------------------------------------- | ------- | ------ | ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vectorizer_replication_lag_ms`               | Gauge   | —      | **operations (approx.)**| `monitoring/metrics.rs:220`; set by `master.rs:656` as `average lag in operations`. The `_ms` suffix is a legacy name — interpret as "pending ops to catch up" unless you have confirmed the master's ~1 ms-per-op estimate. |
| `vectorizer_replication_bytes_sent_total`     | Counter | —      | bytes                   | `monitoring/metrics.rs:226`; `master.rs:393` and `master.rs:415`                                                                                                         |
| `vectorizer_replication_bytes_received_total` | Counter | —      | bytes                   | `monitoring/metrics.rs:232`; `replica.rs:320`                                                                                                                           |
| `vectorizer_replication_operations_pending`   | Gauge   | —      | 1                       | `monitoring/metrics.rs:238`; incremented in `master.rs:443` as WAL operations are appended                                                                                |

### System metrics

| Metric                                | Type            | Labels                                          | Unit  | Source                                                                 |
| ------------------------------------- | --------------- | ----------------------------------------------- | ----- | ---------------------------------------------------------------------- |
| `vectorizer_memory_usage_bytes`       | Gauge           | —                                               | bytes | `monitoring/metrics.rs:245`; set by `SystemCollector::collect_memory_metrics` via `memory_stats::memory_stats()` (process RSS) every 15 s |
| `vectorizer_cache_requests_total`     | Counter vector  | `cache_type`, `result`                          | 1     | `monitoring/metrics.rs:251`; emitted by `cache/query_cache.rs:150-181` — `cache_type="query"`, `result` ∈ `hit`, `miss`, `bypass` |
| `vectorizer_api_errors_total`         | Counter vector  | `endpoint`, `error_type`, `status_code`         | 1     | `monitoring/metrics.rs:257`; emitted by `security/rate_limit.rs:418`, `:439` and other error paths |

Vectorizer does **not** ship its own CPU / load / FD / network-connection
Prometheus gauges. Pair `/prometheus/metrics` with a standard
`node_exporter` / `cAdvisor` for host-level telemetry. (There is a
`SystemMetrics` struct in `monitoring/performance.rs`, but it is internal
bookkeeping for the `PerformanceMonitor` and is not registered with the
Prometheus registry.)

### GPU metrics (macOS Metal, `hive-gpu` feature)

Emitted only on builds that include the `hive-gpu` feature. CPU-only builds
still register the metric families but leave the counters at zero.

| Metric                                  | Type             | Labels            | Unit                | Source                              |
| --------------------------------------- | ---------------- | ----------------- | ------------------- | ----------------------------------- |
| `vectorizer_gpu_backend_type`           | Gauge            | —                 | enum (0 = None, 1 = Metal) | `monitoring/metrics.rs:264`         |
| `vectorizer_gpu_memory_usage_bytes`     | Gauge            | —                 | bytes               | `monitoring/metrics.rs:270`         |
| `vectorizer_gpu_search_requests_total`  | Counter          | —                 | 1                   | `monitoring/metrics.rs:276`         |
| `vectorizer_gpu_search_latency_seconds` | Histogram        | —                 | seconds             | `monitoring/metrics.rs:282` (buckets: 1 ms, 5 ms, 10 ms, 50 ms, 100 ms, 500 ms, 1 s, 5 s) |
| `vectorizer_gpu_batch_operations_total` | Counter vector   | `operation_type`  | 1                   | `monitoring/metrics.rs:291`         |
| `vectorizer_gpu_batch_latency_seconds`  | Histogram vector | `operation_type`  | seconds             | `monitoring/metrics.rs:300` (buckets: 10 ms, 50 ms, 100 ms, 500 ms, 1 s, 5 s, 10 s) |

### HiveHub cluster-mode metrics

Only emitted when HiveHub integration is enabled (`hub.enabled: true` in
`config.yml`). On a single-node non-Hub deployment these families are still
registered but stay empty.

| Metric                                       | Type             | Labels                                       | Unit    | Source                              |
| -------------------------------------------- | ---------------- | -------------------------------------------- | ------- | ----------------------------------- |
| `vectorizer_hub_quota_checks_total`          | Counter vector   | `tenant_id`, `quota_type`, `result`          | 1       | `monitoring/metrics.rs:311`         |
| `vectorizer_hub_quota_check_latency_seconds` | Histogram        | —                                            | seconds | `monitoring/metrics.rs:320` (buckets: 1 ms, 5 ms, 10 ms, 25 ms, 50 ms, 100 ms, 250 ms, 500 ms) |
| `vectorizer_hub_quota_exceeded_total`        | Counter vector   | `tenant_id`, `quota_type`                    | 1       | `monitoring/metrics.rs:329`         |
| `vectorizer_hub_quota_usage`                 | Gauge vector     | `tenant_id`, `quota_type`                    | varies* | `monitoring/metrics.rs:338`         |
| `vectorizer_hub_api_requests_total`          | Counter vector   | `tenant_id`, `endpoint`, `method`, `status`  | 1       | `monitoring/metrics.rs:347`         |
| `vectorizer_hub_api_latency_seconds`         | Histogram vector | `endpoint`                                   | seconds | `monitoring/metrics.rs:356` (buckets: 10 ms, 50 ms, 100 ms, 250 ms, 500 ms, 1 s, 2.5 s, 5 s) |
| `vectorizer_hub_active_tenants`              | Gauge            | —                                            | 1       | `monitoring/metrics.rs:366`         |
| `vectorizer_hub_usage_reports_total`         | Counter vector   | `status`                                     | 1       | `monitoring/metrics.rs:372`         |
| `vectorizer_hub_backup_operations_total`     | Counter vector   | `operation`, `status`                        | 1       | `monitoring/metrics.rs:381`         |

*`quota_type` values include `storage` (bytes), `collections` (count),
`api_requests` (count) — see the Grafana dashboard's `Quota Usage by Tenant`
panel for canonical labels.

In addition, `Metrics::record_tenant_api_request` maintains an in-memory
`DashMap<tenant_id, u64>` (`tenant_api_requests`, `monitoring/metrics.rs:130`)
for fast per-tenant lookup. **This counter is not exported via Prometheus** —
query it from the Hub-specific REST surface if needed.

### File-watcher metrics (`GET /metrics`)

`GET /metrics` returns the full
`FileWatcherMetrics` JSON (type defined in
`crates/vectorizer/src/file_watcher/metrics.rs:87`). These values are **not**
exported in Prometheus format; the dashboard consumes them directly. If you
want them scraped, either poll JSON and relay via a translator, or add them to
the Prometheus registry (see Troubleshooting).

| JSON path                                    | Type   | Unit        | Semantic                                                                                                                          |
| -------------------------------------------- | ------ | ----------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `timing.avg_file_processing_ms`              | f64    | ms          | total processing-time / total-files-processed                                                                                     |
| `timing.avg_discovery_ms`                    | f64    | ms          | total discovery-time / files-discovered                                                                                           |
| `timing.avg_sync_ms`                         | f64    | ms          | total sync-time / files-removed                                                                                                   |
| `timing.peak_processing_ms`                  | u64    | ms          | slowest single file processed                                                                                                     |
| `timing.uptime_seconds`                      | u64    | seconds     | since collector created                                                                                                           |
| `timing.last_activity`                       | string | — (relative)| e.g. `"3s ago"`                                                                                                                   |
| `files.total_files_processed`                | u64    | 1           | success + error                                                                                                                  |
| `files.files_processed_success`              | u64    | 1           | —                                                                                                                                 |
| `files.files_processed_error`                | u64    | 1           | —                                                                                                                                 |
| `files.files_skipped`                        | u64    | 1           | gitignore / hash-match skips                                                                                                     |
| `files.files_in_progress`                    | u64    | 1           | currently in the pipeline                                                                                                         |
| `files.files_discovered`                     | u64    | 1           | emitted by walkers                                                                                                                |
| `files.files_removed`                        | u64    | 1           | orphaned vectors pruned                                                                                                          |
| `files.files_indexed_realtime`               | u64    | 1           | indexed via fs-event (not discovery)                                                                                             |
| `system.memory_usage_bytes`                  | u64    | bytes       | Linux-only: parsed from `/proc/self/status`. `0` on macOS / Windows — use the Prometheus `vectorizer_memory_usage_bytes` gauge instead. |
| `system.cpu_usage_percent`                   | f64    | percent     | Linux-only: from `/proc/loadavg`                                                                                                  |
| `system.thread_count`                        | u32    | 1           | Linux `/proc/self/status` → `available_parallelism` fallback                                                                      |
| `system.active_file_handles`                 | u32    | 1           | Linux `/proc/self/fd` count → 0 fallback                                                                                          |
| `system.disk_io_ops_per_sec`                 | u64    | ops/s       | monotonic counter (misnamed — not rate)                                                                                           |
| `system.network_io_bytes_per_sec`            | u64    | bytes/s     | monotonic counter (misnamed — not rate)                                                                                           |
| `network.total_api_requests`                 | u64    | 1           | file-watcher→server calls                                                                                                         |
| `network.successful_api_requests`            | u64    | 1           | —                                                                                                                                 |
| `network.failed_api_requests`                | u64    | 1           | —                                                                                                                                 |
| `network.avg_api_response_ms`                | f64    | ms          | —                                                                                                                                 |
| `network.peak_api_response_ms`               | u64    | ms          | —                                                                                                                                 |
| `network.active_connections`                 | u32    | 1           | —                                                                                                                                 |
| `status.total_errors`                        | u64    | 1           | sum across all error types                                                                                                        |
| `status.errors_by_type`                      | map    | 1           | `{ "parse_error": 12, "index_error": 3, ... }`                                                                                    |
| `status.current_status`                      | string | —           | `running`, `stopped`, or `initializing` (the latter when file-watcher is disabled)                                               |
| `status.last_error`                          | string | —           | `"<type>: <message>"`                                                                                                             |
| `status.health_score`                        | u8     | 0 – 100     | `(1 - total_errors / total_files_processed) * 100`; `100` when nothing has been processed yet                                    |
| `status.restart_count`                       | u32    | 1           | —                                                                                                                                 |
| `collections[name].total_vectors`            | u64    | 1           | —                                                                                                                                 |
| `collections[name].vectors_added`            | u64    | 1           | —                                                                                                                                 |
| `collections[name].vectors_removed`          | u64    | 1           | —                                                                                                                                 |
| `collections[name].vectors_updated`          | u64    | 1           | —                                                                                                                                 |
| `collections[name].last_update`              | string | RFC 3339    | —                                                                                                                                 |

## Alert rules

`docs/prometheus/vectorizer-alerts.yml` groups alerts by subsystem. The table
below summarizes every rule; thresholds are defensive defaults — tune them
against observed baselines.

### `vectorizer_search` (eval every 30 s)

| Alert                                | Expression (abridged)                                                                         | `for` | Severity  |
| ------------------------------------ | --------------------------------------------------------------------------------------------- | ----- | --------- |
| `VectorizerHighSearchLatency`        | `histogram_quantile(0.95, rate(vectorizer_search_latency_seconds_bucket[5m])) > 0.1`          | 5 m   | warning   |
| `VectorizerCriticalSearchLatency`    | `histogram_quantile(0.99, rate(vectorizer_search_latency_seconds_bucket[5m])) > 0.5`          | 2 m   | critical  |
| `VectorizerLowSearchSuccessRate`     | `100 * success_rate(vectorizer_search_requests_total[5m]) < 95`                               | 5 m   | warning   |

Runbook: `https://docs.vectorizer.dev/runbooks/high-search-latency`
(referenced on the warning alert; no runbook URL on the critical/low-success
alerts yet).

### `vectorizer_indexing`

| Alert                               | Expression                                                                            | `for` | Severity |
| ----------------------------------- | ------------------------------------------------------------------------------------- | ----- | -------- |
| `VectorizerHighInsertLatency`       | `histogram_quantile(0.95, rate(vectorizer_insert_latency_seconds_bucket[5m])) > 0.25` | 5 m   | warning  |
| `VectorizerFailedInserts`           | `rate(vectorizer_insert_requests_total{status="error"}[5m]) > 1`                      | 2 m   | warning  |
| `VectorizerRapidVectorGrowth`       | `rate(vectorizer_vectors_total[1h]) > 100000`                                         | 10 m  | info     |

### `vectorizer_replication`

| Alert                                   | Expression                                                     | `for` | Severity |
| --------------------------------------- | -------------------------------------------------------------- | ----- | -------- |
| `VectorizerHighReplicationLag`          | `vectorizer_replication_lag_ms > 1000`                         | 5 m   | warning  |
| `VectorizerCriticalReplicationLag`      | `vectorizer_replication_lag_ms > 10000`                        | 2 m   | critical |
| `VectorizerReplicationBacklog`          | `vectorizer_replication_operations_pending > 10000`            | 5 m   | warning  |
| `VectorizerHighReplicationBandwidth`    | `rate(vectorizer_replication_bytes_sent_total[5m]) > 10 MiB/s` | 10 m  | info     |

**Lag-unit caveat**: the thresholds `> 1000` / `> 10000` are historical
millisecond values but the metric currently emits lag measured in *operations*
(see the [Replication metrics](#replication-metrics) section). On a
write-heavy master a 10 k-op backlog is a meaningful page; on a quiet master
it may not be. Re-baseline these thresholds against your traffic profile.

### `vectorizer_system`

| Alert                               | Expression                                           | `for` | Severity |
| ----------------------------------- | ---------------------------------------------------- | ----- | -------- |
| `VectorizerHighMemoryUsage`         | `vectorizer_memory_usage_bytes > 4 GiB`              | 10 m  | warning  |
| `VectorizerCriticalMemoryUsage`     | `vectorizer_memory_usage_bytes > 8 GiB`              | 5 m   | critical |
| `VectorizerLowCacheHitRate`         | `100 * cache_hit_rate(5m) < 60`                      | 10 m  | warning  |
| `VectorizerHighAPIErrorRate`        | `rate(vectorizer_api_errors_total[5m]) > 10`         | 2 m   | critical |

### `vectorizer_availability`

| Alert                             | Expression                                           | `for` | Severity |
| --------------------------------- | ---------------------------------------------------- | ----- | -------- |
| `VectorizerNoSearchActivity`      | `rate(vectorizer_search_requests_total[10m]) == 0`   | 30 m  | info     |
| `VectorizerNoInsertActivity`      | `rate(vectorizer_insert_requests_total[10m]) == 0`   | 1 h   | info     |

### Severity tiers

| Severity   | Response time | Escalation        | Example                       |
| ---------- | ------------- | ----------------- | ----------------------------- |
| `critical` | Immediate     | Page on-call      | Memory > 8 GiB, lag > 10 s    |
| `warning`  | ≤ 1 h         | Ticket + email    | Degraded p95 latency, backlog |
| `info`     | Best effort   | Log only          | Capacity-planning signals     |

The full rule file also ships a sample `alertmanager.yml` route-tree (PagerDuty
for `critical`, Slack for `warning`, null receiver for `info`).

## Grafana dashboard

`docs/grafana/vectorizer-dashboard.json` is the reference dashboard, titled
**"Vectorizer Monitoring Dashboard"**. It targets a Prometheus datasource at
`/prometheus/metrics`.

### Import

1.  Grafana → **Dashboards** → **New** → **Import**.
2.  Upload the JSON file or paste its contents.
3.  Select your Prometheus datasource when prompted.

### Panels

| Panel                                    | Queries / Metrics                                                                           |
| ---------------------------------------- | ------------------------------------------------------------------------------------------- |
| Total Collections                        | `vectorizer_collections_total`                                                              |
| Total Vectors                            | `vectorizer_vectors_total`                                                                  |
| Memory Usage                             | `vectorizer_memory_usage_bytes`                                                             |
| Search Requests (req/sec)                | `sum by (search_type) (rate(vectorizer_search_requests_total[5m]))`                         |
| Search Latency (p95 / p99)               | `histogram_quantile(0.95 \| 0.99, sum by (search_type, le) (rate(vectorizer_search_latency_seconds_bucket[5m])))` |
| Active Tenants                           | `vectorizer_hub_active_tenants`                                                             |
| HiveHub API Requests by Tenant (Top 10)  | `topk(10, sum by (tenant_id) (rate(vectorizer_hub_api_requests_total[5m])))`                |
| Quota Usage by Tenant (Top 10)           | `topk(10, vectorizer_hub_quota_usage{quota_type="storage"})` + `{quota_type="collections"}` |
| HiveHub API Latency (p95 / p99)          | `histogram_quantile(…, sum by (method, le) (rate(vectorizer_hub_api_latency_seconds_bucket[5m])))` |
| Quota Exceeded Events (5 m window)       | `sum by (quota_type) (increase(vectorizer_hub_quota_exceeded_total[5m]))`                   |
| Successful Backups (1 h)                 | `sum(increase(vectorizer_hub_backup_operations_total{status="success"}[1h]))`               |
| Failed Backups (1 h)                     | `sum(increase(vectorizer_hub_backup_operations_total{status="failure"}[1h]))`               |
| Successful Usage Reports (1 h)           | `sum(increase(vectorizer_hub_usage_reports_total{status="success"}[1h]))`                   |
| Avg Quota Check Latency                  | `rate(vectorizer_hub_quota_check_latency_seconds_sum[5m]) / rate(…_count[5m])`              |

The dashboard is biased toward HiveHub (multi-tenant) operation. For
single-node deployments most Hub panels stay empty; clone the dashboard and
remove them, or duplicate the Search/Indexing panels per-collection with a
`collection=~"$collection"` variable instead.

## Tuning

### Indicators of overload (scale out / add replicas)

- `vectorizer_search_latency_seconds` p95 approaching the 100 ms bucket with
  request rate rising.
- `vectorizer_memory_usage_bytes` trending toward the `HighMemoryUsage`
  threshold (4 GiB default) while vector count is stable — a memory leak vs.
  working-set growth signature.
- `vectorizer_api_errors_total` rate rising against a flat error endpoint —
  typically rate-limit or backpressure.

### Indicators of data-plane issues

- `vectorizer_replication_lag_ms` climbing despite steady write rate —
  network partition, slow replica disk, or GC pauses on the replica.
- `vectorizer_replication_operations_pending` monotonically growing —
  master-side WAL is filling because replicas cannot drain it. Capacity-plan
  disk and network for the replication path.
- `vectorizer_replication_bytes_received_total` diverging from
  `_bytes_sent_total` beyond WAL truncation — check the replica's
  reconnection path (recent `fix/replication-over-apply-on-reconnect` branch
  fixes related over-apply bugs).

### Indicators of bad queries / poor collection config

- `vectorizer_search_results_count` skewed toward the 0 or 1-result buckets
  — the collection may have the wrong embedding model or distance metric.
- `vectorizer_search_latency_seconds` tail with a flat request rate — large
  `top_k` or missing HNSW tuning. See `CollectionConfig.hnsw_config`.
- Low `vectorizer_cache_requests_total{result="hit"}` ratio (< 60 % triggers
  the `LowCacheHitRate` alert) — query distribution is too diverse for the
  current cache size, or callers are generating unique queries per request.

## Troubleshooting

### `/prometheus/metrics` returns 404

Verify you are using the correct path. `/metrics` (no prefix) is the
file-watcher JSON feed. The Prometheus exposition lives at
`/prometheus/metrics`. Both are registered unconditionally regardless of
feature flags; a 404 means the request did not reach Axum (wrong port / proxy
misconfiguration).

### `/prometheus/metrics` returns 401 in production

Ensure no reverse proxy is adding auth in front of the server. The Vectorizer
server's own production-mode middleware explicitly bypasses auth for this path
(`routing.rs:824-835`), so a 401 means upstream enforcement.

### `/prometheus/metrics` is empty or only shows `# HELP` lines

Every metric family registers on server start via
`monitoring::init()` → `METRICS.register(&registry)`. Counters and histograms
only appear after the first observation — so a freshly started server with no
traffic will show the family lines but no samples. Generate a test query or
insert and re-scrape.

### A metric family is missing entirely

Grep `crates/vectorizer/src/monitoring/metrics.rs` for the metric name. If
it's defined there it will be registered; if the name comes from a third-party
component (node_exporter-style host metrics, GPU metrics on a CPU build), it
won't appear in Vectorizer's output. Pair with `node_exporter` /
`cAdvisor` for host telemetry.

### File-watcher values look zero on macOS / Windows

`FileWatcherMetrics.system.*` uses `/proc/self/*` lookups
(`file_watcher/metrics.rs:560-618`) which exist only on Linux. On other
platforms the collector falls back to `0` or `available_parallelism`. Use the
Prometheus `vectorizer_memory_usage_bytes` gauge for cross-platform memory
usage — it goes through `memory_stats::memory_stats()` and works everywhere.

### I want file-watcher stats in Prometheus format

They are intentionally JSON-only today. Options:

1.  Run `https://github.com/prometheus/json_exporter` against
    `http://vectorizer:15002/metrics` with a jsonpath-to-metric mapping.
2.  Or extend `monitoring::metrics::Metrics` with additional families and
    wire `MetricsCollector` to update them (follow the
    `vectorizer_replication_*` pattern in
    `replication/master.rs`). Remember to register new families in
    `Metrics::register` and document them here.

### Replication-lag panel reads impossibly low / high

See the caveat in [Replication metrics](#replication-metrics): the gauge's
unit is *operations pending*, not milliseconds, despite its
`vectorizer_replication_lag_ms` name. Convert to wall-clock lag by multiplying
by your observed per-operation cost (≈ 1 ms per op is the default
approximation in the source).
