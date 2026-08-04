## 1. Implementation

- [x] 1.1 Establish whether the read path already filters expired vectors, to know if this is memory-growth only or also wrong results — it does not: `Payload::is_expired` had exactly one vector caller, the reaper's own sweep, so it was both memory growth and wrong results; with the reaper running the window is bounded to one sweep (tracked in phase1_filter-expired-vectors-on-read)
- [x] 1.2 Spawn the reaper from bootstrap via `spawn_with_metrics` with the real `PrometheusMetricsSink` — and made it store-wide: it enumerates collections per tick, because collections are created at runtime with no single choke point for a per-collection spawn
- [x] 1.3 Stop the reaper on server shutdown using its existing `shutdown` flag — the handle is held on `VectorizerServer` for its lifetime, since `Drop` signals shutdown; the harness runs without a reaper so a sweep cannot make expiry assertions timing-dependent

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [x] 2.1 Update or create documentation covering the implementation — CHANGELOG; a new TTL reaper section in `docs/prometheus/METRICS.md` documenting the three previously-undocumented families; the sweep-latency note on `vectors.set_expiry` in `docs/specs/VECTORIZER_RPC.md`
- [x] 2.2 Write tests covering the new behavior — 5 unit tests (deletion vs sparing, no-op sweep, missing collection, a collection created after the reaper started, stop halts the loop); the reaper had none
- [x] 2.3 Run tests and confirm they pass — 1043 vectorizer lib, 927 integration, 235 server lib, clippy clean; verified on a live container: expired vector gone in ~40s, survivor intact, scans 0->1, vectors_expired 1.0
