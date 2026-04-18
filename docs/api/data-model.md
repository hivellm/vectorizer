# Data model

Canonical types for vectors, payloads, and search results. The goal is
**one struct per concept**; layer boundaries convert to/from proto or
HTTP shapes via explicit `From` impls, never via ad-hoc mapping code.

## SearchResult

Single source of truth: [`crate::models::SearchResult`](../../src/models/mod.rs).

```rust
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub dense_score: Option<f32>,
    pub sparse_score: Option<f32>,
    pub vector: Option<Vec<f32>>,
    pub payload: Option<Payload>,
}
```

- **Precision.** `score` is `f32` because HNSW scores are natively `f32`.
  Promoting to `f64` and back silently reorders near-tie results; we
  always stay in `f32`.
- **Hybrid scoring.** `dense_score` / `sparse_score` are populated by
  hybrid search; plain vector search leaves them `None`.

### Layer boundaries

| Boundary | Type | Conversion |
|----------|------|------------|
| gRPC `vectorizer.proto` | `vectorizer::SearchResult` (f32) | `impl From<&SearchResult> for vectorizer::SearchResult` (out), `impl From<vectorizer::SearchResult> for SearchResult` (in) |
| gRPC hybrid | `vectorizer::HybridSearchResult` (f32) | `impl From<vectorizer::HybridSearchResult> for SearchResult` — flattens `hybrid_score` into `score` and preserves dense/sparse fields |
| Cluster gRPC `cluster.proto` | `cluster::SearchResult` (already f32) | existing `From` impls in `src/grpc/` |
| REST | JSON via `serde` derive | transparent |

### What is NOT `SearchResult`

Several types *resemble* `SearchResult` but have a different role and
must not be confused with it:

| Type | Purpose |
|------|---------|
| `crate::search::ScoredDocument` | Full-document ranker output with `title`, `snippet`, `score_breakdown`, `highlighted terms`. Different shape from `SearchResult`; renamed in v3.0.0 to prevent accidental mixing. |
| `sdks/rust::SearchResult` | SDK-side mirror for client code. Converted from the canonical type at the client boundary. |
| `benchmark/.../SearchResults` | Test fixture container (`s` plural). |

## Adding a new field

1. Add the field to `crate::models::SearchResult`. Mark it
   `#[serde(skip_serializing_if = "Option::is_none")]` if optional so
   the wire format stays backward-compatible for clients that don't
   send it.
2. Update the `From<&SearchResult> for vectorizer::SearchResult` impl
   in `src/grpc/conversions.rs` to carry the new field.
3. Add the field to `proto/vectorizer.proto` (mirror the field number
   conventions of the surrounding messages; `float` for scores,
   `string` for ids).
4. Run `cargo build` — `build.rs` now emits explicit
   `cargo:rerun-if-changed` directives for each proto file, so the
   bindings regenerate without needing to `touch` anything manually.
5. Update the round-trip tests in `src/grpc/conversions.rs::tests` to
   exercise the new field.

## History

- **v3.0.0**: proto `SearchResult.score` narrowed from `double` →
  `float`; explicit `From` impls both ways; ranker `SearchResult`
  renamed to `ScoredDocument` to eliminate the name collision.
  Tracked under `phase2_unify-search-result-type`.
