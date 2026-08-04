# Proposal: phase1_ttl-reaper-never-spawned

## Why

`TtlReaper` is implemented, exported from `db::mod` and instrumented, but
nothing in the codebase ever spawns it. `grep -rn 'TtlReaper::spawn'` outside
`src/db/ttl_reaper.rs` returns no hits, and no other reference to the type
exists outside the module's own re-export.

That makes vector expiry write-only: `vectors.set_expiry` (RPC) and its REST
counterpart stamp `__expires_at` into the payload, and no background sweep ever
removes the expired vectors. A caller who sets an expiry gets a value that is
recorded and then honoured by nobody — expired vectors keep occupying memory.

Found while auditing the `MetricsSink` wiring for
`phase1_fix-query-cache-prometheus-counter`: the reaper's Prometheus metrics
(`ttl_reaper_scans_total`, `ttl_reaper_lag_secs`, `ttl_vectors_expired_total`)
are registered and can never move, which is what surfaced the missing spawn.

## What Changes

- Decide the owner: bootstrap spawning one reaper per collection that has a
  TTL, or a single sweep task that walks the collections it finds.
- Spawn it via `spawn_with_metrics` with the real `PrometheusMetricsSink`, so
  the three already-registered metrics report.
- Establish whether the read path filters expired vectors today. That decides
  whether this is only a memory-growth bug or also a correctness bug in query
  results.
- Shut the reaper down cleanly on server shutdown (the `shutdown` flag exists).

## Impact

- Affected specs: db/ttl
- Affected code: `crates/vectorizer/src/db/ttl_reaper.rs`,
  `crates/vectorizer-server/src/server/core/bootstrap.rs`
- Breaking change: NO
- User benefit: an expiry actually expires — memory is reclaimed and expired
  vectors stop being served.
