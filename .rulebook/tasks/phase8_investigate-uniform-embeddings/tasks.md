## 1. Reproduce

- [ ] 1.1 Write a minimal repro script at
  `scripts/investigate_uniform_embeddings.py` that creates a
  collection, inserts 5 texts with distinct content, and dumps the
  stored vector for each via `POST /vector` (or equivalent). Confirm
  whether the first ~40 components are constant across all 5.
- [ ] 1.2 Identify the active embedding provider at boot: grep the
  server logs for the "embedding provider initialized" line, or
  query `GET /config`. Record provider name, model path, dim.

## 2. Root-cause

- [ ] 2.1 If BM25 / tfidf fallback: check the token vocabulary for
  the probe texts — are the "unique phrase alpha-N beta-M gamma-K"
  tokens actually entering the vocabulary, or is the tokenizer
  stripping them? Inspect the provider state via test hooks.
- [ ] 2.2 If fastembed / ort: isolate the same input against a
  v2.1.x checkout of the embedding crate and compare byte-for-byte.
  Bisect dep bumps (fastembed first since it moved most: 5.4 → 5.13).
- [ ] 2.3 If neither: suspect serialization. Insert with provider A,
  read back, compare raw bincode payload against expected — verify
  `hmac 0.13` / `sha2 0.11` / `bincode 2` round-trip.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update or create documentation covering the implementation
  (append the resolution section to
  `docs/releases/v3.0.0-verification.md#f4`; if a code fix is needed,
  open a follow-up `phase8_fix-...` task and link it from the doc).
- [ ] 3.2 Write tests covering the new behavior (regression test that
  catches the failure mode — e.g. inserts 5 distinct texts and
  asserts pairwise cosine similarity of their embeddings is < 0.95).
- [ ] 3.3 Run tests and confirm they pass (the new regression test
  plus `cargo test -p vectorizer embedding::`).
