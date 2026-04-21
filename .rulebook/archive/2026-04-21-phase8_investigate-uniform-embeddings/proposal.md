# Proposal: phase8_investigate-uniform-embeddings

## Why

During probe 2.1 of `phase8_release-v3-runtime-verification` (seeding
100 synthetic texts with unique per-document content, then running
`POST /collections/{name}/search/text` with query "probe document
number forty two"), all top-5 hits returned `score: 1.0` with
stored vectors whose first 40+ components were a constant value
(~0.0436). That is either:

1. The default embedder (likely BM25 or the tfidf fallback) emits a
   pathological vector for these synthetic inputs — which would make
   the v3 out-of-the-box search experience useless on similar-shaped
   inputs.
2. A regression from one of the v3 dep bumps — fastembed 5.4 → 5.13,
   ort 2.0.0-rc.10 → rc.11, candle 0.9 → 0.10, hf-hub 0.4 → 0.5, or
   tantivy 0.25 → 0.26.
3. A behavior change from hmac 0.12 → 0.13 / sha2 0.10 → 0.11 /
   bincode 1.x → 2 that corrupted stored vectors during persist/load
   (unlikely since the test stayed in one process).

Source: `docs/releases/v3.0.0-verification.md` finding F4.

## What Changes

Investigate the root cause — this is a research-first task with no
guaranteed code change. Phases:

1. Reproduce deterministically with a minimal script.
2. Identify which embedding provider is active at boot (log grep +
   `GET /config`).
3. If BM25/tfidf fallback: determine whether the inputs genuinely
   produce a degenerate vector (short text + too few unique tokens)
   or whether the provider has a bug.
4. If fastembed/ort: compare the produced vector for the same input
   against v2.x (pre-bump) to isolate a regression.
5. File a follow-up fix task if a real regression is found; otherwise,
   document the minimum input conditions that produce meaningful
   embeddings in `docs/EMBEDDING.md` and in `/health` surface if
   useful.

## Impact

- Affected specs: none (research task); may spawn a fix task that
  touches `crates/vectorizer/src/embedding/` or the provider selection
  in `crates/vectorizer-server/src/server/core/bootstrap.rs`.
- Affected code: depends on root cause.
- Breaking change: NO (research); maybe if a fix task lands with a
  different default provider for v3.
- User benefit: confidence that v3 default embeddings produce useful
  search results before the release ships.
