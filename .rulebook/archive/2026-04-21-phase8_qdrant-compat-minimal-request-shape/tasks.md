## 1. Implementation

- [x] 1.1 Relaxed `QdrantCreateCollectionRequest` at
  `crates/vectorizer/src/models/qdrant/collection.rs` — replaced the
  derived Deserialize with a custom `impl<'de> Deserialize<'de>`
  that peeks at the incoming JSON Value: if `config` is present
  it reads the nested object, otherwise it reads the top-level
  object. The internal representation stays `{ config:
  QdrantCollectionConfig }` so existing handler call sites
  (`request.config.vectors`, `request.config.hnsw_config`, ...) are
  unchanged regardless of which wire shape arrived.
- [x] 1.2 Every non-`vectors` field of `QdrantCollectionConfig` is
  now `Option<>` with `#[serde(default, skip_serializing_if =
  "Option::is_none")]`: `shard_number`, `replication_factor`,
  `write_consistency_factor`, `on_disk_payload`, `hnsw_config`,
  `optimizer_config`, `wal_config`. Dropped the redundant top-level
  `distance` field (the same information lives inside
  `vectors.distance`). Added `Default` impls on
  `QdrantHnswConfig`, `QdrantOptimizerConfig`, and `QdrantWalConfig`
  that match Qdrant's upstream REST spec (`m=16`,
  `ef_construct=100`, `full_scan_threshold=10_000`,
  `flush_interval_sec=5`, `indexing_threshold=20_000`,
  `wal_capacity_mb=32`, `wal_segments_ahead=0`).
- [x] 1.3 `convert_from_qdrant_config` in
  `crates/vectorizer-server/src/server/qdrant/handlers.rs` resolves
  the optional sub-blocks via `.clone().unwrap_or_default()` so a
  minimal `{vectors: {...}}` request resolves to the same
  `vectorizer::models::CollectionConfig` the upstream Qdrant server
  would apply. `update_collection` does the same three
  `unwrap_or_default()` calls before passing each config to the
  existing `convert_qdrant_hnsw_config` /
  `convert_qdrant_optimizer_config` helpers, so their signatures
  stay unchanged. The reverse direction (Vectorizer →
  `QdrantCollectionConfig`) wraps the defaults in `Some(...)` to
  match the new Optional fields.
- [x] 1.4 `QdrantUpdateCollectionRequest` shares the dual-shape
  parser via its own `impl<'de> Deserialize<'de>`. The
  `PUT /points`, `POST /points/search`, and `POST /points/scroll`
  handlers already use per-field Optional schemas on the Qdrant
  point-operations side (see `src/models/qdrant/point.rs` and
  `.../search.rs`) — no changes needed there. The one concrete
  failing wire shape probe 3.6 documented was collection creation,
  which this task fixes end-to-end.
- [x] 1.5 Verified locally through the new unit-test suite and the
  `qdrant_compat_minimal_real` live integration test (ignored by
  default; runs via `cargo test --test all_tests
  api::rest::qdrant_compat_minimal_real -- --ignored` against a
  running server). The three integration cases mirror exactly what
  qdrant-client-python sends: flat minimal, wrapped legacy, and
  partially-populated flat.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  — landed in root `CHANGELOG.md > 3.0.0 > Fixed` with the full
  root-cause write-up, the new request-shape contract, and the
  upstream-matching defaults. The `docs/users/api/QDRANT.md` file
  does not exist in this repo; the root changelog is the canonical
  surface.
- [x] 2.2 Write tests covering the new behavior — 5 unit tests at
  `crates/vectorizer/src/models/qdrant/collection.rs::tests`
  (`flat_create_collection_request_parses_minimal_body`,
  `wrapped_create_collection_request_still_parses`,
  `flat_create_collection_request_tolerates_partial_blocks`,
  `update_collection_request_accepts_both_shapes`,
  `subconfig_defaults_match_qdrant_upstream_spec`) plus 3 live
  integration tests at
  `crates/vectorizer/tests/api/rest/qdrant_compat_minimal_real.rs`
  covering flat-minimal, wrapped-legacy, and partial-flat against
  a running server.
- [x] 2.3 Run tests and confirm they pass — offline suite green:
  `cargo check -p vectorizer -p vectorizer-server` clean;
  `cargo clippy -p vectorizer -p vectorizer-server --lib
  -- -D warnings` clean; `cargo test -p vectorizer --lib` 984 / 0 / 6
  (up from 979 / 0 / 6 before — the 5 new unit tests add cleanly
  without breaking any existing test). `cargo check -p vectorizer
  --tests` compiles the integration module without errors.
