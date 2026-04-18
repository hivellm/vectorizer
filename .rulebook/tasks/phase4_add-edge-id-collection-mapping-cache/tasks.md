## 1. In-memory cache

- [ ] 1.1 Add `edge_index: Arc<DashMap<String, String>>` to `GraphApiState`.
- [ ] 1.2 Populate on `POST /graph/edges`; remove on `DELETE /graph/edges/{id}`.
- [ ] 1.3 Replace the scan at `src/api/graph.rs:668` with an index lookup; retain scan as a safety fallback when the index misses.

## 2. Rebuild on startup

- [ ] 2.1 Add a `rebuild_edge_index(&VectorStore)` function that walks every collection's graph once on server start.
- [ ] 2.2 Call it from the server bootstrap path.

## 3. Tests

- [ ] 3.1 Integration: create edge in collection A, delete by id, assert `O(1)` path took it.
- [ ] 3.2 Integration: crash simulation — drop the `edge_index`, call `rebuild`, assert delete still works.

## 4. Tail (mandatory)

- [ ] 4.1 Document the cache in the graph module README.
- [ ] 4.2 Tests above cover the new behavior.
- [ ] 4.3 Run `cargo test --all-features` and confirm pass.
