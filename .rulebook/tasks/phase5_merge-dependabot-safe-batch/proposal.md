# Proposal: phase5_merge-dependabot-safe-batch

## Why

Six Dependabot PRs target patch/minor version bumps where the rust-tests CI job passed (only the pre-existing `lint` failure in `cluster_ha.rs` caused red status — addressed by `phase1_fix-lint-cluster-ha`):

- #250 `fastrand` 2.3.0 → 2.4.0 (patch)
- #249 `blake3` 1.8.3 → 1.8.4 (patch)
- #247 `cc` 1.2.57 → 1.2.59 (patch)
- #246 `libc` 0.2.183 → 0.2.184 (patch)
- #244 `arc-swap` 1.9.0 → 1.9.1 (patch)
- #243 `tokio` 1.50.0 → 1.51.0 (minor)

These are low-risk bumps. The longer they sit, the more rebase churn Dependabot will produce and the more likely a future PR merges behind them.

## What Changes

Prerequisite: `phase1_fix-lint-cluster-ha` is merged so the `lint` job is green.

Then, for each PR above:

1. Trigger a rebase on Dependabot (`@dependabot rebase`) to pick up the lint fix.
2. Confirm CI goes green (all jobs).
3. Merge using the project's standard squash-merge.

Alternatively, group them into a single "deps: bump 6 crates" PR manually if the team prefers one merge over six.

## Impact

- Affected specs: none
- Affected code: `Cargo.toml`, `Cargo.lock`
- Breaking change: NO
- User benefit: current dependency baseline; tokio 1.51.0 includes perf/fix updates; fewer stale branches.
