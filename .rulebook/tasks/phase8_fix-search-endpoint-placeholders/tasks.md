## 1. Implementation

- [x] 1.1 Extract the core of `search_vectors_by_text`
  (`crates/vectorizer-server/src/server/rest_handlers/search.rs:37`)
  into a private helper `fn do_vector_search(state, collection_name,
  query_embedding, limit, threshold, filter, tenant_ctx) ->
  Result<Value>` that handles collection lookup, tenant validation,
  query cache hit, and vector HNSW search.
- [x] 1.2 Rewrite `search_vectors` (same file, line 351) to parse
  `vector` (required), `collection` (required when path parameter is
  absent), `limit`, `threshold`, `filter`; validate vector length
  against collection dimension; call `do_vector_search` with the raw
  query vector; return the shared response shape.
- [x] 1.3 Register the handler for `/collections/{name}/search` with a
  `Path<String>` extractor so the name comes from the URL; fall back
  to `payload["collection"]` for the bare `/search` route (or split
  into two small handlers if `Path` extractor makes that awkward).
- [x] 1.4 Apply the query cache with `QueryKey::from_vector(vector,
  limit, threshold)` and verify the hit path returns from cache
  identically to the text handler.
- [x] 1.5 Document response shape in a doc comment over the handler.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  (CHANGELOG.md under `3.0.0 > Fixed`; `docs/specs/REST.md` if it
  documents `/search`).
- [x] 2.2 Write tests covering the new behavior (integration test at
  `tests/api/rest/vector_search_real.rs` that creates a collection
  with dim 128, inserts 10 known vectors, POSTs `/search` with a
  target query vector, asserts top-1 is the expected id with score
  close to 1.0, and asserts the same via `/collections/{name}/search`).
- [x] 2.3 Run tests and confirm they pass
  (`cargo test --workspace --lib --all-features` plus the new
  integration test).
