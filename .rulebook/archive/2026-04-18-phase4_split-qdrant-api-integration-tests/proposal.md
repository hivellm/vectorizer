# Proposal: phase4_split-qdrant-api-integration-tests

## Why

`tests/integration/qdrant_api.rs` is **1,595 lines** covering every Qdrant-compatible capability in a single file: points CRUD, collections CRUD, search, snapshots, filters, aliases. A failing test in one capability forces the reviewer to scan the whole file to understand scope; adding a new capability-level test touches one long shared setup block.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Split the file by capability under `tests/integration/qdrant_api/`:

- `points.rs`, `collections.rs`, `search.rs`, `snapshots.rs`, `filters.rs`, `aliases.rs`.
- `mod.rs` — shared test fixtures and `mod` declarations.

The shared `setup_test_server()` / `cleanup_test_collection()` helpers live in `mod.rs`; each capability file imports them.

## Impact

- Affected specs: none.
- Affected code: `tests/integration/qdrant_api.rs`, new subdirectory.
- Breaking change: NO (test-only).
- User benefit: capability-scoped failures report against focused files; adding new tests for one capability doesn't force-merge against another's edits.
