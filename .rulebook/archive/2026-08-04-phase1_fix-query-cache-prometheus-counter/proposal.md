# Proposal: phase1_fix-query-cache-prometheus-counter

## Why

`cache::query_cache_behaviour::prometheus_counter_increments_on_every_cache_get`
fails: it expects three pre-insert reads to register three `miss` increments on
`METRICS.cache_requests_total{cache="query"}`, and the counter does not move at
all (`left: 0.0, right: 3.0`).

Either the query cache's `get` no longer touches that counter, or it writes
different label values than the test reads. Both mean the cache hit/miss
metric the dashboard and Prometheus scrape is not being produced, so cache
effectiveness is invisible in production.

Found while validating `phase1_bump-openraft-alpha30`. It is **not** caused by
that bump: reproduced on a clean worktree at commit `cd6100cd` with
openraft still at `=0.10.0-alpha.22`, and the openraft bump moved only the five
`openraft*` crates in `Cargo.lock` (verified by diff), none of which are on the
cache path. It fails in isolation too, so it is not test-ordering pollution.

## What Changes

- Establish which side is wrong: the instrumentation in the query cache `get`
  path, or the label values the test asserts against.
- Fix the instrumentation so a cache read increments
  `cache_requests_total` with the `hit` / `miss` label, and make the test
  assert the shape that actually ships.
- Check the sibling caches for the same gap, since the counter is shared and
  labelled by cache name.

## Impact

- Affected specs: monitoring/metrics
- Affected code: `crates/vectorizer/src/cache/`,
  `crates/vectorizer/src/monitoring/metrics.rs`,
  `crates/vectorizer/tests/cache/query_cache_behaviour.rs`
- Breaking change: NO
- User benefit: cache hit rate becomes observable again in Prometheus and on
  the dashboard.
