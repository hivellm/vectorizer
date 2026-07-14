## 1. Implementation
- [x] 1.1 Add a metric-aware distance for OptimizedHnswIndex (runtime-dispatching MetricDistance impl over DistCosine/DistL2/sigmoid(-dot)), driven by config.distance_metric instead of hardcoded DistCosine
- [x] 1.2 Make the distance→similarity/score conversion metric-aware (distance_to_similarity) — cosine `1-d`, L2 `1/(1+d)`, dot `1-sigmoid(-dot)`
- [x] 1.3 Vector normalization in collection/data.rs already applies only for Cosine (data.rs:90/288/446/521) — consistent with the chosen index metric; no change needed
- [x] 1.4 reindex_with_params already passes self.config.metric (index.rs:150); now effective since new() honors it

## 2. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 2.1 Update or create documentation covering the implementation — CHANGELOG [Unreleased] Fixed entry + MetricDistance/distance_to_similarity doc comments
- [x] 2.2 Write tests covering the new behavior — euclidean_ranks_by_l2_not_cosine + dot_product_ranks_by_inner_product_not_cosine (discriminating cases) + cosine-unchanged assertion
- [x] 2.3 Run tests and confirm they pass — 4/4 optimized_hnsw + 36/36 db::collection, clippy clean
