## 1. Layout

- [ ] 1.1 Create per-provider files under `src/embedding/`: bm25, bert, minilm, tfidf, svd, bow, char_ngram — partial. The three contiguous providers at the tail of the file (BagOfWordsEmbedding, CharNGramEmbedding) + the EmbeddingManager facade have been extracted into `src/embedding/providers/{bag_of_words,char_ngram,manager}.rs`. The interleaved providers (TfIdf, Bm25, Svd, Bert, MiniLm) still sit in `embedding/mod.rs` — their impls are not contiguous (Bm25's inherent impl is at L115-382 but its EmbeddingProvider impl lives at L996-1144; TfIdf is split across three segments). Extracting them cleanly needs its own pass; tracked under `phase4_split-interleaved-embedding-providers`.
- [x] 1.2 Extract `EmbeddingManager` to `src/embedding/manager.rs` — delivered as `src/embedding/providers/manager.rs` (the per-provider sub-directory layout makes more sense than a flat `manager.rs` next to 7 provider files).
- [x] 1.3 Keep `src/embedding/mod.rs` as the re-export hub — re-exports `BagOfWordsEmbedding`, `CharNGramEmbedding`, `EmbeddingManager` via `pub use providers::{...};`.

## 2. Verification

- [x] 2.1 `cargo check --all-features` clean.
- [x] 2.2 Existing embedding tests pass without edits — 1122/1122 lib, 780/780 integration.
- [ ] 2.3 Every new file is under 500 lines — partially met. `bag_of_words.rs` (199), `char_ngram.rs` (212), `providers/mod.rs` (12) are well under. `manager.rs` is 249 lines (under). `embedding/mod.rs` is down from 1,788 to 1,178 — still over but a 610-line reduction. The full sub-500 target needs the interleaved-provider extraction noted in 1.1.

## 3. Tail (mandatory)

- [x] 3.1 Update module-level doc comment — every new file has its own `//!` header citing the split.
- [x] 3.2 No new tests required — layout refactor only (the existing tests in `manager.rs` already exercise every provider via `EmbeddingManager`).
- [x] 3.3 `cargo test --all-features` pass — 1122/1122 lib, 780/780 integration.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
