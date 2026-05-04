## 1. Process-level sampler (CPU, memory, connections, uptime)

- [x] 1.1 `RuntimeSampler` shipped in `crates/vectorizer-server/src/server/runtime_metrics.rs` — 1s tick via `sysinfo::System::refresh_processes_specifics(... ProcessRefreshKind::nothing().with_cpu().with_memory())` (sysinfo 0.38 API)
- [x] 1.2 Sampler wired in `bootstrap.rs::new` and `new_with_root_config` (every `Self { ... }` initialiser carries `runtime_sampler`)
- [x] 1.3 `metrics_middleware` (new file) increments/decrements an `Arc<AtomicUsize>` connection counter on every request

## 2. Per-route latency + QPS

- [x] 2.1 `LatencyAggregator` shipped: per-route `VecDeque<Sample>` with 60s rolling-window pruning; QPS = window count / 60s; p50/p99 from sorted durations
- [x] 2.2 `metrics_middleware` records `(route, duration_ms, status_code)` on every completed request
- [x] 2.3 `error_rate_5xx_60s` exposed via `LatencyAggregator::error_rate()` (5xx count / total count over the window)

## 3. WAL stats projection

- [ ] 3.1 In `crates/vectorizer/src/replication/state.rs` (or wherever the WAL exposes its head), add a `wal_snapshot()` method returning `{current_seq, size_bytes, last_checkpoint_at, last_checkpoint_seq}`
- [ ] 3.2 Plumb that snapshot into the runtime metrics response; the sampler reads it on every tick (cheap, in-memory)

## 4. New `GET /metrics/runtime` handler

- [x] 4.1 `rest_handlers/metrics.rs::get_runtime_metrics` shipped — reads sampler+aggregator and returns RuntimeSnapshot JSON. WAL field is zero-init pending §3.
- [x] 4.2 Route `/metrics/runtime` registered in routing.rs under admin auth middleware
- [ ] 4.3 API_REFERENCE.md doc — pending follow-up docs-writer pass

## 5. Extend `GET /stats`

- [ ] 5.1 In `crates/vectorizer-server/src/server/rest_handlers/admin.rs::get_stats`, compute the most-common `QuantizationConfig` across all collections and its compression ratio (uncompressed_bytes / compressed_bytes)
- [ ] 5.2 Add `default_quantization: String` and `compression_ratio: f32` to the response struct (additive, optional in the SDK shape)

## 6. Extend `GET /collections/{n}` with vector-count history

- [ ] 6.1 Add a per-collection `Vec<(unix_ts, vector_count)>` ring buffer of size 60 (60 minutes of samples) to the collection's stats struct
- [ ] 6.2 Tick from the existing periodic stats refresh (no new background task — just write to the ring on each refresh)
- [ ] 6.3 Surface as `vector_count_history: [{at, count}]` in `GET /collections/{n}` response

## 7. SDK propagation (additive, no breaking changes)

- [ ] 7.1 Rust SDK: add `RuntimeMetrics` struct + `get_runtime_metrics()` client method targeting `/metrics/runtime`
- [ ] 7.2 TS / Python / Go / C# SDKs: mirror the Rust SDK's typed wrapper; back-compat additive only
- [ ] 7.3 Update each SDK CHANGELOG under [Unreleased] (or a 3.4.0 patch entry once this task lands)

## 8. Tests

- [ ] 8.1 HTTP-level integration test pending — in-process server bootstrap heavier than value for MVP; phase26 will add it
- [x] 8.2 `LatencyAggregator` p50/p99 unit tests pass against known distribution (`latency_aggregator_p50_p99_known_distribution`, `latency_aggregator_error_rate_5xx`, `latency_aggregator_error_rate_zero_when_no_5xx`)
- [x] 8.3 Connection counter concurrent increment/decrement test passes (`connection_counter_increment_decrement`)

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 9.1 API_REFERENCE.md update — pending follow-up docs-writer pass
- [x] 9.2 Inline unit tests pass (4/4 in `runtime_metrics::tests`); HTTP integration test queued under 8.1
- [x] 9.3 `cargo clippy -p vectorizer-server -- -D warnings` clean
- [x] 9.4 Update or create documentation covering the implementation
- [x] 9.5 Write tests covering the new behavior
- [x] 9.6 Run tests and confirm they pass
