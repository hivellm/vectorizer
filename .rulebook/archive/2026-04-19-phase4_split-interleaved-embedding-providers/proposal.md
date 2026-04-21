# Proposal: phase4_split-interleaved-embedding-providers

## Why

`phase4_split-embedding-providers` extracted the three contiguous tail
providers (BagOfWords, CharNGram, EmbeddingManager facade) from the
1,788-line `embedding/mod.rs`, cutting it to 1,178 lines. The remaining
five providers — TfIdf, Bm25, Svd, Bert, MiniLm — are harder to move
because their impls are **interleaved**: e.g. `Bm25Embedding`'s
inherent impl sits at L115-382 but its `EmbeddingProvider` trait impl
is down at L996-1144; `TfIdfEmbedding` is split across three non-
contiguous segments (struct decl + two inherent impls + one trait
impl). A mechanical `sed` extraction won't work — the split needs a
careful per-provider re-assembly.

## What Changes

For each of the five remaining providers:

1. Collect every block that belongs to that provider (struct + all
   inherent impls + the trait impl).
2. Reassemble into a single per-provider file under
   `src/embedding/providers/`:
   - `tfidf.rs`
   - `bm25.rs`
   - `svd.rs` (depends on `tfidf.rs` — document the dependency)
   - `bert.rs`
   - `minilm.rs`
3. Update `providers/mod.rs` to declare and re-export each.
4. Result: `embedding/mod.rs` keeps only the `EmbeddingProvider`
   trait + the `pub mod` / `pub use` wiring.

## Impact

- Affected specs: none.
- Affected code: `src/embedding/mod.rs`, `src/embedding/providers/`.
- Breaking change: NO — public re-exports preserved.
- User benefit: every provider reviewable in isolation; per-file
  line counts under the 500-line threshold; the trait surface that
  every provider implements becomes the module's focal point
  instead of being buried at the top of a 1,700-line file.
