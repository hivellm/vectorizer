## 1. Investigation

- [ ] 1.1 Check `cargo search openraft` and openraft GitHub releases for latest stable/RC
- [ ] 1.2 Review openraft's CHANGELOG since 0.10.0-alpha.17 for breaking changes
- [ ] 1.3 Check `ort` crate for current stable/RC; record findings in `design.md`

## 2. Implementation

- [ ] 2.1 Upgrade `openraft` / `openraft-memstore` to the chosen target version in `Cargo.toml`
- [ ] 2.2 Resolve API breakage in `src/cluster/raft_node.rs`, `src/db/raft.rs`, and any consumer
- [ ] 2.3 If upstream offers no stable, pin exact alpha with `"="` prefix and document in README
- [ ] 2.4 Upgrade `ort` to latest release (optional feature `fastembed`)
- [ ] 2.5 Add `deny.toml` with `[advisories] version-req = "!pre"` (or equivalent) to flag future pre-release adds

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Document the decision (upgrade vs pin) in CHANGELOG and `README.md`'s HA section; add a deployment risk note
- [ ] 3.2 Run existing Raft/HA integration tests (`tests/integration/cluster_ha.rs`, replication tests); ensure they all pass
- [ ] 3.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
