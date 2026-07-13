## 1. Implementation
- [ ] 1.1 Add a metric-aware distance for OptimizedHnswIndex (runtime-dispatching Distance<f32> impl or enum over DistCosine/DistL2/DistDot), driven by config.distance_metric instead of hardcoded DistCosine
- [ ] 1.2 Make the distance→similarity/score conversion metric-aware (optimized_hnsw.rs:234-236) — cosine `1-d`, L2, and dot handled correctly
- [ ] 1.3 Ensure vector normalization in collection/data.rs is applied only for Cosine and stays consistent with the chosen index metric
- [ ] 1.4 Verify reindex_with_params (collection/index.rs) preserves the metric through the rebuild

## 2. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 2.1 Update or create documentation covering the implementation
- [ ] 2.2 Write tests: per-metric ranking correctness (Cosine/Euclidean/Dot) proving non-cosine collections rank by the requested metric, incl. a case where cosine vs euclidean give different top-k
- [ ] 2.3 Run tests and confirm they pass
