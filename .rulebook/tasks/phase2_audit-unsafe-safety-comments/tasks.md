## 1. Audit

- [ ] 1.1 Enumerate all unsafe blocks in `src/` via `grep -rn 'unsafe {' src/` — list in `design.md` with file:line and current presence/absence of SAFETY comment
- [ ] 1.2 For each site, classify: (a) add comment, (b) replace with safe wrapper, (c) promote to `unsafe fn`

## 2. Implementation

- [ ] 2.1 Fix `src/embedding/cache.rs:334` — add SAFETY for mmap invariant (file locked, not truncated during lifetime)
- [ ] 2.2 Fix `src/embedding/candle_models.rs:135,296` — document safetensors invariant or replace with safe variant
- [ ] 2.3 Fix `src/bin/vectorizer-cli.rs:240` — daemon/setsid SAFETY
- [ ] 2.4 Fix `src/storage/mmap.rs:101` — MmapOptions SAFETY for file-backed mapping
- [ ] 2.5 Fix remaining 15 sites discovered in 1.1

## 3. Enforcement

- [ ] 3.1 Add `undocumented_unsafe_blocks = "deny"` to `clippy.toml` (or `[lints.clippy]` in `Cargo.toml`)
- [ ] 3.2 Run `cargo clippy --all-features --all-targets -- -D warnings` and confirm zero hits

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `docs/development/unsafe-policy.md` explaining the rule and examples
- [ ] 4.2 Add a unit/doc test for any newly-introduced safe wrapper
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
