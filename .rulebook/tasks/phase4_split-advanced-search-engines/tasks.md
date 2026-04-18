## 1. Interface definition

- [ ] 1.1 Enumerate cross-engine method calls inside `src/search/advanced_search/mod.rs` (grep for `self.query_processor.`, `self.ranking_engine.`, `self.analytics.`, `self.suggestions.`).
- [ ] 1.2 Decide: inline `pub(super)` methods on each engine, or a shared trait per engine. Pick one consistent pattern.

## 2. Engine extraction

- [ ] 2.1 Extract `impl AdvancedSearchEngine` + `impl SearchIndex` to `engine.rs`.
- [ ] 2.2 Extract `impl QueryProcessor` to `query_processor.rs`.
- [ ] 2.3 Extract `impl RankingEngine` to `ranker.rs`.
- [ ] 2.4 Extract `impl SearchAnalytics` + `impl SearchSuggestions` to `analytics.rs`.

## 3. Verification

- [ ] 3.1 `cargo check --all-features` clean.
- [ ] 3.2 Existing search tests pass unchanged (1120/1120 lib, 780/780 integration).
- [ ] 3.3 Every file (including `mod.rs` and `types.rs`) under 600 lines.

## 4. Tail (mandatory)

- [ ] 4.1 Update the module-level doc comment describing the new layout.
- [ ] 4.2 Layout-only refactor; existing tests are sufficient.
- [ ] 4.3 Run `cargo test --all-features` and confirm pass.
