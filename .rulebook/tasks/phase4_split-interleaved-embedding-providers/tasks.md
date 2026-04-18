## 1. Per-provider extraction

- [ ] 1.1 `tfidf.rs` — collect struct (L57-61 of the pre-phase4 file) + inherent impls (L763-907 + L908-995) + trait impl (L854-907).
- [ ] 1.2 `bm25.rs` — struct (L64-73) + inherent (L115-382) + trait (L996-1144).
- [ ] 1.3 `svd.rs` — struct (L76-85) + inherent + trait (L383-499). Depends on `tfidf.rs`; document the dependency.
- [ ] 1.4 `bert.rs` — struct (L88-99) + inherent + trait (L500-631).
- [ ] 1.5 `minilm.rs` — struct (L102-113) + inherent + trait (L632-762).

## 2. Wiring

- [ ] 2.1 Add `mod tfidf; mod bm25; mod svd; mod bert; mod minilm;` to `providers/mod.rs` + `pub use` lines for each struct.
- [ ] 2.2 Remove all five struct/impl blocks from `embedding/mod.rs`; keep only the `EmbeddingProvider` trait, `candle_models` gating, and `pub mod providers;`.

## 3. Verification

- [ ] 3.1 `cargo check --all-features` clean.
- [ ] 3.2 1122/1122 lib + 780/780 integration still passes.
- [ ] 3.3 Every file under 500 lines.

## 4. Tail (mandatory)

- [ ] 4.1 Update module-level doc comments on every new file.
- [ ] 4.2 Existing tests sufficient; layout-only refactor.
- [ ] 4.3 Run `cargo test --all-features` and confirm pass.
