# Proposal: phase3_split-collection-monolith

## Why

`src/db/collection.rs` is **2,636 lines** — one of the three largest non-generated files in the tree. `impl Collection` tangles vector storage, HNSW index management, persistence, graph relations, and a 500+ line `#[cfg(test)]` block in a single file. Every touch on a single concern forces reviewers to scroll past the other four; IDE navigation is slow; and cross-cutting refactors (rust-lint parking_lot migration, layering fix, `Secret<T>`) have hit this file repeatedly.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md) for the full audit context.

## What Changes

1. Extract the five `impl Collection` concerns into sibling files under a new `src/db/collection/` module:
   - `data.rs` — insert/update/delete/get of vectors + payload.
   - `index.rs` — HNSW construction, rebuild, ef tuning.
   - `persistence.rs` — save/load/serialize to the `.vecdb` format.
   - `graph.rs` — edge/node helpers that currently sit on `Collection`.
   - `quantization.rs` — the quantized-storage pathway.
2. Extract the `#[cfg(test)]` block into `collection/tests.rs`.
3. Keep `src/db/collection.rs` as a thin `mod.rs` that defines the struct + re-exports the public surface, so external callers (dozens of them) don't need to touch imports.
4. Each sub-file stays under ~600 lines.

## Impact

- Affected specs: none (layout-only refactor).
- Affected code: `src/db/collection.rs` (becomes `mod.rs`), new `src/db/collection/*.rs`.
- Breaking change: NO — public API preserved via re-export.
- User benefit: 5×smaller review surface per change; concerns reviewable in isolation; future work on any single concern stops pulling in the other four.
