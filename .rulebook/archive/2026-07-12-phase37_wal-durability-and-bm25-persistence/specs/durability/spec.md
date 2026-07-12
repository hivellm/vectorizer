# Durability Spec — WAL & BM25 Persistence

## ADDED Requirements

### Requirement: WAL fsync on append

The WAL SHALL issue an fsync (`sync_data` or equivalent) after every
append and checkpoint flush when `wal.fsync` is enabled (default:
enabled).

#### Scenario: Power loss after acknowledged append

Given a WAL with `wal.fsync` enabled
When a transaction append returns Ok and the process is killed
immediately (before OS page-cache writeback)
Then recovery after restart MUST replay that transaction

### Requirement: WAL record integrity framing

Each WAL record MUST be written with a length prefix and a CRC32
checksum. The reader MUST verify the checksum before deserializing.

#### Scenario: Torn final record does not abort recovery

Given a WAL file whose final record is truncated mid-write
When recovery runs
Then all complete records before the torn record MUST be replayed
And the torn record MUST be discarded with a warning log
And recovery MUST NOT return an error

#### Scenario: Legacy JSON-lines WAL still readable

Given a WAL file written by a pre-framing version (plain JSON lines)
When recovery runs
Then all valid legacy records MUST be replayed

### Requirement: Monotonic unique WAL sequence numbers

WAL sequence numbers MUST be unique and strictly increasing across
concurrent appends and across restart/recovery boundaries.

#### Scenario: Concurrent appends

Given two transactions appended concurrently from different threads
When both appends complete
Then their sequence ranges MUST NOT overlap

#### Scenario: Append after recovery

Given a WAL recovered with maximum sequence N
When the next transaction is appended
Then it MUST receive a sequence strictly greater than N

### Requirement: BM25 vocabulary persisted by auto-save

Auto-save MUST persist the complete BM25 vocabulary (terms, document
frequencies, statistics) for every BM25-backed collection — not a
stub.

#### Scenario: Vocabulary file content after auto-save

Given a BM25 collection with at least one indexed document
When auto-save completes
Then the persisted tokenizer file MUST contain the full vocabulary
with `vocab_size` greater than zero

### Requirement: BM25 vocabulary restored on load

Collection load MUST restore the persisted BM25 vocabulary so query
embeddings after a restart occupy the same vector space as stored
vectors.

#### Scenario: Search works after restart

Given a BM25 collection indexed and auto-saved, then the server is
restarted
When a text query that matched before the restart is searched
Then the search MUST return the same top result without re-indexing

#### Scenario: Missing vocabulary is surfaced

Given a persisted BM25 collection whose vocabulary file is absent or
corrupt
When the collection loads
Then the server MUST log a warning and expose a degraded-health flag
for that collection instead of silently using the hash fallback
