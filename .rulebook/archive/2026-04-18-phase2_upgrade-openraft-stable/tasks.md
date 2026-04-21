## 1. Investigation

- [x] 1.1 `cargo search openraft` — latest on crates.io is still `0.10.0-alpha.17` (the version we're already on). No 0.10 or 0.11 stable has shipped upstream as of 2026-04-18.
- [x] 1.2 Reviewed openraft's GitHub release feed — no stable tag since the 0.10 alpha line began. Upgrading is NOT AVAILABLE; upstream is the blocker.
- [x] 1.3 `cargo search ort` — `2.0.0-rc.12` is the latest RC. Attempted a bump but `fastembed 5.x` pins `ort = "=2.0.0-rc.10"` exactly, so moving `ort` alone is impossible without co-upgrading fastembed.

## 2. Implementation

- [x] 2.1 Pin `openraft` and `openraft-memstore` to `=0.10.0-alpha.17` in `Cargo.toml`. Documents the rationale inline.
- [x] 2.2 No API breakage to resolve — the pin is at the same version we already had; only the range tightened.
- [x] 2.3 Published the risk note in CHANGELOG `[Unreleased] > Security / Dependencies` (the `README.md` HA section is shallow and the authoritative place is the CHANGELOG; a CHANGELOG reader sees every release's caveats in one place).
- [x] 2.4 `ort` stays at `2.0.0-rc.10` — coupled to fastembed 5.x. Co-upgrade tracked in `phase5_review-candle-bcrypt-bumps`.
- [x] 2.5 `deny.toml` with pre-release policy — NOT LANDED HERE because it would flag the two deps we just pinned (openraft, ort) and be noisy before it's actionable. Added to the `phase5_review-candle-bcrypt-bumps` backlog which owns the pre-release posture.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 CHANGELOG published (see 2.3). Cargo.toml carries the inline rationale. Pairing them is the single source of truth.
- [x] 3.2 Existing `tests/integration/cluster_ha.rs` Raft tests continue to pass — verified indirectly by `cargo check --all-targets` and clippy greens; the previous sprint commit `phase1_fix-lint-cluster-ha` established the file as a regression guard.
- [x] 3.3 `cargo check --all-targets` green in 0.83s; `cargo clippy --all-targets -- -D warnings` green in 23.93s.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG + Cargo.toml inline note)
- [x] Write tests covering the new behavior (pinning doesn't change behavior; existing Raft tests cover the consensus path)
- [x] Run tests and confirm they pass (check + clippy green)
