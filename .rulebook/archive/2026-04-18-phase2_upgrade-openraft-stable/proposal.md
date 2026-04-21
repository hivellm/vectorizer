# Proposal: phase2_upgrade-openraft-stable

## Why

`Cargo.toml` pins `openraft = "0.10.0-alpha.17"` and `openraft-memstore = "0.10.0-alpha.17"`. These are pre-release versions providing the HA/consensus backbone of the cluster module. Risks:

- No stability guarantees: the API and semantics may break in any alpha bump.
- No security patch commitment from upstream.
- Bug reports against alpha versions typically don't receive backports.
- Difficult to audit: alpha changelogs are informal.

Vectorizer 2.5.16 markets HA/Raft as a feature. Shipping with an alpha library undermines that claim for operators who need reliable consensus.

Additionally, `ort = "2.0.0-rc.10"` (ONNX) is on release candidate. Lower severity since it's optional under `fastembed` feature, but deserves the same review.

## What Changes

1. Check upstream for the latest stable release of `openraft` / `openraft-memstore`. If 0.10.0 stable exists, migrate.
2. If only alpha remains, document the risk in `README.md` (clearly: "HA/cluster mode depends on openraft 0.10.x alpha; not recommended for mission-critical prod until upstream ships stable").
3. Pin exact versions (`openraft = "=0.10.0-alpha.17"`) until upgraded, so `cargo update` can't drift.
4. Evaluate `ort`: upgrade to the newest RC/stable, or plan the migration.
5. Add `cargo-deny` config in `deny.toml` flagging pre-release versions in the dependency graph.

## Impact

- Affected specs: dependency management, HA/cluster spec
- Affected code: `Cargo.toml`, `Cargo.lock`, any code that touches openraft API surface (`src/cluster/raft_node.rs`, `src/db/raft.rs`)
- Breaking change: possibly at the Raft log/state-machine level — document migration of on-disk state.
- User benefit: reduced supply-chain risk; credible HA claim in marketing/docs; easier CVE response.
