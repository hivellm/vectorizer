## 1. Per-provider extraction

- [x] 1.1 `tfidf.rs` - collected struct + both non-contiguous inherent impls + trait impl (253 lines).
- [x] 1.2 `bm25.rs` - struct + 267-line inherent impl + 147-line trait impl (441 lines).
- [x] 1.3 `svd.rs` - struct + inherent + trait (138 lines). Depends on `tfidf.rs`; documented at top of the file and exposed the `dimension()` method so `SvdEmbedding::fit_svd` can read the TF-IDF vocabulary size without touching a private field.
- [x] 1.4 `bert.rs` - struct + inherent + trait (159 lines).
- [x] 1.5 `minilm.rs` - struct + inherent + trait (156 lines).

## 2. Wiring

- [x] 2.1 Added `mod tfidf; mod bm25; mod svd; mod bert; mod minilm;` to `providers/mod.rs` with `pub use` lines for all seven provider types. `tfidf` is declared `pub(super)` so `svd` can import the concrete type.
- [x] 2.2 Stripped all five struct/impl blocks out of `embedding/mod.rs`. mod.rs now carries only the `EmbeddingProvider` trait, the `candle_models` feature gate, the `providers` declaration, the re-exports, and the cache / real_models / fast_tokenizer / onnx wiring - 86 lines total (was 1176).

## 3. Verification

- [x] 3.1 `cargo check --lib --all-features`: clean. `cargo check --lib` (default features): clean - confirms `candle_models` cfg gating still works across files.
- [x] 3.2 `cargo test --lib --all-features`: 1158/1158 pass, 12 ignored (unchanged from pre-task baseline).
- [x] 3.3 File sizes: mod.rs 86, tfidf.rs 253, bm25.rs 441, svd.rs 138, bert.rs 159, minilm.rs 156, bag_of_words.rs 198, char_ngram.rs 211, manager.rs 252 - every file under 500 lines.

## 4. Tail (mandatory)

- [x] 4.1 Added module-level `//!` doc comments on every new file describing the provider and its dependencies. `embedding/mod.rs` doc block now documents the Layout.
- [x] 4.2 Layout-only refactor; existing test suite covers the public surface.
- [x] 4.3 `cargo clippy --lib --all-features -- -D warnings`: 0 warnings. `cargo fmt`: clean.
