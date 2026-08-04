# Round-trip tests expose silent persistence format drift (phase37)

**Category**: testing
**Tags**: persistence, testing, round-trip, wal, phase37, analysis:2026-07-11-improvement-analysis

## Description

The phase37 BM25 save→restart→search round-trip test immediately exposed a second, unrelated latent bug: the legacy instance save path serialized a bare PersistedCollection while load_persisted_collection requires the versioned PersistedVectorStore envelope — every legacy-format save was unreadable on the next boot ("missing field 'version'"), silently warned and counted as 0 loaded. Save-side unit tests and load-side unit tests both passed individually; only the round-trip caught it. Pattern: for any save/load pair, write at least one test that persists through the REAL writer and reloads through the REAL reader in the same test, asserting on behavior (search result), not file existence. Applied in crates/vectorizer/tests/bm25_vocab_persistence.rs. Also relevant: WAL framing kept line-oriented (C1 <crc> <len> <json>) so BufRead::lines() and legacy files keep working — backward-compatible framing beats binary reformat when both readers must coexist.

## When to Use

When touching any persistence writer/reader pair (autosave, WAL, .vecdb, snapshots) or reviewing tests for them.
