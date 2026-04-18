# Proposal: phase4_reenable-ignored-tests

## Why

29 tests in the repo are marked `#[ignore]`, meaning they exist on paper but **never run in CI**. Modules affected:

- `tests/replication/comprehensive.rs` — replication correctness
- `tests/replication/failover.rs` — failover behavior
- `tests/grpc/grpc_s2s.rs` — server-to-server gRPC
- `tests/core/wal_*.rs` — WAL behavior
- `tests/gpu/hive_gpu.rs` — GPU acceleration
- `tests/integration/cluster_performance.rs` — cluster perf
- `tests/integration/sparse_vector.rs` — sparse vectors
- `tests/integration/graph.rs` — graph ops
- `tests/integration/storage.rs` — storage layer

Common reason cited: "Requires TCP connection", "Run with: `cargo test --release -- --ignored`". This means the most risky subsystems (HA replication, GPU, gRPC s2s, cluster perf) are NOT actually validated by the project's `rust-tests` CI job.

A Rust project that markets HA/cluster/GPU as features must test them.

## What Changes

For each `#[ignore]` test:
1. Classify: **environment-dependent** (needs external service), **slow** (long-running), or **actually-broken** (genuinely failing).
2. For **environment-dependent**: start the required service in the test setup (testcontainers-rs for TCP endpoints; feature-gate GPU tests behind `hive-gpu` in CI matrix).
3. For **slow**: move to a nightly or `cargo test --release -- --ignored` CI job that runs on a scheduled cadence (not per-PR).
4. For **actually-broken**: either fix, or delete with a rulebook decision recording why.
5. Remove `#[ignore]` once the test is viable on the target CI matrix.

## Impact

- Affected specs: quality-enforcement spec
- Affected code: 9+ test files under `tests/`
- Breaking change: NO
- User benefit: confidence that HA, replication, GPU, and cluster features actually work; flakiness surfaced early.
