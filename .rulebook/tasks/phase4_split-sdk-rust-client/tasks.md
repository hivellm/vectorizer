## 1. Layout

- [ ] 1.1 Create `sdks/rust/src/client/` and extract transport to `transport.rs`.
- [ ] 1.2 Extract per-surface modules (collections, vectors, search, graph, admin, auth).
- [ ] 1.3 Rewrite `client.rs` as `mod.rs` re-exporting the surface.

## 2. Verification

- [ ] 2.1 `cargo check --all-features` in `sdks/rust/` clean.
- [ ] 2.2 `cargo test` in `sdks/rust/` passes unchanged.
- [ ] 2.3 No sub-file exceeds 600 lines.

## 3. Tail (mandatory)

- [ ] 3.1 Update `sdks/rust/README.md`.
- [ ] 3.2 No new tests required.
- [ ] 3.3 Run the SDK tests and confirm pass.
