## 1. Implementation

- [ ] 1.1 Re-run each failing test with `--nocapture` and record the
  exact panic message + field name in this task's notes.
- [ ] 1.2 Update `sdks/rust/src/models.rs` response structs to match
  the current emitters:
    - `ListCollectionsResponse`: field rename if needed (`total` vs
      `total_collections`).
    - `CollectionInfo`: add `normalization`, `quantization`,
      `size.{index_bytes, payload_bytes, total_bytes}` sub-structs
      with `#[serde(default)]` so pre-v3 servers still deserialize.
    - `BatchInsertResponse`: `{collection, inserted, failed, count,
      results}` (match F1).
    - `SearchResponse`: add `query_type`, `total_results`.
- [ ] 1.3 Drop `#[serde(deny_unknown_fields)]` on response models;
  keep it on request models so a typo in the caller's payload is
  still caught.
- [ ] 1.4 Update `sdks/rust/tests/integration_tests.rs` assertions if
  any tests hardcoded expected-field counts.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`sdks/rust/CHANGELOG.md` under `3.0.0 > Fixed` + response-shape
  block in `docs/users/sdks/RUST.md` if it documents the shapes).
- [ ] 2.2 Write tests covering the new behavior (the repaired
  `test_list_collections`, `test_get_collection_info`,
  `test_serialization` are the tests; add one fresh test that
  asserts the response models tolerate one extra unknown field at
  every level).
- [ ] 2.3 Run tests and confirm they pass
  (`cargo test -p vectorizer-sdk --features rpc` — target 0
  failures).
