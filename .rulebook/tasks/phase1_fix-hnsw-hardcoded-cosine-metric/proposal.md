# Proposal: phase1_fix-hnsw-hardcoded-cosine-metric

## Why
`OptimizedHnswIndex` hardcodes the HNSW distance function to `DistCosine`
regardless of the collection's configured `DistanceMetric`. A collection
created with `metric: Euclidean` or `metric: Dot` is still ranked by cosine
similarity, silently returning wrong ordering. The bug is masked for the
default `Cosine` collections because vectors are L2-normalized only in that
case, so cosine happens to be correct there.

Evidence:
- `crates/vectorizer/src/db/optimized_hnsw.rs:65` — field type is
  `Hnsw<'static, f32, DistCosine>` (distance fixed at the type level).
- `optimized_hnsw.rs:87-92` — always constructs `Hnsw::<f32, DistCosine>::new(..., DistCosine {})`.
- `optimized_hnsw.rs:38` — `OptimizedHnswConfig.distance_metric` exists but is never read.
- `crates/vectorizer/src/db/collection/index.rs:150` — the correct metric IS
  passed into the config, then dropped by the index.
- `optimized_hnsw.rs:234-236` — distance→similarity conversion assumes cosine
  (`similarity = 1.0 - distance`), also wrong for other metrics.
- `crates/vectorizer/src/db/collection/data.rs:90` — vectors are normalized
  only when `metric == Cosine`.

This was surfaced by a VecLite parity investigation: parity between the two
engines only holds on the cosine axis. The parity test has real discriminating
power — a wrong metric scored 0.566 and poorly-clustered data 0.97, while only
well-structured cosine data exceeds 0.99 — so the cosine-only agreement was
nearly mistaken for full correctness.

## What Changes
Make the HNSW index honor the collection's configured metric:
- Select the distance function from `config.distance_metric` (Cosine →
  cosine, Euclidean → L2, Dot → dot/inner-product) instead of hardcoding
  `DistCosine`. Options: a runtime-dispatching custom `Distance<f32>` impl
  (single generic type, matches on the metric) or an enum over
  `Hnsw<f32, DistCosine|DistL2|DistDot>`; the custom-`Distance` approach keeps
  the struct field one type and is preferred.
- Make the distance→similarity/score conversion metric-aware (cosine:
  `1 - d`; L2: score from distance without the cosine assumption; dot:
  handle "larger = closer" correctly).
- Ensure vector normalization in `collection/data.rs` stays consistent with
  the chosen metric (only Cosine should be normalized).
- Verify `reindex_with_params` preserves the metric.

## Impact
- Affected specs: db/hnsw search correctness
- Affected code: `crates/vectorizer/src/db/optimized_hnsw.rs` (primary),
  `crates/vectorizer/src/db/collection/{index.rs,data.rs}` (score/normalize
  consistency)
- Breaking change: NO API change; fixes incorrect ranking for Euclidean/Dot
  collections. Existing Cosine collections are unaffected.
- User benefit: Euclidean and Dot collections return correctly-ranked results;
  the metric field stops being silently ignored.
