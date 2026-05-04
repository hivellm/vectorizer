## 1. Process-level sampler (CPU, memory, connections, uptime)

- [ ] 1.1 Add a `RuntimeSampler` background task in `crates/vectorizer-server/src/server/runtime_metrics.rs` (new) that ticks every 1s using `sysinfo::System::refresh_process` and stores the latest snapshot in an `Arc<RwLock<RuntimeSnapshot>>`
- [ ] 1.2 Wire the sampler into the server bootstrap (`crates/vectorizer-server/src/server/core/bootstrap.rs`) so it starts with the Tokio runtime and stops on graceful shutdown
- [ ] 1.3 Add an axum middleware that increments/decrements an `AtomicUsize` connection counter on each HTTP request

## 2. Per-route latency + QPS

- [ ] 2.1 Add a `LatencyAggregator` that maintains, per route template (e.g. `/collections/:name/search/text`), a 60s rolling histogram (HDR or quantile-est crate) — bounded memory, lock-free hot path
- [ ] 2.2 Wire it into the existing axum tracing middleware so every completed request records `(route_template, duration_ms, status_code)`
- [ ] 2.3 Track 5xx counts per route in the same aggregator; expose `error_rate_5xx_60s` as `5xx_count / total_count` over the window

## 3. WAL stats projection

- [ ] 3.1 In `crates/vectorizer/src/replication/state.rs` (or wherever the WAL exposes its head), add a `wal_snapshot()` method returning `{current_seq, size_bytes, last_checkpoint_at, last_checkpoint_seq}`
- [ ] 3.2 Plumb that snapshot into the runtime metrics response; the sampler reads it on every tick (cheap, in-memory)

## 4. New `GET /metrics/runtime` handler

- [ ] 4.1 Add `crates/vectorizer-server/src/server/rest_handlers/metrics.rs` with a single handler that reads from the sampler + latency aggregator + WAL snapshot and returns the JSON shape documented in the proposal
- [ ] 4.2 Register the route in `crates/vectorizer-server/src/server/core/routing.rs` under `/metrics/runtime`, gated by the existing admin auth middleware
- [ ] 4.3 Document the endpoint in `docs/users/api/API_REFERENCE.md` with a full example response

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

- [ ] 8.1 Add `crates/vectorizer-server/tests/runtime_metrics.rs` integration test that boots the server, fires N requests against several routes in parallel, then asserts `/metrics/runtime` returns: `qps_window_60s > 0`, the right routes appear in `throughput_by_route`, and `error_rate_5xx_60s` matches the count of 5xx responses
- [ ] 8.2 Unit test the `LatencyAggregator` quantile math against a known input distribution (asserts p50 + p99 within ±5%)
- [ ] 8.3 Unit test the connection counter increment/decrement under concurrent requests

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 9.1 Update `docs/users/api/API_REFERENCE.md` runtime-metrics + extended /stats sections
- [ ] 9.2 Run `cargo test -p vectorizer-server --test runtime_metrics` and confirm pass
- [ ] 9.3 Run `cargo clippy -p vectorizer-server -- -D warnings` clean
