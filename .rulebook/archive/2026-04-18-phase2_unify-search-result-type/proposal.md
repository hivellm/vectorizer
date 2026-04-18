# Proposal: phase2_unify-search-result-type

## Why

The codebase has **four distinct `SearchResult` struct definitions** with incompatible shapes and precision:

1. `src/models/mod.rs:593` — canonical, `score: f32`
2. `src/grpc/vectorizer.rs:234` — prost-generated, `score: f64`
3. `src/grpc/vectorizer.cluster.rs:179` — prost-generated cluster variant, `score: f64`
4. `src/search/advanced_search.rs:469` — ranker variant

Every layer boundary (REST ↔ core ↔ gRPC ↔ cluster ↔ ranker) has an implicit conversion. The `f32 ↔ f64` hop silently loses precision and can reorder near-tie results. Adding a new field to one site won't reach the others without manual replication.

This is a classic Cursor-generated symptom: each module grew its own "nearest-match" type instead of importing the shared one.

## What Changes

1. **Elect `src/models/mod.rs:593` as canonical** (`f32` matches HNSW's native precision).
2. **Ban other definitions** by lint or code review. The only allowed "variants" are:
   - `crate::models::SearchResult` (internal)
   - prost-generated types, converted to canonical with `impl From<proto::SearchResult> for models::SearchResult`
3. **Fix precision**: the proto schema should use `float` (f32), not `double` (f64). Update `proto/*.proto` files and regenerate.
4. Ensure `advanced_search.rs` returns `Vec<SearchResult>` (canonical) — or documents a narrower `ScoredCandidate` that isn't confused with `SearchResult`.

## Impact

- Affected specs: data model spec, gRPC API spec
- Affected code: `src/models/mod.rs`, `src/grpc/*.rs` (regenerated), `proto/*.proto`, `src/search/advanced_search.rs`, all callers
- Breaking change: **YES on gRPC wire** (double → float). Bump proto version; document migration.
- User benefit: eliminates silent precision loss, single source of truth, cleaner API.
