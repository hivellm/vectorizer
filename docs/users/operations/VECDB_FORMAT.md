---
title: .vecdb File Format
module: operations
id: vecdb-format-guide
order: 6
description: On-disk format for vectorizer data — what .vecdb and .vecidx actually contain, how to back them up, and how to recover
tags: [operations, backup, recovery, storage, vecdb, vecidx, format]
---

# .vecdb File Format — Operations Guide

> Scope: this document describes what lives on disk in a Vectorizer data
> directory, and what operators need to know about it for backups,
> verification, and recovery. It is deliberately source-backed — every
> statement here maps to a concrete path under
> `crates/vectorizer/src/storage/` or `crates/vectorizer/src/persistence/`.

## Purpose — why this doc exists

Every production Vectorizer installation persists its data in two files:
`vectorizer.vecdb` and `vectorizer.vecidx`. These files are the
single source of truth for every collection. If either file is lost
or corrupted, collections cannot be loaded without a prior backup.

Despite being production-critical, the format was not previously
documented at an operator level. This page fills that gap. It is aimed
at SREs and platform operators who need to:

- Back up and restore a live installation correctly
- Verify archive integrity after a copy, snapshot, or restore
- Diagnose "collection failed to load" errors
- Plan cross-version upgrades without data loss

This is not a developer reference. For the internal spec, see
[`docs/specs/STORAGE.md`](../../specs/STORAGE.md).

## TL;DR

- `vectorizer.vecdb` is a **ZIP archive** (DEFLATE compression) — not gzip,
  not a raw binary dump. You can open it with any standard `unzip` tool for
  inspection.
- `vectorizer.vecidx` is a **JSON sidecar** listing every file in the archive
  with its **SHA-256** checksum.
- There is exactly **one** `.vecdb` and **one** `.vecidx` per installation,
  not per collection. Collections live as separate entries **inside** the
  ZIP.
