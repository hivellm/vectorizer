## 1. WAL durability

- [ ] 1.1 Add fsync (`sync_data`) after flush in WAL append and checkpoint paths (`wal.rs:198,217`), behind a `wal.fsync` config flag defaulting to true
- [ ] 1.2 Add per-record CRC32 + length-prefix framing to WAL writes (`crc32fast` already in deps)
- [ ] 1.3 Make WAL reader accept both framed records and legacy JSON-lines; tolerate a trailing torn/corrupt record instead of aborting recovery
- [ ] 1.4 Compute transaction sequence numbers under the file lock (fix race at `wal.rs:179`)
- [ ] 1.5 Store `max_sequence + 1` after recovery (fix duplicate-sequence off-by-one at `wal.rs:149`)

## 2. BM25 vocabulary persistence

- [ ] 2.1 Replace the tokenizer stub in `autosave.rs:398-426,508-534` with a real `save_vocabulary_json` call for BM25 collections
- [ ] 2.2 Wire `load_vocabulary_json`/`restore_vocabulary_data` into the collection load path so restarts restore the BM25 space
- [ ] 2.3 Add a fallback guard: if vocab load fails, log at warn and surface a collection health flag instead of silently hashing

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update or create documentation covering the implementation
- [ ] 3.2 Write tests covering the new behavior (WAL torn-write recovery, sequence continuity, BM25 save→restart→search round-trip)
- [ ] 3.3 Run tests and confirm they pass
