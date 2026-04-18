# Proposal: phase4_split-embedding-providers

## Why

`src/embedding/mod.rs` is **1,788 lines** stitching together seven distinct embedding providers (BM25, BERT, MiniLM, TF-IDF, SVD, BOW, CharNGram) plus the `EmbeddingManager` that dispatches between them. Each provider has its own vocabulary, training loop, and persistence format — the only thing they share is a trait boundary. Seven concerns in one file is reviewer-hostile and hides provider-specific bugs.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Create `src/embedding/` with one file per provider:

- `bm25.rs`, `bert.rs`, `minilm.rs`, `tfidf.rs`, `svd.rs`, `bow.rs`, `char_ngram.rs` — one impl per file.
- `manager.rs` — `EmbeddingManager` dispatch facade.
- `traits.rs` — the shared `EmbeddingProvider` trait (if not already extracted).

Keep `src/embedding/mod.rs` thin: re-exports and module declarations.

## Impact

- Affected specs: none.
- Affected code: `src/embedding/mod.rs`, new per-provider files.
- Breaking change: NO — public surface preserved via re-export.
- User benefit: provider-specific work (e.g. fixing a BM25 IDF bug or tuning MiniLM attention) no longer drags reviewers through six unrelated providers.
