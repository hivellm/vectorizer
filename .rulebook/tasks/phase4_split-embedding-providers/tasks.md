## 1. Layout

- [ ] 1.1 Create per-provider files under `src/embedding/`: bm25, bert, minilm, tfidf, svd, bow, char_ngram.
- [ ] 1.2 Extract `EmbeddingManager` to `src/embedding/manager.rs`.
- [ ] 1.3 Keep `src/embedding/mod.rs` as the re-export hub.

## 2. Verification

- [ ] 2.1 `cargo check --all-features` clean.
- [ ] 2.2 Existing embedding tests pass without edits.
- [ ] 2.3 Every new file is under 500 lines.

## 3. Tail (mandatory)

- [ ] 3.1 Update module-level doc comment.
- [ ] 3.2 No new tests required — layout refactor only.
- [ ] 3.3 `cargo test --all-features` pass.
