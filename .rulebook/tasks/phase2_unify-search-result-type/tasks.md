## 1. Audit

- [ ] 1.1 Grep all definitions of `struct SearchResult` and `struct ScoredResult` in `src/` and `proto/`; record sites in `design.md`
- [ ] 1.2 Map every call site that constructs or consumes each variant

## 2. Implementation

- [ ] 2.1 Make `src/models/mod.rs::SearchResult` the canonical struct; add documented field set
- [ ] 2.2 In `proto/vectorizer.proto` and `proto/vectorizer.cluster.proto`, change `double score` → `float score`; regenerate via `build.rs`
- [ ] 2.3 Add `impl From<proto::vectorizer::SearchResult> for models::SearchResult` (and `Into` the other way) in `src/grpc/conversions.rs`
- [ ] 2.4 Replace the `SearchResult` definition in `src/search/advanced_search.rs` with a re-export of the canonical type (or rename to `ScoredCandidate` if the shape legitimately differs)
- [ ] 2.5 Update every call site to use `crate::models::SearchResult`; delete the duplicate local defs

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/api/data-model.md` (or create) with the canonical type; add CHANGELOG breaking entry for the proto change
- [ ] 3.2 Write tests: round-trip conversion proto ↔ canonical preserves score within f32 eps; cluster path preserves ordering
- [ ] 3.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
