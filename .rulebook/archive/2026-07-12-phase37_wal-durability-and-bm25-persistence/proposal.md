# Proposal: phase37_wal-durability-and-bm25-persistence

Source: docs/analysis/2026-07-11-improvement-analysis/ (§1.4, §4.1 item 3)

## Why

Two CRITICAL findings from the 2026-07-11 improvement analysis cause
silent data loss and a confirmed search regression:

1. **WAL is not durable.** `persistence/wal.rs` only calls
   `file.flush()` (lines 198, 217) — never `sync_all()`/`sync_data()`.
   Acknowledged writes sit in the OS page cache; a power loss or hard
   kill drops entries the caller believes durable. Additionally the
   WAL is JSON-lines with no per-record checksum or length framing
   (lines 222-226): a torn final write on crash produces a partial
   line, and `serde_json::from_str` (line 145) aborts recovery of
   **all subsequent entries**. Two smaller defects compound this: the
   sequence number is read outside the file lock
   (`append_transaction`, line 179) allowing overlapping sequences
   under concurrency, and post-recovery `sequence.store(max_sequence)`
   (line 149) duplicates the last sequence on the next append.

2. **BM25 vocabulary is not persisted by auto-save.**
   `db/vector_store/autosave.rs` `save_collection_tokenizer`
   (lines 398-426, 508-534) writes a stub (`"vocab_size": 0`, empty
   vocab). The real `save_vocabulary_json` (`bm25.rs:143`) is only
   invoked from the file-watcher indexer; `load_vocabulary_json` /
   `restore_vocabulary_data` have zero callers in `src/`. After a
   restart the BM25 provider is empty, query embedding falls back to
   the hash path (`bm25.rs:385-425`) — a vector space disjoint from
   the stored vectors — and **search returns nothing until a full
   re-index**. This was reproduced manually during the v3.4.0 Docker
   validation.

## What Changes

- `persistence/wal.rs`: fsync after flush on append and checkpoint
  (configurable, default on); per-record CRC32 (crate already depends
  on `crc32fast`) + length-prefix framing with backward-compatible
  reader for legacy JSON-lines WALs; compute sequence numbers under
  the file lock; store `max_sequence + 1` after recovery; recovery
  skips a trailing torn record instead of aborting.
- `db/vector_store/autosave.rs`: replace the tokenizer stub with a
  real call into `save_vocabulary_json` for BM25-backed collections.
- Collection load path (`load_all_persisted_collections`): call
  `load_vocabulary_json` / `restore_vocabulary_data` so a restarted
  server embeds queries in the same space as stored vectors.
- Regression tests: WAL torn-write recovery, WAL sequence continuity,
  BM25 save→restart→search round-trip.

## Impact

- Affected specs: `specs/durability/spec.md` (new, in this task)
- Affected code: `crates/vectorizer/src/persistence/wal.rs`,
  `crates/vectorizer/src/db/vector_store/autosave.rs`,
  `crates/vectorizer/src/db/vector_store/persistence.rs`,
  `crates/vectorizer/src/embedding/providers/bm25.rs`
- Breaking change: NO (WAL format change ships with a
  backward-compatible reader; new writes use framed format)
- User benefit: no silent data loss on power failure; search works
  after restart without re-indexing
