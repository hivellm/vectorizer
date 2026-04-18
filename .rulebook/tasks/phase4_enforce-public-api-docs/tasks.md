## 1. Baseline

- [ ] 1.1 Run `cargo doc --no-deps --all-features` and capture the current warning count in `design.md`
- [ ] 1.2 Add `#![warn(missing_docs)]` to `src/lib.rs` behind a feature flag or temporarily to produce the full violation list

## 2. Documentation pass (per module, sequential)

- [ ] 2.1 Document `src/db/` public items (30+ re-exports + structs)
- [ ] 2.2 Document `src/api/` module-level `//!` headers + public handlers/types
- [ ] 2.3 Document `src/embedding/` concrete provider types
- [ ] 2.4 Document `src/server/` module headers + public types
- [ ] 2.5 Document `src/grpc/` public conversion types and service traits
- [ ] 2.6 Document `src/cluster/`, `src/replication/`, `src/auth/`, `src/cache/`, `src/discovery/`, `src/persistence/`, `src/quantization/`

## 3. Enforcement

- [ ] 3.1 Flip `#![warn(missing_docs)]` to `#![deny(missing_docs)]` once zero remain
- [ ] 3.2 Add `.github/workflows/rust-docs.yml` running `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"` on every PR
- [ ] 3.3 Add `#![doc = include_str!("../README.md")]` (or curated text) at the top of `src/lib.rs`

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Publish `cargo doc` output to `docs/rustdoc/` or link from README
- [ ] 4.2 Add a test or CI check that runs `cargo doc` as part of the normal build
- [ ] 4.3 Run `cargo test --all-features --doc` (runs doc-tests) and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
