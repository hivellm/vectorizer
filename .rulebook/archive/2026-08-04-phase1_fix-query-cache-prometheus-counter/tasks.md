## 1. Implementation

- [x] 1.1 Trace the query cache `get` path and record whether it touches `cache_requests_total`, and with which label values — `get` does call `metrics.cache_request("query", hit)`; the emission was never the problem, the sink was: `QueryCache::new` injects `NoopMetricsSink` and bootstrap used it
- [x] 1.2 Fix the side that is wrong (instrumentation or the test's expected labels) — production was wrong: bootstrap now injects `PrometheusMetricsSink`; the test builds its cache the same way instead of via the Noop constructor; the server test harness deliberately stays on Noop
- [x] 1.3 Audit the other caches sharing `cache_requests_total` for the same gap — `PrometheusMetricsSink` was injected only at the two auth sites; HiveHub `QuotaManager` had the same gap and is fixed, and `TtlReaper` turned out never to be spawned at all (tracked in phase1_ttl-reaper-never-spawned)

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [x] 2.1 Update or create documentation covering the implementation — CHANGELOG; `docs/prometheus/METRICS.md` source note plus a new "family shows up but samples never move" troubleshooting entry; corrected `docs/architecture/caching.md`, which claimed this counter had already been fixed
- [x] 2.2 Write tests covering the new behavior — the existing test now exercises the shipping path; production wiring verified end to end on a container (two identical text searches move miss and hit on /prometheus/metrics), with the pre-fix image as a negative control leaving both at zero
- [x] 2.3 Run tests and confirm they pass — 927 passed / 0 failed in the vectorizer suite (was 926/1 with this test failing), 235 in vectorizer-server, clippy clean
