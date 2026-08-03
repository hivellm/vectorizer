# Proposal: phase1_bump-openraft-alpha30

## Why
Dependabot #382 bumps openraft 0.10.0-alpha.22 -> 0.10.0-alpha.30. Both
`openraft` and `openraft-memstore` are pinned with `=` in the Cargo.tomls on
purpose (comment in crates/vectorizer/Cargo.toml): the consensus layer must not
drift silently between alphas, and any bump requires re-validating the HA Raft
path. This was split out of phase1_deps-refresh-3.6.0 because it is not a blind
lockfile bump — it needs a live-cluster HA test.

## What Changes
- Bump the `=0.10.0-alpha.22` pins to `=0.10.0-alpha.30` for both `openraft`
  and `openraft-memstore` in every crate that pins them (vectorizer,
  vectorizer-server).
- Review the alpha.22 -> alpha.30 changelog for API/behavior changes.
- Run and pass the HA path: tests/integration/cluster_ha.rs (needs a live
  multi-node cluster harness).

## Impact
- Affected specs: replication/consensus
- Affected code: crates/vectorizer/Cargo.toml, crates/vectorizer-server/Cargo.toml, Cargo.lock, HA cluster code if the API changed
- Breaking change: NO (internal dependency), but consensus-critical
- User benefit: current openraft alpha with upstream fixes, HA-verified
