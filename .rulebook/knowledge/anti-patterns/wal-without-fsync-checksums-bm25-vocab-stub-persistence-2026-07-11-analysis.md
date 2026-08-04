# WAL without fsync/checksums + BM25 vocab stub persistence (2026-07-11 analysis)

**Category**: architecture
**Tags**: analysis:2026-07-11-improvement-analysis, wal, durability, bm25, persistence, phase37, phase38, phase39, phase40, phase41

## Description

Two CRITICAL defects found by the 2026-07-11 improvement analysis (docs/analysis/2026-07-11-improvement-analysis/): (1) persistence/wal.rs only flush()es, never fsyncs, and stores JSON-lines with no CRC/length framing — power loss silently drops acknowledged writes and a torn final line aborts recovery of all later entries (wal.rs:198,217,222-226; sequence race :179; off-by-one :149). (2) autosave.rs save_collection_tokenizer writes a stub (vocab_size:0) instead of calling save_vocabulary_json; load_vocabulary_json has zero callers — after restart BM25 queries fall to the hash fallback space and search returns nothing until re-index (reproduced during v3.4.0 Docker validation). Fix tasks: phase37 (durability), phase38 (hot-path locks/PQ/SIMD), phase39 (test harness), phase40 (API parity/hardening), phase41 (decoupling). Full findings with file:line evidence in the analysis directory.

## When to Use

Consult before implementing phase37-41 or touching persistence/wal.rs, autosave.rs, bm25.rs, insert_batch, capabilities.rs, or the config loader.