- Both files must be backed up **together**. Losing one means data loss
  (see [Recovery](#recovery-from-partial-corruption)).
- The archive is produced atomically: writes go to `.tmp` files and are
  renamed into place only after success.

## Files on disk

A healthy Vectorizer data directory looks like this:

```
data/
├── vectorizer.vecdb     # ZIP archive (all collections)
├── vectorizer.vecidx    # JSON index with SHA-256 checksums
└── snapshots/           # Optional periodic snapshots (see below)
    └── 20260424_120000/
```

Constants that define these paths live in
`crates/vectorizer/src/storage/mod.rs`:

```rust
pub const STORAGE_VERSION: &str = "1.0";
pub const VECDB_FILE:   &str = "vectorizer.vecdb";
pub const VECIDX_FILE:  &str = "vectorizer.vecidx";
pub const TEMP_SUFFIX:  &str = ".tmp";
pub const SNAPSHOT_DIR: &str = "snapshots";
```

When the server starts, `storage::detect_format()` looks at `data/`
and decides:

- `StorageFormat::Compact` if `vectorizer.vecdb` exists — the normal case.
- `StorageFormat::Legacy` if only individual `*_vector_store.bin` files
  exist — the server will migrate them to `.vecdb` on demand.

For the migration path, see [Cross-version migration](#cross-version-migration).

## What is inside `vectorizer.vecdb`

### Outer layer — ZIP archive

The `.vecdb` file is a standard ZIP archive. It is created by the
[`zip`](https://crates.io/crates/zip) crate with
`CompressionMethod::Deflated` — the same DEFLATE algorithm used by
`.zip` files everywhere. Source: `storage/writer.rs`.

Practical consequences:

- `unzip -l /var/lib/vectorizer/data/vectorizer.vecdb` works and will
  list every file inside.
- `file vectorizer.vecdb` reports `Zip archive data`.
- The STORAGE.md reference previously (and still) calls this a
  "compressed ZIP archive" — that description is **accurate**. Do not
  confuse it with raw gzip (`.gz`) or bincode.
- The compression level is configurable
  (`storage.compression.level`, 1–22, default 3) but the wire format is
  DEFLATE regardless. The `compression.format: "zstd"` key in
  `StorageConfig` is validated but the writer currently emits DEFLATE.
  If you are tuning space usage, treat the level knob as advisory.

### Inner layout — per-collection files

Inside the ZIP there is a flat layout, one set of files per collection:

| Path in archive                     | Purpose                                                     | Required? |
|-------------------------------------|-------------------------------------------------------------|-----------|
| `<collection>_vector_store.bin`     | Vectors + payloads as JSON (`PersistedVectorStore`)         | Yes       |
| `<collection>_metadata.json`        | `CollectionConfig` + timestamps + vector count              | Recommended |
| `<collection>_tokenizer.json`       | BM25 vocabulary (only for BM25 collections)                 | BM25 only |
| `<collection>_checksums.json`       | Per-vector integrity checksums (optional)                   | Optional  |

The `_vector_store.bin` name is historical; the payload is **JSON**, not
a binary dump. The reader (`storage/reader.rs`) says so explicitly:

```rust
// Files are saved as JSON, not bincode
let json_str = std::str::from_utf8(vector_data)?;
let persisted_store: PersistedVectorStore = serde_json::from_str(json_str)?;
```

### `PersistedVectorStore` — the actual data structure

From `crates/vectorizer/src/persistence/mod.rs`:

```rust
pub struct PersistedVectorStore {
    pub version: u32,                           // Currently 1
    pub collections: Vec<PersistedCollection>,
}

pub struct PersistedCollection {
    pub name: String,
    pub config: Option<CollectionConfig>,       // Dim, metric, HNSW params, …
    pub vectors: Vec<PersistedVector>,
    pub hnsw_dump_basename: Option<String>,
}

pub struct PersistedVector {
    id: String,
    data: Vec<f32>,                             // Raw vector components
    payload_json: Option<String>,               // Payload re-serialized as JSON string
    normalized: bool,                           // True if L2-normalized for cosine
}
```

Important operator notes:

- `version: 1` is the **only** accepted value today. Anything else is
  rejected with `Unsupported vector store version: {n}`
  (`persistence/mod.rs`).
- Payloads are stored as **escaped JSON strings** (`payload_json`) inside
  the outer JSON document. This is intentional — it keeps the document
  serializable without custom `serde` adapters.
- HNSW indexes are **not** stored on disk. They are rebuilt in memory
  on load from the vector data. Expect a rebuild cost proportional to
  `vector_count` on every server start.

### Legacy save path

`VectorStore::save()` (used by some background save paths and
compatibility code) writes a **different** file layout: a single
`<collection>_vector_store.bin` that is gzip-compressed JSON (not a ZIP
archive). The unified `.vecdb` archive wraps these files, but if you
see a bare `*_vector_store.bin` sitting in `data/` with a gzip magic
header, that is the legacy format. Running `vectorizer storage migrate`
converts them to `.vecdb`.

## What is inside `vectorizer.vecidx`

`.vecidx` is a **pretty-printed JSON document** describing everything
in the archive. The schema is defined by `StorageIndex` in
`crates/vectorizer/src/storage/index.rs`.

Representative example:

```json
{
  "version": "1.0",
  "created_at": "2026-04-24T12:00:00Z",
  "updated_at": "2026-04-24T13:00:00Z",
  "collections": [
    {
      "name": "docs",
      "files": [
        {
          "path": "docs_vector_store.bin",
          "size": 10485760,
          "compressed_size": 5242880,
          "checksum": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          "type": "vectors"
        },
        {
          "path": "docs_metadata.json",
          "size": 512,
          "compressed_size": 256,
          "checksum": "…",
          "type": "metadata"
        }
      ],
      "vector_count": 12345,
      "dimension": 768,
      "metadata": {}
    }
  ],
  "total_size": 10485760,
  "compressed_size": 5242880,
  "compression_ratio": 0.5
}
```

Field reference (exact names as produced by the code):

- `version` — **string**, currently `"1.0"` (the `STORAGE_VERSION` constant).
- `created_at`, `updated_at` — RFC 3339 UTC timestamps (`chrono::DateTime<Utc>`).
- `collections[]` — list of `CollectionIndex`:
  - `name` — collection identifier.
  - `files[]` — list of `FileEntry`:
    - `path` — filename inside the ZIP.
    - `size` — original uncompressed size (bytes).
    - `compressed_size` — DEFLATE-compressed size (bytes, approximate for some entries).
    - `checksum` — **SHA-256** of the uncompressed content, as a 64-character lowercase hex string.
    - `type` — one of `vectors`, `metadata`, `config`, `index`, `tokenizer`, `other` (lowercase).
  - `vector_count`, `dimension`.
  - `metadata` — free-form string/string map (usually empty).
- `total_size`, `compressed_size` — sums across all files.
- `compression_ratio` — `compressed_size / total_size` (0.0–1.0).

> **Correction to earlier docs**: the checksum algorithm is **SHA-256**,
> not CRC32. If tooling in your runbooks expects CRC32, update it.
> The hex string in `checksum` is exactly 64 characters long; CRC32
> values are 8 characters long.

## Versioning & compatibility

Two version numbers coexist:

| Where | Constant | Current value | Type |
|-------|----------|---------------|------|
| `.vecidx` → `version` | `STORAGE_VERSION` | `"1.0"` | string |
| `.vecdb` inner JSON → `version` | hard-coded in `PersistedVectorStore` | `1` | `u32` |

The inner JSON is deserialized with `serde_json`, so it does tolerate:

- **Added optional fields** (marked `#[serde(default)]`), e.g. `name`,
  `config`, and `vectors` on `PersistedCollection`.
- Field reordering — JSON is order-independent.

It does **not** tolerate:

- Changing `version` away from `1` — the loader rejects the file.
- Renaming existing fields without a compatibility alias.
- Changing a field's type in an incompatible way (e.g. string → u32).

Practical rule: a newer server can almost always read an older
`.vecdb`, but an older server may fail to read a `.vecdb` produced by
a newer server if that newer server introduced required fields.
**Always back up before upgrading the binary.**

There is no automated downgrade. If you roll back the server, assume
you may have to restore the pre-upgrade backup.

## Backup procedure

The archive write path (`StorageWriter::write_from_memory`) is already
atomic: it writes `vectorizer.vecdb.tmp` + `vectorizer.vecidx.tmp`,
then `fs::rename` both into place only if the archive creation
succeeded. That means a crash mid-save cannot corrupt the live files.
However, a backup taken mid-save can capture a partially written pair
of `.tmp` files if you are not careful.

Recommended procedure:

1. Pause the service, or drain writes to it:
   - `systemctl stop vectorizer` on Linux, or
   - `Stop-Service Vectorizer` on Windows, or
   - disable ingestion at the load balancer.
2. Confirm no `*.tmp` files are left in the data directory:
   ```bash
   ls /var/lib/vectorizer/data/*.tmp 2>/dev/null && echo "WAIT - write in progress"
   ```
3. Copy **both** `vectorizer.vecdb` and `vectorizer.vecidx` together,
   using a single tool invocation so they end up in the same backup:
   ```bash
   tar -czf vectorizer-$(date +%Y%m%d_%H%M%S).tar.gz \
       -C /var/lib/vectorizer/data \
       vectorizer.vecdb vectorizer.vecidx
   ```
4. Verify the archive by listing it and by re-reading the index:
   ```bash
   vectorizer storage info
   vectorizer storage verify
   ```
5. Restart the service.

If you cannot stop the service, take a **filesystem-level snapshot**
(LVM, ZFS, EBS snapshot, `btrfs snapshot`) instead of a userspace
copy. Atomic rename on a snapshotted filesystem preserves the
`.vecdb`/`.vecidx` pair coherently.

Never back up one without the other. A `.vecdb` without its `.vecidx`
will still load on a fresh server because the archive is
self-describing, but you lose the SHA-256 checksums used to detect
corruption — see [Recovery](#recovery-from-partial-corruption).

## Restore procedure

1. Stop the server (`systemctl stop vectorizer`).
2. Move the current data directory aside — never overwrite:
   ```bash
   mv /var/lib/vectorizer/data /var/lib/vectorizer/data.old
   mkdir -p /var/lib/vectorizer/data
   ```
3. Extract the backup into the data directory:
   ```bash
   tar -xzf vectorizer-20260424_120000.tar.gz -C /var/lib/vectorizer/data
   chown -R vectorizer:vectorizer /var/lib/vectorizer/data
   ```
4. Verify with the CLI **before** starting the server:
   ```bash
   vectorizer storage info
   vectorizer storage verify
   ```
   - `storage info` should report `Format: Compact` and list the
     expected collections and vector counts.
   - `storage verify` re-opens the ZIP and revalidates entries.
5. Start the service and watch the logs:
   ```bash
   systemctl start vectorizer
   journalctl -u vectorizer -f | grep -Ei 'checksum|corrupt|failed to deserialize'
   ```
6. If you see `Failed to deserialize collection`, `Invalid UTF-8 in
   vector store`, or any `Storage(...)` error, **stop**. Do not accept
   writes. Restore a different backup.

## Recovery from partial corruption

| Lost or corrupted | Can recover? | How |
|-------------------|--------------|-----|
| `vectorizer.vecidx` only | Yes, with caveats | Regenerate by extracting and re-running the writer against the same data. You lose the cached SHA-256 values until the next save. Starting the server without a `.vecidx` will currently fail in `StorageReader::new` (`Index not found`). Restore the sidecar from the newest backup or trigger a full save. |
| `vectorizer.vecdb` only | No | The archive is authoritative. Restore from backup. |
| Both | No | Restore from backup. |
| Single collection inside the archive | Partially | Use `vectorizer storage verify` to detect corrupted entries. If the ZIP central directory is intact, other collections remain readable. |

Snapshots (if enabled) provide a second line of defence. With the
defaults in `StorageConfig`, the server keeps **48 snapshots** at
**1-hour** intervals for **2 days**. They live under
`data/snapshots/<YYYYMMDD_HHMMSS>/` and can be promoted back into
place with:

```bash
vectorizer snapshot list
vectorizer snapshot restore <id>
```

Snapshots are not a backup substitute — they live on the same
filesystem as the primary `.vecdb`. Treat them as fast rollback only.

## Cross-version migration

Most version bumps preserve on-disk compatibility because
`PersistedVectorStore.version` has not changed in the supported
2.x / 3.x line (the constant is still `1`). When an upgrade **does**
require a format bump, use this pattern:

1. Take a cold backup of `data/` (see [Backup procedure](#backup-procedure)).
2. Upgrade the server binary.
3. Start the server. On first boot it will:
   - Detect legacy `*_vector_store.bin` files and migrate them into
     `.vecdb` via `StorageMigrator::migrate()`.
   - Write a pre-migration backup under `data/backup_<timestamp>/` —
     keep this until you have verified the migration.
4. Run `vectorizer storage info` to confirm collection counts and
   dimensions match the pre-upgrade state.
5. Exercise the REST API (`/health`, `GET /collections`, a sample
   search) before re-enabling writes.

If the new server refuses to load the old `.vecdb` (hard version
bump), the fallback is an export/re-import:

1. Keep the old binary running.
2. On the old server, page through each collection via the REST API:
   ```bash
   GET /collections/<name>/vectors?limit=1000&offset=0
   ```
   and stream the results to disk as JSON.
3. Stop the old server, install the new server, start it against a
   **fresh** data directory.
4. Re-create each collection via
   `POST /collections` with the original `CollectionConfig`
   (read it from the `_metadata.json` entry in the old archive if you
   need to recover parameters).
5. Bulk-insert via `POST /collections/<name>/vectors`.
6. Retire the old server only after parity checks on counts, sample
   neighbour queries, and any domain-specific golden tests.

There is no standalone `vectorizer migrate` command for cross-version
moves today; the existing `vectorizer storage migrate` only handles
legacy-layout → `.vecdb` within the same format version.

## Operator CLI reference

The CLI shipped with the server (`vectorizer storage …`) is the
supported way to interrogate and repair archives. From
`crates/vectorizer-cli/src/cli/commands.rs`:

```bash
# Summary of archive contents (counts, compression ratio)
vectorizer storage info [--detailed]

# Convert legacy *_vector_store.bin files to .vecdb
vectorizer storage migrate [--force] [--level <1-22>]

# Re-open the archive and validate entries end-to-end
vectorizer storage verify [--fix]

# Rebuild the archive to reclaim space after many deletes
vectorizer storage compact [--force]
```

`storage verify` is cheap and non-destructive — run it after every
restore. `storage compact` rewrites the archive and should only be
run with the server stopped (or in a maintenance window) because it
touches the same files the server holds open.

## FAQ

**Can I open a `.vecdb` with standard ZIP tools?**
Yes. `unzip`, `7z`, or Python's `zipfile` can all list and extract
entries. Inspecting the inner `*_vector_store.bin` requires parsing
JSON. There is no native Python loader that rebuilds a working
collection — for programmatic access use the Vectorizer SDKs or REST
API against a running server.

**Is the format gzip?**
No. `.vecdb` is a ZIP archive. The legacy single-file save path
(`VectorStore::save()`) does use gzip, but the unified production
format is ZIP/DEFLATE. If you have previously seen claims that
`.vecdb` is "gzip-compressed bincode", they are incorrect.

**What checksum algorithm is used?**
SHA-256. Each `FileEntry.checksum` is a 64-character lowercase hex
string computed over the uncompressed file content. CRC32 is not used
anywhere in the persistence path.

**Can I edit a collection's metadata without re-indexing?**
No. Configuration and vectors are persisted together as a single
JSON document inside the ZIP. Use the REST API (`PATCH
/collections/<name>`) to update config — the server will re-persist
the archive safely.

**Does saving block reads?**
No. Writes go to `.tmp` files and are renamed into place atomically,
and `StorageReader` reopens the archive on demand. Running queries
continue against the in-memory HNSW index during the save.

**Where are HNSW indexes stored on disk?**
They are not persisted. HNSW graphs are rebuilt in memory from the
stored vectors on every start-up. `PersistedCollection.hnsw_dump_basename`
exists in the schema but is always `None` in current builds.

**How big should `.vecdb` be?**
Roughly `vectors × dimension × 4 bytes` (float32) plus payload size,
compressed at 50–70% of raw. `vectorizer storage info` reports the
exact numbers for your installation, including `compression_ratio`.

## Related

- [Backup and Restore](./BACKUP.md) — higher-level backup strategies and scheduling
- [Service Management](./SERVICE_MANAGEMENT.md) — stopping and starting the service cleanly
- [Troubleshooting](./TROUBLESHOOTING.md) — diagnosing load and checksum errors
- [`docs/specs/STORAGE.md`](../../specs/STORAGE.md) — internal storage specification
