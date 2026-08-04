## 1. Implementation

- [x] 1.1 Decide lazy-delete vs filter-only, and record the reasoning — filter-only: `search` holds the index read lock while building results and a delete needs the write lock, so reclaiming on read risks a deadlock; the reaper already reclaims within one interval
- [x] 1.2 Filter expired vectors out of `get_vector` — applied on `CollectionType::get_vector` so the CPU, GPU and sharded backends cannot drift; `get_vector_including_expired` is the raw accessor
- [x] 1.3 Filter expired hits out of the search and paginated-list paths — `search`, `search_explained`, RPC `vectors.list` and REST `list_vectors`; the listings filter before counting so `total` matches the page; `get_all_vectors` stays raw for the reaper and for saves

## 2. Tail (docs + tests — check or waive with tailWaiver)

- [x] 2.1 Update or create documentation covering the implementation — CHANGELOG, plus corrected the sweep-latency notes in `docs/prometheus/METRICS.md` and `docs/specs/VECTORIZER_RPC.md` that said reads do not filter
- [x] 2.2 Write tests covering the new behavior — three tests proving an expired vector is unreadable and not a search hit with no sweep involved, and that a future expiry is untouched; each asserts the vector is still stored, so it exercises the filter rather than a deletion
- [x] 2.3 Run tests and confirm they pass — 1046 vectorizer lib, 927 integration, 235 server lib, 4 REST get_vector, 23 lifecycle; also repaired the reaper's own tests, which the new filter had silently weakened (they asserted through `get_vector`, which now hides an expired vector whether or not the sweep deleted it)
