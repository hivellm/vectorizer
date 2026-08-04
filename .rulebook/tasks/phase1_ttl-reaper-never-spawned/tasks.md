## 1. Implementation

- [ ] 1.1 Establish whether the read path already filters expired vectors, to know if this is memory-growth only or also wrong results
- [ ] 1.2 Spawn the reaper from bootstrap via `spawn_with_metrics` with the real `PrometheusMetricsSink`
- [ ] 1.3 Stop the reaper on server shutdown using its existing `shutdown` flag

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [ ] 2.1 Update or create documentation covering the implementation
- [ ] 2.2 Write tests covering the new behavior
- [ ] 2.3 Run tests and confirm they pass
