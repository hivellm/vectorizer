## 1. Implementation

- [ ] 1.1 Relax `QdrantCreateCollectionRequest` at
  `crates/vectorizer/src/models/qdrant/collection.rs:306` to accept
  `{vectors: QdrantVectorsConfig, ...all other fields Option<>}` at
  the top level (drop the mandatory `config:` wrapper; support both
  `{config: {vectors: ...}}` and `{vectors: ...}` via a custom
  Deserialize impl or a `#[serde(untagged)]` enum).
- [ ] 1.2 Mark every non-`vectors` field of `QdrantCollectionConfig`
  as `Option<>` + `#[serde(default)]`.
- [ ] 1.3 Fill server-side defaults in
  `crates/vectorizer-server/src/server/qdrant/handlers.rs`
  `create_collection` when the optional blocks arrive as `None`.
- [ ] 1.4 Sweep sibling Qdrant-compat endpoints for the same pattern
  (`UpdateCollectionRequest`, `UpsertPointsRequest`,
  `SearchPointsRequest`, `ScrollRequest`); relax each request shape
  so it matches Qdrant's upstream REST spec.
- [ ] 1.5 Verify against a real Qdrant client (e.g.
  `pip install qdrant-client && python -c
  'from qdrant_client import QdrantClient;
  c=QdrantClient(url="http://127.0.0.1:15002/qdrant");
  c.create_collection("probe", vectors_config={"size":4,"distance":"Cosine"})'`).

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`docs/users/api/QDRANT.md` request-shape block matching upstream
  docs; `CHANGELOG.md > 3.0.0 > Fixed`).
- [ ] 2.2 Write tests covering the new behavior (integration test at
  `crates/vectorizer/tests/api/rest/qdrant_compat_minimal_real.rs`
  that creates a collection with `{vectors: {size:4, distance:
  "Cosine"}}` only, upserts 3 points, searches, and asserts
  response shapes match Qdrant's REST spec).
- [ ] 2.3 Run tests and confirm they pass (new integration test +
  `cargo test --workspace --lib` — target 0 regressions).
