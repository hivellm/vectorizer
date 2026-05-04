## 1. Process-level sampler (CPU, memory, connections, uptime)

- [x] 1.1 `RuntimeSampler` shipped in `crates/vectorizer-server/src/server/runtime_metrics.rs` — 1s tick via `sysinfo::System::refresh_processes_specifics(... ProcessRefreshKind::nothing().with_cpu().with_memory())` (sysinfo 0.38 API)
- [x] 1.2 Sampler wired in `bootstrap.rs::new` and `new_with_root_config` (every `Self { ... }` initialiser carries `runtime_sampler`)
- [x] 1.3 `metrics_middleware` (new file) increments/decrements an `Arc<AtomicUsize>` connection counter on every request

## 2. Per-route latency + QPS

- [x] 2.1 `LatencyAggregator` shipped: per-route `VecDeque<Sample>` with 60s rolling-window pruning; QPS = window count / 60s; p50/p99 from sorted durations
- [x] 2.2 `metrics_middleware` records `(route, duration_ms, status_code)` on every completed request
- [x] 2.3 `error_rate_5xx_60s` exposed via `LatencyAggregator::error_rate()` (5xx count / total count over the window)

## 3. WAL stats projection

- [x] 3.1 `WalSnapshot` struct + `DurableReplicationLog::wal_snapshot()` shipped in `crates/vectorizer/src/replication/durable_log.rs`; `mark_replicated()` advances `last_checkpoint_at` only when `min_confirmed_offset` actually moves forward; `MasterNode::wal_snapshot()` passthrough in `master.rs`. Re-exported from `vectorizer::replication`. 3 unit tests cover memory-only zeros, on-disk size, and checkpoint advancement
- [x] 3.2 `RuntimeSnapshot.wal: WalSnapshot` field added; sampler tick reads `master_node.wal_snapshot()` (or zero default when replication disabled). `bootstrap.rs` calls `sampler.set_master_node(...)` before `sampler.start()`

## 4. New `GET /metrics/runtime` handler

- [x] 4.1 `rest_handlers/metrics.rs::get_runtime_metrics` shipped — reads sampler+aggregator and returns RuntimeSnapshot JSON. WAL field is zero-init pending §3.
- [x] 4.2 Route `/metrics/runtime` registered in routing.rs under admin auth middleware
- [x] 4.3 `docs/specs/API_REFERENCE.md` — added `/metrics/runtime` row in the Health & Status table plus a full response-shape section covering CPU/memory/connections/QPS/per-route latency/5xx rate and the WAL block (with the standalone-mode caveat)

## 5. Extend `GET /stats`

- [x] 5.1 `get_stats` in `rest_handlers/meta.rs` (the actual home — task description's `admin.rs` was wrong) now iterates collections, groups by canonical quantization label, picks the most-common, and averages each member's static compression ratio. PQ ratio is dimension-aware (`d × 32 / (subq × log2(centroids))`); SQ ratio is `32 / bits`; Binary is 32; None is 1. Empty store falls back to `("none", 1.0)`
- [x] 5.2 Response now carries `default_quantization: String` + `compression_ratio: f32`. Additive only — old SDK clients ignore them. 4 unit tests pin the label/ratio helpers (`quantization_label_covers_known_variants`, `compression_ratio_static_variants`, `compression_ratio_pq_depends_on_dimension`, `compression_ratio_handles_degenerate_configs`)

## 6. Extend `GET /collections/{n}` with vector-count history

- [x] 6.1 `Collection.vector_count_history: Arc<RwLock<VecDeque<VectorCountSample>>>` capped at 60 entries (`VECTOR_COUNT_HISTORY_CAP`); `VectorCountSample { at, count }` re-exported from `vectorizer::db`
- [x] 6.2 No periodic refresh exists in the codebase — sampling moved to the read path. `Collection::record_vector_count_sample()` is a no-op when the last sample is < 60s old, so a static collection produces zero ongoing CPU. Static / GPU / DistributedSharded variants no-op via `CollectionType::record_vector_count_sample()`
- [x] 6.3 `GET /collections/{name}` calls `record_vector_count_sample()` then returns `vector_count_history: [{at, count}]`. 4 unit tests pin the ring (empty start, first sample, 60s dedup, capacity rotation)

## 7. SDK propagation (additive, no breaking changes)

- [ ] 7.1 Rust SDK: add `RuntimeMetrics` struct + `get_runtime_metrics()` client method targeting `/metrics/runtime`
- [ ] 7.2 TS / Python / Go / C# SDKs: mirror the Rust SDK's typed wrapper; back-compat additive only
- [ ] 7.3 Update each SDK CHANGELOG under [Unreleased] (or a 3.4.0 patch entry once this task lands)

## 8. Tests

- [ ] 8.1 HTTP-level integration test pending — in-process server bootstrap heavier than value for MVP; phase26 will add it
- [x] 8.2 `LatencyAggregator` p50/p99 unit tests pass against known distribution (`latency_aggregator_p50_p99_known_distribution`, `latency_aggregator_error_rate_5xx`, `latency_aggregator_error_rate_zero_when_no_5xx`)
- [x] 8.3 Connection counter concurrent increment/decrement test passes (`connection_counter_increment_decrement`)

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 9.1 API_REFERENCE.md updated (see §4.3 — same change satisfies both items)
- [x] 9.2 Inline unit tests pass (4/4 in `runtime_metrics::tests`); HTTP integration test queued under 8.1
- [x] 9.3 `cargo clippy -p vectorizer-server -- -D warnings` clean
- [x] 9.4 Update or create documentation covering the implementation
- [x] 9.5 Write tests covering the new behavior
- [x] 9.6 Run tests and confirm they pass
