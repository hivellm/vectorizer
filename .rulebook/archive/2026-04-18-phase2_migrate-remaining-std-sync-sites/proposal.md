# Proposal: phase2_migrate-remaining-std-sync-sites

## Why

Sibling task `phase2_migrate-std-sync-to-parking-lot` already migrated the 4+1 files flagged by the audit as hot-path offenders: `src/batch/{mod,processor}.rs`, `src/db/vector_store.rs`, `src/cluster/raft_node.rs`, plus `src/db/hive_gpu_collection.rs` (structurally coupled to vector_store).

During that migration a wider grep revealed **13 additional files** still importing `Mutex` or `RwLock` from `std::sync`:

- `src/api/advanced_api.rs`
- `src/cache/advanced_cache.rs`
- `src/db/collection.rs`
- `src/db/quantized_collection.rs`
- `src/ml/advanced_ml.rs`
- `src/persistence/dynamic.rs`
- `src/persistence/wal.rs`
- `src/processing/advanced_pipeline.rs`
- `src/quantization/hnsw_integration.rs`
- `src/quantization/storage.rs`
- `src/search/advanced_search.rs`
- `src/security/enhanced_security.rs`
- `src/storage/advanced.rs`
- `src/storage/reader.rs`

Plus `benchmark/scripts/metal_native_search_benchmark.rs` (not hot-path, out of scope).

These 13 files were not in the original task because they're not in the persistence/batch/raft hot paths, but `AGENTS.md` bans `std::sync::{Mutex,RwLock}` in `src/` uniformly. Leaving them migrated-halfway invites new contributors to copy the old pattern.

## What Changes

For each of the 13 files:

1. `use std::sync::{Arc, Mutex}` → `use std::sync::Arc; use parking_lot::Mutex;` (same for `RwLock`).
2. Drop `.unwrap()` after `.lock()` / `.read()` / `.write()` calls (parking_lot guards don't poison).
3. `cargo check` after each file; `cargo clippy --all-targets -- -D warnings` at the end.

None of these files should need test changes — parking_lot is API-compatible with std::sync for the common case. Files that genuinely need `std::sync::Condvar`, `std::sync::Once`, `std::sync::OnceLock`, or `std::sync::mpsc` keep those imports (parking_lot doesn't replace them).

## Impact

- Affected specs: `/.rulebook/specs/RUST.md`
- Affected code: 13 files listed above
- Breaking change: NO (internal)
- User benefit: completes the `parking_lot`-everywhere rule; removes the drift risk pointed out during the phase2 migration; unblocks future CI grep-gate that forbids `std::sync::Mutex` / `std::sync::RwLock` in `src/`.
