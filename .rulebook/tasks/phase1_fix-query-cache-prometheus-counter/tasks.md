## 1. Implementation

- [ ] 1.1 Trace the query cache `get` path and record whether it touches `cache_requests_total`, and with which label values
- [ ] 1.2 Fix the side that is wrong (instrumentation or the test's expected labels)
- [ ] 1.3 Audit the other caches sharing `cache_requests_total` for the same gap

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [ ] 2.1 Update or create documentation covering the implementation
- [ ] 2.2 Write tests covering the new behavior
- [ ] 2.3 Run tests and confirm they pass
