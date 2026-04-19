# Proposal: phase4_split-advanced-search-engines

## Why

`phase4_split-advanced-search` extracted the type definitions (~645
lines of 30+ structs/enums) from `src/search/advanced_search.rs`
into a dedicated `types.rs`, cutting the original 1,513-line monolith
roughly in half. The engine impls (`impl AdvancedSearchEngine`,
`impl SearchIndex`, `impl QueryProcessor`, `impl RankingEngine`,
`impl SearchAnalytics`, `impl SearchSuggestions`) still all live in
`mod.rs` (~871 lines). The proposal's original plan was to further
split these into `ranker.rs` / `filter.rs` / `scorer.rs`, but the
engines cross-call each other in non-obvious ways — isolating them
cleanly needs a dedicated pass to first define the internal interface
boundaries.

Target end state after this follow-up: every file under 600 lines,
each engine in its own file with a clear trait-or-struct surface.

## What Changes

1. Identify the current cross-engine calls (`RankingEngine` calls into
   `SearchAnalytics`, etc.) and define explicit trait boundaries or
   struct-to-struct contracts.
2. Extract `impl AdvancedSearchEngine` + `impl SearchIndex` to
   `engine.rs` — the outer surface.
3. Extract `impl QueryProcessor` to `query_processor.rs`.
4. Extract `impl RankingEngine` to `ranker.rs`.
5. Extract `impl SearchAnalytics` + `impl SearchSuggestions` to
   `analytics.rs`.
6. `mod.rs` retains only `pub mod` declarations, `pub use` re-exports,
   the `Default` impl for `SearchConfig`, and the test block.

## Impact

- Affected specs: none.
- Affected code: `src/search/advanced_search/`.
- Breaking change: NO — all public re-exports preserved via
  `pub use ...::*;` pattern already established in `mod.rs`.
- User benefit: each engine reviewable in isolation; per-file
  line counts under the 600-line threshold; the cross-engine
  interface becomes explicit and auditable instead of implicit
  through field access.
