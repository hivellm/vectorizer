# Design notes — phase37_wal-durability-and-bm25-persistence

## D1 — WAL record framing (tasks §1.2/§1.3)

Current format: bare JSON-lines. New format stays **line-oriented** so
`BufRead::lines()` keeps working and old files remain readable:

```
C1 <crc32-hex8> <payload-len-decimal> <json>\n
```

Reader rules (single helper `read_entries_from_file` used by
`initialize_sequence`, `read_from`, `read_collection_entries`,
`validate_integrity`, `get_stats`):

1. Line starts with `"C1 "` → parse crc + len + payload. `len`
   mismatch OR crc mismatch on a **final** line → torn write: warn,
   discard, stop (recovery continues with everything before it).
   Same failure on a **non-final** line → `WALError::Corruption`
   (that is real corruption, not a torn append).
2. Otherwise → legacy bare-JSON line, parse as before (back-compat).
   Unparseable **final** legacy line → warn + discard (torn).
   Unparseable non-final line → `SerializationError` as today.

`crc32fast` is already a workspace dep.

## D2 — sequence fixes (tasks §1.4/§1.5)

- `append_transaction` currently reads `sequence` **before** taking
  the file lock (`wal.rs:179`) and `fetch_add`s after writing —
  two concurrent transactions compute the same base. Fix: reserve the
  whole range up-front with
  `let base = self.sequence.fetch_add(n as u64, Ordering::SeqCst)`
  and delete the post-write `fetch_add`.
- `initialize_sequence` stores `max_sequence` — the next
  `fetch_add(1)` **returns** `max_sequence`, duplicating the last
  entry's seq after every restart. Fix: store `max_sequence + 1` when
  the file has at least one entry (empty file keeps 0).
- `validate_integrity` expects dense `0..N` — stays valid after the
  fix (global sequences remain dense; the current code is the thing
  that breaks density after restart).

## D3 — fsync (tasks §1.1)

`WALConfig.fsync: bool` (default **true**). After `file.flush()` in
`write_entry` and `append_transaction`, call `file.sync_data()`.
`sync_data` (fdatasync) is sufficient — sequence/length metadata
lives inside the file content, not in inode metadata we care about.
Checkpoint truncate path: `sync_data` the new file handle after
rename so an immediately-following crash can't resurrect stale
entries.

## D4 — BM25 vocabulary restore strategy (tasks §2)

Query-time embedding uses ONE global `EmbeddingManager`
(`bootstrap.rs:262`, `state.embedding_manager.embed(query)` in
search.rs — no collection argument). `Bm25Embedding::build_vocabulary`
REBUILDS vocab indices from a corpus (frequency-sorted, not
append-stable). Consequences:

- Collections indexed at different times already live in different
  vocab-index spaces pre-restart when the global vocab was rebuilt
  between them — that is a pre-existing design flaw, tracked by
  phase38/phase41 scope (per-collection providers), NOT this task.
- The restart regression this task fixes: today the vocab is
  **entirely lost** on restart (stub file, zero load callers), so
  every text query hash-falls-back and returns nothing.
- Correct restore for the current architecture = load the **newest**
  persisted tokenizer snapshot (max `created_at`) into the global
  provider at boot. That reproduces exactly the pre-restart global
  vocab state — same top results for every collection that worked
  before the restart.

Implementation:

1. **Save side**: `VectorStore` gets an injectable
   `tokenizer_saver: RwLock<Option<Arc<dyn Fn(&str, &Path) -> Result<()> + Send + Sync>>>`.
   Bootstrap registers a closure capturing the global
   `EmbeddingManager` that calls
   `manager.save_vocabulary_json(default_provider, path)`.
   `save_collection_tokenizer{,_static}` use it when set; the stub
   remains only as a warn-logged fallback when no saver is registered
   (bare-VectorStore test environments).
2. **Load side**: add `load_vocabulary_json` to the
   `EmbeddingProvider` trait (default: no-op Err(Unsupported)); wire
   bm25/tfidf/bag_of_words/char_ngram inherent loaders; add manager
   dispatch. Bootstrap, after `load_all_persisted_collections`, scans
   `data_dir` for `*_tokenizer.json`, picks the newest with
   `vocab_size > 0`/real payload, and loads it into the default
   provider.
3. **Health surfacing**: collections whose tokenizer file is missing
   or stub (`vocab_size: 0`) get a warn log + a `degraded_bm25: true`
   flag exposed through collection health (spec scenario "Missing
   vocabulary is surfaced"). No silent hash fallback at boot.

## D5 — sub-task decomposition (sequential-editing rule)

| # | Files | Verify |
|---|---|---|
| A | `persistence/wal.rs` | wal unit tests incl. new torn-write/seq tests |
| B1 | `embedding/mod.rs` + `providers/manager.rs` | cargo check |
| B2 | `providers/{bm25,tfidf,bag_of_words,char_ngram}.rs` trait wiring | provider unit tests |
| B3 | `db/vector_store/{mod,autosave}.rs` saver injection | cargo check |
| B4 | `bootstrap.rs` register saver + boot restore + health flag | integration test |
| C | tests: WAL recovery + BM25 save→restart→search round-trip | cargo test |
