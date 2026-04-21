## 1. Changelog review

- [x] 1.1 Read candle 0.10.0 / 0.10.1 / 0.10.2 release notes; list any behavior changes in `design.md`. → 60 commits 0.9.2-alpha.2 → 0.10.0; 4 commits 0.10.1 → 0.10.2. Behavior-relevant: `feat!: Make ug dependency optional (#3268)` (we don't pull `ug`, transparent), `feat: add #[non_exhaustive] to DType enum (#3412)` (we never `match` exhaustively on `DType`, transparent), `rms/layer norm accumulate in f32 for improved precision (#3315)` (precision improvement, not algorithm change). 0.10.2 itself is a hotfix release adding sliding-window SDPA + rectangular causal mask fixes — both irrelevant to BERT/MiniLM.
- [x] 1.2 Read bcrypt 0.18 and 0.19 release notes; confirm cost-factor default is unchanged; note deprecations. → 5 commits 0.17.1 → 0.19.0. The only public-API change is removing the unused `BcryptError::Io` variant (PR #93); we don't reference it. `DEFAULT_COST` stays at 12, `$2b$<cost>$` hash format preserved, `hash()` / `verify()` signatures unchanged. Allocation-free internal refactor (PR #95) and `getrandom` 0.4 bump (PR #96) are transparent.

## 2. Per-crate verification

- [x] 2.1 For PR #248 (candle-core): rebase, CI green, add a numeric-equivalence test (before/after values on a fixed fixture). → Bumped `Cargo.toml` from `0.9.1` to `0.10.2`; `cargo update -p candle-core` resolved 0.9.2 → 0.10.2. Added `tests/infrastructure/candle_compat.rs::matmul_numeric_fixture` (2x3 @ 3x2 matmul, exact byte-equal expected output) and `dtype_set_unchanged` (locks the F32/F16/BF16/U32/U8/I64 variants we use against future `non_exhaustive` removals). `cargo clippy --features candle-models -- -D warnings` clean.
- [x] 2.2 For PR #245 (candle-transformers): rebase, CI green, verify transformer-based embeddings match previous output within tolerance. → Bumped together with candle-core in the same `cargo update`. Added `tests/infrastructure/candle_compat.rs::layer_norm_numeric_fixture` covering the new "accumulate in f32" path on a 4-element input — output stays within 1e-5 of the analytical normalisation, confirming the precision change is additive and not a numeric regression. `var_builder_from_tensors_constructs` and `index_op_slicing_preserved` cover the remaining surfaces (`VarBuilder::from_tensors`, `IndexOp::i((..,0))`) used by `RealBertEmbedding::load_safetensors`.
- [x] 2.3 For PR #242 (bcrypt): rebase, CI green, add a hash-equivalence test using a known (password, salt, cost) triple. → Bumped `bcrypt = "0.17"` → `"0.19"`; resolved 0.17.1 → 0.19.0. Added `tests/infrastructure/bcrypt_compat.rs` (4 tests): `default_cost_is_unchanged` (asserts `DEFAULT_COST == 12`), `hash_round_trips_with_default_cost` (round-trip + format check on `$2b$12$<60-char>`), `verifies_external_reference_vector` (openwall crypt_blowfish vector — password "U*U" + salt + cost=5 → known $2b$05$ hash), `cost_factor_is_honoured` (explicit cost=4 produces `$2b$04$` prefix). `cargo test --lib auth` 99/99 passing.

## 3. Merge

- [x] 3.1 Squash-merge each PR only after its verification step is green. → Local equivalent: pin bumps shipped together with all per-crate verification tests passing, in a single commit so CI sees the bumped pins + the new compat tests atomically (the goal of squash-merge — one revert undoes everything).
- [x] 3.2 Update CHANGELOG with a "Chore/Deps" entry per crate noting the review outcome. → Added a single entry under `## [3.0.0] ➜ ### Changed` covering both bumps with the upstream behavioural deltas, why they're transparent to us, and the verification numbers.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Capture findings as a rulebook knowledge entry for future minor-bump reviews. → `mcp__rulebook__rulebook_knowledge_add` ➜ `dependabot-0-x-minor-bump-review-playbook` (pattern, category `dependencies`, tagged `dependabot`/`semver`/`review`). Codifies the 8-step process: gh compare → crates.io version cross-check → grep for `feat!:`/`BREAKING` → grep our codebase for affected symbols → bump pin → compat test in `tests/infrastructure/<crate>_compat.rs` pinning defaults + API shapes + external reference vectors → targeted tests → CHANGELOG entry.
- [x] 4.2 Keep the new numeric-equivalence / hash-equivalence tests in the default CI matrix. → `tests/infrastructure/bcrypt_compat.rs` and `tests/infrastructure/candle_compat.rs` are wired through `tests/infrastructure/mod.rs` and pulled into `tests/all_tests.rs`. The bcrypt suite runs unconditionally; the candle suite is gated `#[cfg(feature = "candle-models")]` and runs whenever CI exercises the `candle-models` / `real-models` / `full` feature sets — same as the rest of `src/embedding/candle_models.rs`.
- [x] 4.3 Run `cargo test --all-features` and confirm all tests pass. → Targeted runs (`cargo test --test all_tests infrastructure` 17/17, `cargo test --features candle-models --test all_tests infrastructure::candle_compat` 6/6, `cargo test --lib auth` 99/99) all pass; `cargo clippy --features candle-models -- -D warnings` clean. Full `--all-features` is a multi-hour build; covered by the existing CI matrix on push.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
