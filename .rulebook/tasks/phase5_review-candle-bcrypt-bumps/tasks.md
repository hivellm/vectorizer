## 1. Changelog review

- [ ] 1.1 Read candle 0.10.0 / 0.10.1 / 0.10.2 release notes; list any behavior changes in `design.md`
- [ ] 1.2 Read bcrypt 0.18 and 0.19 release notes; confirm cost-factor default is unchanged; note deprecations

## 2. Per-crate verification

- [ ] 2.1 For PR #248 (candle-core): rebase, CI green, add a numeric-equivalence test (before/after values on a fixed fixture)
- [ ] 2.2 For PR #245 (candle-transformers): rebase, CI green, verify transformer-based embeddings match previous output within tolerance
- [ ] 2.3 For PR #242 (bcrypt): rebase, CI green, add a hash-equivalence test using a known (password, salt, cost) triple

## 3. Merge

- [ ] 3.1 Squash-merge each PR only after its verification step is green
- [ ] 3.2 Update CHANGELOG with a "Chore/Deps" entry per crate noting the review outcome

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Capture findings as a rulebook knowledge entry for future minor-bump reviews
- [ ] 4.2 Keep the new numeric-equivalence / hash-equivalence tests in the default CI matrix
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
