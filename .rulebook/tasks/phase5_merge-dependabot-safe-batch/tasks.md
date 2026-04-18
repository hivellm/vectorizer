## 1. Prerequisite

- [ ] 1.1 Confirm `phase1_fix-lint-cluster-ha` is merged to `main`

## 2. Rebase and merge

- [ ] 2.1 Comment `@dependabot rebase` on PR #250 (fastrand); wait for green CI; squash-merge
- [ ] 2.2 Same for PR #249 (blake3)
- [ ] 2.3 Same for PR #247 (cc)
- [ ] 2.4 Same for PR #246 (libc)
- [ ] 2.5 Same for PR #244 (arc-swap)
- [ ] 2.6 Same for PR #243 (tokio)

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Verify CHANGELOG auto-updates or add a collective "Chore: bump deps" entry
- [ ] 3.2 Pull `main` locally after all merges; run `cargo build --release` and `cargo test --all-features` to confirm green
- [ ] 3.3 Run `cargo audit` to confirm no new advisories

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
