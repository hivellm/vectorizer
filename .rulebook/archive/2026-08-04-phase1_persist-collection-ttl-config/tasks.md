## 1. Implementation
- [x] 1.1 Pick the persistence surface (sidecar / `CollectionIndex.metadata` / `CollectionConfig`) and record why
- [x] 1.2 Persist a TTL on set and clear, with the write path taking an explicit path so tests do not touch the real data dir
- [x] 1.3 Load persisted TTLs at bootstrap through `VectorStore::set_collection_ttl`
- [x] 1.4 Remove the process-scoped caveat from the API reference, RPC spec, all five SDK doc comments, and the CHANGELOG bullet

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Update or create documentation covering the implementation
- [x] 2.2 Write tests covering the new behavior
- [x] 2.3 Run tests and confirm they pass
