# Proposal: phase4_split-advanced-search

## Why

`src/search/advanced_search.rs` is **1,513 lines** — roughly 640 of which are configuration and type definitions at the top of the file, and the remainder is the ranker / filter / scorer engine. Splitting the config off makes reviewers of engine changes scroll 60% less and opens the door to reusing the config types from other search paths (discovery, hybrid).

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Create `src/search/advanced_search/`:

- `config.rs` — all config structs, weights, thresholds, scoring knobs that currently dominate the first ~640 lines.
- `ranker.rs` — `ScoredDocument` + the ranking implementation.
- `filter.rs` — the filter pass.
- `scorer.rs` — the scorer pass.
- `mod.rs` — re-exports + the top-level entry point.

## Impact

- Affected specs: none.
- Affected code: `src/search/advanced_search.rs`, new subdirectory.
- Breaking change: NO — `ScoredDocument` keeps the name it got in phase2; `pub use advanced_search::*;` in `search/mod.rs` keeps external callers working.
- User benefit: engine changes and config changes are no longer tangled in the same 1,500-line file; the config types become reusable.
