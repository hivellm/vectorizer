## 1. Layout

- [ ] 1.1 Create `sdks/rust/src/client/` and extract transport to `transport.rs`.
- [ ] 1.2 Extract per-surface modules (collections, vectors, search, graph, admin, auth).
- [ ] 1.3 Rewrite `client.rs` as `mod.rs` re-exporting the surface.
- [ ] 1.4 Shape `transport.rs::Transport` as an enum that today carries a single `Rest(...)` variant but is positioned so a sibling `Rpc(...)` variant from `phase6_sdk-rust-rpc` slots in without changing any per-surface module. Per-surface trait methods call through the transport — they must not hold a `reqwest::Client` directly.

## 2. Verification

- [ ] 2.1 `cargo check --all-features` in `sdks/rust/` clean.
- [ ] 2.2 `cargo test` in `sdks/rust/` passes unchanged.
- [ ] 2.3 No sub-file exceeds 600 lines.
- [ ] 2.4 Add a doc-test that sketches a `Transport::Mock(...)` (or equivalent) variant satisfying the same trait surface — proves the per-surface modules are not coupled to the concrete `Rest` variant and acts as the RPC-readiness regression guard.

## 3. Tail (mandatory)

- [ ] 3.1 Update `sdks/rust/README.md` — note that the new layout hosts the RPC client (`phase6_sdk-rust-rpc`) using the canonical `vectorizer://host:15503` URL scheme as the default transport.
- [ ] 3.2 No new tests required for the split itself; the doc-test from 2.4 doubles as the RPC-readiness regression guard.
- [ ] 3.3 Run the SDK tests and confirm pass.
