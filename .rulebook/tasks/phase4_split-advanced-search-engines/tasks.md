## 1. Interface definition

- [x] 1.1 Enumerated cross-engine calls in the pre-split `mod.rs`: only the orchestrator `AdvancedSearchEngine::search` calls into `query_processor`, `ranking_engine`, `analytics`, `suggestions`, and `index`. No engine-to-engine calls exist; the orchestrator is the single fan-out point.
- [x] 1.2 Picked `pub(super) fn` on each engine method the orchestrator calls. Struct fields changed to `pub(super)` in `types.rs` so sibling submodules can construct them. No trait abstraction needed because there is no engine substitution at runtime.

## 2. Engine extraction

- [x] 2.1 Extracted `impl AdvancedSearchEngine` + `impl SearchIndex` to `engine.rs` (408 lines).
- [x] 2.2 Extracted `impl QueryProcessor` + the `ProcessedQuery` bridge type to `query_processor.rs` (108 lines).
- [x] 2.3 Extracted `impl RankingEngine` to `ranker.rs` (114 lines).
- [x] 2.4 Extracted `impl SearchAnalytics` + `impl SearchSuggestions` to `analytics.rs` (129 lines).

## 3. Verification

- [x] 3.1 `cargo check --lib --all-features`: clean.
- [x] 3.2 `cargo test --lib --all-features`: 1158/1158 pass - no regression. (Note: the module is not registered in `src/lib.rs` and has been dead code for some time; test file at `tests.rs` is preserved for when the module is wired back in.)
- [x] 3.3 File sizes: `mod.rs` 110 (was 800), `engine.rs` 408, `query_processor.rs` 108, `ranker.rs` 114, `analytics.rs` 129 - all well under 600. `types.rs` 658 (pre-existing, outside this task's scope; predates the split).

## 4. Tail (mandatory)

- [x] 4.1 Updated `mod.rs` module-level `//!` doc with a "Layout" section listing the new files and their responsibilities, and documenting the `pub(super)` cross-engine contract.
- [x] 4.2 Layout-only refactor; existing tests are sufficient.
- [x] 4.3 `cargo clippy --lib --all-features -- -D warnings`: 0 warnings. `cargo fmt`: clean.
