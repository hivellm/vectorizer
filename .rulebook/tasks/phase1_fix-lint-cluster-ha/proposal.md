# Proposal: phase1_fix-lint-cluster-ha

## Why

The `lint` CI job is failing on `main` with 3 clippy errors in `tests/integration/cluster_ha.rs`. Every open Dependabot PR (#241-#250, 10 PRs) inherits the red status, blocking the team from merging safe dependency updates. The failures are:

1. `tests/integration/cluster_ha.rs:14` — unused import `ClusterNode`.
2. `tests/integration/cluster_ha.rs:318` — `clippy::uninlined_format_args` (use of `{}` with a separate arg instead of `{id}`).
3. `tests/integration/cluster_ha.rs:734` — same `clippy::uninlined_format_args` on `elapsed`.

These errors were introduced by commit `b86faa74` ("test(cluster): add 15 integration tests for HA cluster functionality") and have been propagating since.

`cargo clippy -- -D warnings` is the project's quality gate; it must stay green on `main`.

## What Changes

- Remove the unused `ClusterNode` import at line 14.
- Inline the `format!` args in both `assert!`/`assert_eq!` macros at lines 318 and 734.
- Re-run `cargo clippy --all-targets -- -D warnings` locally to confirm zero warnings.

## Impact

- Affected specs: none
- Affected code: `tests/integration/cluster_ha.rs`
- Breaking change: NO
- User benefit: unblocks 10 Dependabot PRs and restores `lint` CI signal for every future PR.
