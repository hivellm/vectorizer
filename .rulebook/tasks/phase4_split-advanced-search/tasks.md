## 1. Layout

- [ ] 1.1 Create `src/search/advanced_search/` and move the existing file to `mod.rs`.
- [ ] 1.2 Extract config/structs (~640 lines) to `config.rs`.
- [ ] 1.3 Extract the ranker (`ScoredDocument` + ranker impl) to `ranker.rs`.
- [ ] 1.4 Extract filter pass to `filter.rs`.
- [ ] 1.5 Extract scorer pass to `scorer.rs`.

## 2. Verification

- [ ] 2.1 `cargo check --all-features` clean.
- [ ] 2.2 Existing search tests pass unchanged.
- [ ] 2.3 No sub-file exceeds 600 lines.

## 3. Tail (mandatory)

- [ ] 3.1 Update the module-level doc comment.
- [ ] 3.2 Existing tests sufficient; no new tests required.
- [ ] 3.3 `cargo test --all-features` pass.
