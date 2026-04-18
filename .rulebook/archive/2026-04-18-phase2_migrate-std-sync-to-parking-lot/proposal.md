# Proposal: phase2_migrate-std-sync-to-parking-lot

## Why

`AGENTS.md` explicitly mandates `parking_lot` locks over `std::sync` ("`parking_lot` locks (not std::sync)"). The audit found **28 violations** of this rule in hot paths:

- `src/batch/mod.rs` — 7 `Arc<std::sync::Mutex<EmbeddingManager>>` across batch/processor.rs
- `src/db/vector_store.rs` — `pending_saves`, `save_task_handle`, `wal` fields as `Arc<std::sync::Mutex<...>>` (hit on every save operation)
- `src/cluster/raft_node.rs` — `snapshot_idx: std::sync::Mutex<u64>` (hit on every Raft snapshot)
- Plus 18 other sites across `src/cache/`, `src/persistence/`, `src/monitoring/`

The project already imports `parking_lot` (67 correct uses) and the API is almost identical — the only reason these remain is missed in initial Cursor generation and never cleaned up. `parking_lot::Mutex` is roughly 2× faster, smaller (1 byte vs 8+), poisoning-free, and supports `.lock()` without unwrap.

## What Changes

1. Global replace `use std::sync::{Mutex, RwLock}` → `use parking_lot::{Mutex, RwLock}` in `src/` (NOT in tests — `std::sync::Once` / `OnceLock` stay in std).
2. Drop `.unwrap()` after `.lock()` / `.read()` / `.write()` since parking_lot guards don't return `PoisonResult`.
3. Where code inspects poison state (if any), migrate to an explicit `Arc<AtomicBool>` flag or accept that parking_lot doesn't poison.
4. Add a clippy deny lint (`clippy::std_sync_once_no_std`) or a custom grep CI gate that rejects `std::sync::Mutex` / `std::sync::RwLock` in `src/`.

Exclusions: `std::sync::Arc`, `std::sync::atomic::*`, `std::sync::Once`, `std::sync::OnceLock` — these have no `parking_lot` equivalent and are allowed.

## Impact

- Affected specs: Rust conventions spec
- Affected code: ~28 files in `src/batch/`, `src/db/`, `src/cluster/`, `src/cache/`, `src/persistence/`, `src/monitoring/`
- Breaking change: NO (internal)
- User benefit: measurable throughput win on lock-heavy paths (batch embedding, WAL, Raft snapshots); removes deadlock risk from poisoning semantics.
