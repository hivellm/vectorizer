## 1. Implementation

- [x] 1.1 Extract the chunk+embed+store body of `insert_text` into a
  shared helper `insert_one_text(&state, tenant_ctx, collection, text,
  metadata, public_key, auto_chunk, chunk_size, chunk_overlap)` living
  in a new `rest_handlers/insert_shared.rs` (or inside `insert.rs` as
  `pub(super) async fn insert_one_text`).
- [x] 1.2 Rewrite `batch_insert_texts` in `vectors.rs` to parse
  `collection` + `texts[]` (items `{id?, text, metadata?}`), loop
  through `insert_one_text` per item, and return `{inserted, failed,
  results}` with a 200 on partial success.
- [x] 1.3 Rewrite `insert_texts` the same way (same handler body —
  consider routing both endpoints to one function).
- [x] 1.4 Validate payload shape: return 400 if `collection` missing,
  if `texts` missing, if `texts` empty, or if a per-item `text` is
  missing.
- [x] 1.5 Propagate the real `METRICS.insert_latency_seconds` timer
  and the replication hook that `insert_text` already wires up.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  (CHANGELOG.md under `3.0.0 > Fixed`; `docs/specs/REST.md` if it
  documents these endpoints).
- [x] 2.2 Write tests covering the new behavior (integration test at
  `tests/api/rest/batch_insert_real.rs` that creates a collection,
  POSTs 10 texts via `/batch_insert`, verifies `list_vectors` returns
  10 entries with matching metadata, and asserts response shape
  `{inserted:10, failed:0, results:[{id,status:"ok"}; 10]}`; same for
  `/insert_texts`).
- [x] 2.3 Run tests and confirm they pass
  (`cargo test --workspace --lib --all-features` plus the new
  integration test).
