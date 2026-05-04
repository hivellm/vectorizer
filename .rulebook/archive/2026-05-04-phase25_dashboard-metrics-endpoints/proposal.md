# Proposal: phase25_dashboard-metrics-endpoints

## Why

The redesigned dashboard (PR #266, merged 2026-05-04) renders KPI cards,
Ring gauges, Bars, and Sparkline graphs across `OverviewPage`,
`MonitoringPage`, and `CollectionsPage`. A static audit on 2026-05-04
confirmed the bulk of those visuals are driven by a synthetic data
generator:

```ts
const SPARK = (n: number, base: number, amp: number): number[] =>
  Array.from({ length: n }, (_, i) => base + Math.sin(i / 2) * amp + Math.random() * amp * 0.3);
```

The `MonitoringPage` source even self-documents the gap:

```ts
// TODO(stats-endpoint): the REST/MCP throughput breakdown, p99, 5xx rate
// and SIMD primitive throughput stay synthetic — the backend does not yet
// expose these in /stats or /health. WAL sequence/size/checkpoint also
// stay synthetic until /health (or a /wal endpoint) starts emitting them.
```

End-to-end consequence on `vectorizer:3.3.0`:

- `Ring` gauges for CPU%, MEMORY%, CONNECTIONS on `OverviewPage` are fed
  by `useMetrics()` which has no backend source — they render as `0%`.
- The Sparkline above "Total throughput" on `MonitoringPage` is
  `SPARK(60, 2400, 220)` → a sine wave, not real QPS.
- "MAP score +8.9%" + "Recall@10 98.4%" + "compression ratio 4.0×" on
  `OverviewPage` are hardcoded literals, not computed from the active
  collections.
- "vectorizer 3.0.0" + bind-address strings on `OverviewPage:188-190`
  are hardcoded — the running container is `3.3.0` and binds to
  whatever the user configured.

User report (2026-05-04): "os gráficos do dashboard só me parece
placeholders nada está funcional" — confirmed by the audit.

This task ships the backend metric endpoints the dashboard needs so the
charts become real. A follow-up task (phase24) handles the dashboard
side: drop `SPARK`, drop hardcoded literals, wire to the new endpoints.

## What Changes

Add a single new admin/observability endpoint plus extend an existing one:

1. **New `GET /metrics/runtime`** (admin-gated). Returns a snapshot of
   process-level + request-level metrics the dashboard needs:

   ```jsonc
   {
     "cpu_percent": 12.4,                 // process CPU, 0-100
     "memory_percent": 23.7,              // RSS / total mem, 0-100
     "memory_rss_bytes": 124857600,
     "memory_total_bytes": 17179869184,
     "active_connections": 8,             // active HTTP conns + WS clients
     "qps_window_60s": 142.3,             // rolling 60s queries/sec
     "throughput_by_route": [             // top-N routes by RPS
       {"route": "/insert_texts", "qps": 12.0, "p50_ms": 8.2, "p99_ms": 41.0},
       {"route": "/collections/*/search/text", "qps": 88.0, "p50_ms": 2.4, "p99_ms": 18.1}
     ],
     "error_rate_5xx_60s": 0.001,         // 0..1 fraction
     "wal": {                             // pull from existing replication state
       "current_seq": 482919,
       "size_bytes": 12582912,
       "last_checkpoint_at": 1714828800,
       "last_checkpoint_seq": 482800
     },
     "uptime_seconds": 3712
   }
   ```

   Implementation: a background sampler in `vectorizer-server` builds
   ring buffers of per-route latencies (already needed for slow_queries)
   and serves them through this endpoint. Process-level CPU/memory uses
   the `sysinfo` crate (already a transitive dep). Active connections
   counter is incremented/decremented in the axum middleware that
   `vectorizer-server` already has.

2. **Extend `GET /stats`** with `default_quantization` and
   `compression_ratio` so `OverviewPage`'s "Quantization" card reflects
   real config instead of "SQ-8bit · default · 4.0×". The numbers are
   already known by the server (per-collection `QuantizationConfig` —
   aggregate to the most-common config + compute its compression ratio).

3. **Extend `GET /collections/{n}` response** with a
   `vector_count_history` array (last N samples from a per-collection
   ring buffer) so `CollectionsPage`'s Sparkline can render real growth
   instead of `SPARK(40, vector_count/1000, 8)`. The ring buffer lives
   in the existing collection metadata; sampler ticks every 60s.

The new sampler is bounded (small ring buffers, fixed sample interval)
and only allocates when at least one client is connected.

## Impact

- Affected code:
  - `crates/vectorizer-server/src/server/rest_handlers/metrics.rs`
    (new) — `GET /metrics/runtime` handler + sampler bootstrap
  - `crates/vectorizer-server/src/server/middleware.rs` — connection
    counter + per-route latency ring buffer
  - `crates/vectorizer-server/src/server/rest_handlers/admin.rs` —
    extend `GET /stats` with the two new aggregated fields
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs`
    — extend `GET /collections/{n}` with `vector_count_history`
  - `crates/vectorizer/src/db/collection/stats.rs` (or new file) —
    per-collection sampler ring buffer
- Affected specs:
  - new `phase25_dashboard-metrics-endpoints/specs/runtime-metrics/spec.md`
- Breaking change: NO (additive — `/metrics/runtime` is new; `/stats`
  and `/collections/{n}` add new optional fields that the SDKs ignore
  if absent)
- User benefit: Dashboard graphs become real instead of sine + random.
  Operators can finally see live process load, request throughput,
  per-route p99 latency, and per-collection growth without `top` /
  `htop` / external Prometheus.
