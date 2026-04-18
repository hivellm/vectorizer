# Proposal: phase3_split-vector-store-monolith

> **Part of the oversized-files audit.** See
> [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md)
> for the full inventory and severity rubric. This task covers the
> `critical`-severity `vector_store.rs` entry.

## Why

`src/db/vector_store.rs` is **3,948 lines** — the core database engine compressed into a single file. Problems:

- Mixes concerns: collection lifecycle, CRUD on vectors, index management, persistence orchestration, WAL, search dispatch, alias management, snapshot coordination.
- 15 `#[cfg(feature = "cluster")]` blocks (phantom feature — see `phase1_remove-phantom-cluster-feature`) pollute the file.
- 6 `.ok()` silencing errors on collection lookup (e.g., on alias resolution at line ~1680).
- `Arc<std::sync::Mutex<...>>` fields that should be `parking_lot` (see `phase2_migrate-std-sync-to-parking-lot`).
- Hard to test in isolation: touching one concern requires loading the whole 3.8k-line file context.

Splitting by concern makes the three dependent Phase 1/2 fixes mechanically simpler.

## What Changes

Decompose `src/db/vector_store.rs` into:

- `src/db/vector_store/mod.rs` — `VectorStore` struct, public API surface, state
- `src/db/vector_store/collections.rs` — create/delete/list/get collection
- `src/db/vector_store/vectors.rs` — insert/upsert/delete/get vector
- `src/db/vector_store/search.rs` — search dispatch, alias resolution for search
- `src/db/vector_store/aliases.rs` — alias management
- `src/db/vector_store/snapshots.rs` — snapshot/restore/backup
- `src/db/vector_store/persistence.rs` — save/load/WAL coordination
- `src/db/vector_store/hnsw_ops.rs` — low-level HNSW bridging

Each file ≤700 LOC. All external API preserved (`VectorStore::*` methods).

## Impact

- Affected specs: none
- Affected code: `src/db/vector_store.rs` (split), consumers unchanged
- Breaking change: NO
- User benefit: enables per-concern testing, simplifies parking_lot + phantom-feature + unwrap cleanups, reduces PR review friction.
