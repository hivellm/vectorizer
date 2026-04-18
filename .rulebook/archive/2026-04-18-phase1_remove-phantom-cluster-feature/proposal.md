# Proposal: phase1_remove-phantom-cluster-feature

## Why

15 occurrences of `#[cfg(feature = "cluster")]` exist in the codebase — notably in `src/db/vector_store.rs` and `src/db/distributed_sharded_collection.rs` — yet **the feature `"cluster"` is NOT defined in `Cargo.toml`**. Consequences:

- The gated code is permanently dead (never compiled). Bug fixes never reach it; type drift goes undetected.
- If someone ever adds `cluster = []` to `Cargo.toml` without reviewing the gated code, the build will explode on missing `crate::cluster::*` types (e.g., `ClusterManager`, `DistributedShardRouter`) that may have moved/renamed.
- It misleads readers: code looks like an opt-in feature but is structurally impossible to enable.

This is a ticking time-bomb introduced by Cursor-style code generation that adds feature flags speculatively. Decide now: either make the feature real (compile + test path) or delete the gated code.

Separately: the `src/cluster/` module IS compiled (no feature gate) and has passing tests (`test_raft_single_node_bootstrap_and_propose`, etc.), so the capability exists — only the `#[cfg(feature = "cluster")]` wrapping is wrong.

## What Changes

Option A (preferred): **Make the feature real.**
1. Add `cluster = ["dep:openraft", "dep:openraft-memstore"]` to `Cargo.toml` (`openraft` is currently unconditional; move to optional).
2. Ensure all gated code compiles under `--features cluster` AND under default features with the gate flipped.
3. Add a CI matrix row that builds with `--no-default-features --features cluster` and another with `--features full`.

Option B (fallback if Option A requires too much refactoring): **Delete the gate.**
1. Remove every `#[cfg(feature = "cluster")]` — the code becomes unconditionally compiled (the `src/cluster/` module already is).
2. Remove the `#[cfg(not(feature = "cluster"))]` fallback branches if any.

Decide between A and B in the task's design.md after a small spike.

## Impact

- Affected specs: `/.rulebook/specs/RUST.md` (feature-flag hygiene), cluster module spec
- Affected code: `Cargo.toml`, `src/db/vector_store.rs` (15 sites), `src/db/distributed_sharded_collection.rs`, CI workflow
- Breaking change: NO (Option B) / NO behavior change (Option A adds a new opt-in path)
- User benefit: eliminates phantom-feature maintenance hazard; code either compiles or is deleted — no in-between limbo.
