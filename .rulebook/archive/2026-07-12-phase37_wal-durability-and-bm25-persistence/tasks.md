## 1. WAL durability

- [x] 1.1 fsync (`sync_data`) after flush in append + transaction + checkpoint-truncate paths, behind `WALConfig.fsync` (default true); `wal_integration.rs` literal replaced with `unwrap_or_default()`
- [x] 1.2 CRC32 + length framing per record (`C1 <crc8> <len> <json>` lines, `crc32fast`); writer always frames
- [x] 1.3 Reader accepts framed + legacy bare-JSON lines; damaged FINAL line = torn append → warn + discard + recovery continues; damaged mid-file line → `WALError::Corruption` (tests: torn-final, mid-file, legacy)
- [x] 1.4 `append_transaction` reserves the full sequence range via `fetch_add(n)` up-front (fixes the same-base race; concurrency test: 8 tx × 5 ops = 40 disjoint sequences)
- [x] 1.5 `initialize_sequence` stores `max + 1` (fixes duplicate seq after restart; reopen test + dense validate_integrity)

## 2. BM25 vocabulary persistence

- [x] 2.1 `save_collection_tokenizer` uses the injected `TokenizerSaver` (bootstrap closure → `EmbeddingManager::save_vocabulary_json`); minimal-tokenizer fallback only when no saver is registered, logged at warn. Bonus: fixed a latent bug — the legacy save wrote a bare `PersistedCollection` while the loader requires the `PersistedVectorStore{version}` envelope, so every legacy-format save was unreadable on the next boot
- [x] 2.2 `EmbeddingProvider::load_vocabulary_json` on the trait + impls (bm25/tfidf/bow/char_ngram) + manager dispatch + `restore_vocabulary_from_disk` (raw files + `_tokenizer.json` entries inside .vecdb via StorageReader; snapshot with the largest (total_docs, vocab) wins — see design.md D4); called in bootstrap before the manager Arc-wrap
- [x] 2.3 Guard: missing/empty snapshots → warn + `degraded_vocabulary:{collection}=true` metadata on the VectorStore (`VocabularyRestoreReport.degraded_collections`)

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 Update or create documentation covering the implementation — design.md (D1-D5) + CHANGELOG [3.5.0] Fixed section + doc-comments at the integration points
- [x] 3.2 Write tests covering the new behavior — 5 new WAL unit tests (torn-final, mid-file corruption, legacy read, seq reopen, concurrent tx) + `tests/bm25_vocab_persistence.rs` full round-trip (save → restart → restore → same top hit) + degraded surfacing
- [x] 3.3 Run tests and confirm they pass — WAL lib 11/11, WAL integration 19 passed, bm25 round-trip 1/1, lib suite 1026 passed, clippy 0 warnings
