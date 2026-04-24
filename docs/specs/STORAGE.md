# Vectorizer Storage System

## Overview

Vectorizer supports two storage formats:

1. **Legacy Format** - Individual files in directory structure
2. **Compact Format (.vecdb)** - Single compressed archive with index

The compact format provides compression, snapshots, and efficient backups while maintaining full compatibility with the legacy format.

## Storage Formats

### Legacy Format (Default for existing installations)

```
data/
├── collection_1/
│   ├── vectors.bin
│   ├── metadata.json
│   └── config.json
├── collection_2/
│   └── ...
```

**Characteristics:**
- One directory per collection
- Uncompressed binary files
- Fast random access
- Higher disk usage
- Suitable for active development

#### Legacy single-file format

A second, older path in `crates/vectorizer/src/persistence/mod.rs`
(`PersistedVectorStore::save` / `load`) writes each collection to an
individual file — typically named `<collection>.vecdb` or `<collection>.vecdb.gz`
depending on deployment. **This path is completely distinct from the compact
ZIP `.vecdb` archive above.** It is:

- **Gzip-compressed JSON**, not ZIP and not bincode. The save path calls
  `serde_json::to_string(&persisted)` and pipes the bytes through
  `flate2::GzEncoder` at `Compression::best()`.
- Wrapped in a `PersistedVectorStore { version: u32, collections: Vec<PersistedCollection> }`
  envelope (`version` is a `u32`, currently `1`; a mismatch is rejected on load).
- Backward-compatible: the loader first tries `GzDecoder`, then falls back to
  reading the file as plain-text JSON for older on-disk data.
- **No per-file checksum.** Integrity relies on the underlying filesystem and
  gzip's CRC-32 trailer; there is no SHA-256 sidecar in this path.

Because the format is JSON wrapped in gzip, a legacy file can be inspected
with `gunzip -c my_collection.vecdb.gz | jq .`. The compact ZIP `.vecdb`
cannot — it is a ZIP archive and must be opened with `unzip` first.

### Compact Format (.vecdb)

```
data/
├── vectorizer.vecdb     # Compressed ZIP archive
├── vectorizer.vecidx    # JSON index
└── snapshots/           # Automatic snapshots
    ├── 20241014_120000/
    └── 20241014_130000/
```

**Characteristics:**
- Single compressed archive for all collections
- 20-30% disk space reduction
- Snapshot support with retention policies
- Atomic updates (write to .tmp, then rename)
- Efficient backups and portability

**Archive internals:**
- The `.vecdb` file is a standard ZIP archive produced with `zip::ZipWriter` using `Deflated` compression, so it opens with any off-the-shelf tool (`unzip`, `7z`, Finder, Windows Explorer).
- Entries inside the ZIP are **JSON documents, not binary blobs**. Each collection is stored as a file named `<collection>_vector_store.bin` whose contents are `serde_json::to_vec(&PersistedVectorStore { version: 1, collections: vec![...] })`. The `.bin` suffix is historical — the bytes are UTF-8 JSON and can be inspected with `unzip -p archive.vecdb my_collection_vector_store.bin | jq .`.
- Optional per-collection metadata entries (config, tokenizer, etc.) are also JSON.
- Per-file integrity is guarded by **SHA-256** checksums stored in the sidecar — there is no CRC32 beyond the ZIP container's own built-in CRC-32.

## Index Format (.vecidx)

The `.vecidx` file is a plain JSON serialization of the `StorageIndex` struct
defined in `crates/vectorizer/src/storage/index.rs`. It is written alongside
the `.vecdb` archive on every save and is the authoritative source of integrity
and size information.

### `StorageIndex` fields (exactly as serialized)

| Field | Type | Description |
|-------|------|-------------|
| `version` | `String` | Format version, currently `"1.0"` (`STORAGE_VERSION` constant). |
| `created_at` | `DateTime<Utc>` | ISO-8601 timestamp of first creation. |
| `updated_at` | `DateTime<Utc>` | ISO-8601 timestamp of the most recent write. |
| `collections` | `Vec<CollectionIndex>` | One entry per collection (see below). |
| `total_size` | `u64` | Sum of uncompressed sizes of all entries. |
| `compressed_size` | `u64` | Sum of compressed sizes inside the ZIP. |
| `compression_ratio` | `f64` | `compressed_size / total_size`. |

Each `CollectionIndex` has `name`, `files: Vec<FileEntry>`, `vector_count`,
`dimension`, and a free-form `metadata: HashMap<String, String>`. Each
`FileEntry` has `path`, `size`, `compressed_size`, `checksum`, and `type`.
**`checksum` is the hex-encoded SHA-256 digest of the uncompressed entry
bytes** — not CRC32. A mismatch on load is reported as a clear error through
`VectorizerError`.

```json
{
  "version": "1.0",
  "created_at": "2024-10-14T12:00:00Z",
  "updated_at": "2024-10-14T13:00:00Z",
  "collections": [
    {
      "name": "my_collection",
      "files": [
        {
          "path": "my_collection_vector_store.bin",
          "size": 1048576,
          "compressed_size": 524288,
          "checksum": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
          "type": "vectors"
        }
      ],
      "vector_count": 1000,
      "dimension": 384,
      "metadata": {}
    }
  ],
  "total_size": 10485760,
  "compressed_size": 5242880,
  "compression_ratio": 0.50
}
```

## Configuration

Add to `config.yml`:

```yaml
storage:
  # Compression settings
  compression:
    enabled: true          # Enable .vecdb format
    format: "zstd"         # Compression algorithm
    level: 3               # Compression level (1-22)
  
  # Snapshot settings
  snapshots:
    enabled: true          # Enable automatic snapshots
    interval_hours: 1      # Snapshot frequency
    retention_days: 2      # How long to keep snapshots
    max_snapshots: 48      # Maximum snapshots to retain
    path: "./data/snapshots"
  
  # Compaction settings
  compaction:
    batch_size: 1000       # Operations before consolidation
    auto_compact: true     # Enable auto-compaction
```

## CLI Commands

### Storage Information

```bash
# Show storage statistics
vectorizer storage info

# Detailed statistics with per-collection breakdown
vectorizer storage info --detailed
```

### Migration

```bash
# Migrate from legacy to .vecdb format
vectorizer storage migrate

# Force migration with custom compression
vectorizer storage migrate --force --level 5
```

### Verification

```bash
# Verify storage integrity
vectorizer storage verify

# Verify and fix issues
vectorizer storage verify --fix
```

### Manual Compaction

```bash
# Compact storage (if batch threshold not reached)
vectorizer storage compact --force
```

### Snapshots

```bash
# List all snapshots
vectorizer snapshot list

# List with detailed information
vectorizer snapshot list --detailed

# Create manual snapshot
vectorizer snapshot create

# Create with description
vectorizer snapshot create --description "Before major update"

# Restore from snapshot
vectorizer snapshot restore --id 20241014_120000 --force

# Delete specific snapshot
vectorizer snapshot delete --id 20241014_120000

# Clean up old snapshots
vectorizer snapshot cleanup

# Dry run cleanup
vectorizer snapshot cleanup --dry-run
```

## Performance Characteristics

Based on benchmark results:

| Metric | Legacy | Compact | Improvement |
|--------|--------|---------|-------------|
| **Storage Size** | 100% | ~72% | 28% reduction |
| **Load Time (Small)** | 158ms | 55ms | 65% faster |
| **Load Time (Medium)** | 1.5s | 3.9s | 160% slower* |
| **Load Time (Large)** | 2.1s | 479s | Much slower* |
| **Save Time** | Varies | Varies | Depends on size |

\* **Note:** Large dataset load times show optimization opportunities. The current implementation decompresses all data upfront. Future versions will implement lazy loading to improve performance.

## Best Practices

### When to Use Compact Format

✅ **Use .vecdb format for:**
- Production deployments
- Backup/restore scenarios
- Datasets that change infrequently
- Cloud storage (reduced costs)
- Version control (smaller diffs)
- Disaster recovery scenarios

❌ **Stick with legacy format for:**
- Active development
- Frequently changing datasets
- Very large datasets (>1GB)* until lazy loading is implemented
- Applications requiring instant startup

### Optimization Tips

1. **Compression Level**
   - Level 1-3: Fast compression, moderate ratio
   - Level 3-9: Balanced (recommended)
   - Level 10-22: Maximum compression, slower

2. **Batch Size**
   - Small (100-500): More frequent compaction
   - Medium (1000): Balanced (default)
   - Large (5000+): Less frequent, larger batches

3. **Snapshots**
   - Hourly snapshots for critical data
   - Adjust retention based on storage capacity
   - Use cleanup regularly to manage disk space

## Atomic Updates

All write operations use atomic updates to prevent corruption:

1. Write to temporary file (`.vecdb.tmp`)
2. Verify integrity
3. Atomic rename to replace old file
4. Update index (`.vecidx`)

If the process is interrupted, the old `.vecdb` remains intact.

## Backup Safety

When backing up compact storage, treat the archive and its sidecar as a
single unit:

- **Copy both `<name>.vecdb` and `<name>.vecidx` atomically.** Snapshot the
  directory (e.g. filesystem snapshot, `rsync --link-dest`, tarball) instead
  of copying the two files with separate unrelated commands. Losing only the
  sidecar does not destroy data — the ZIP is self-describing and can still
  be read — but you lose the recorded SHA-256 checksums and original
  size/ratio metadata, so integrity verification on load is no longer possible.
- **SHA-256 validation runs on load.** If an entry's content hash in the
  archive does not match the digest recorded in `.vecidx`, Vectorizer logs a
  clear `VectorizerError` and refuses to treat the collection as healthy.
  Bit-rot and partial writes surface immediately rather than silently
  corrupting search results.
- **Never edit files inside the ZIP by hand.** Opening the archive, changing
  a JSON entry, and re-zipping will break the SHA-256 match and the next
  load will fail. All mutations must go through the normal API
  (`VectorStore` / REST / gRPC / MCP) so the index is rewritten with fresh
  checksums.
- Snapshots created by `vectorizer snapshot create` already contain matched
  `.vecdb` + `.vecidx` pairs; copying a snapshot directory is the safest way
  to ship a point-in-time backup.

## Migration Safety

The migration process:

1. ✅ Creates timestamped backup
2. ✅ Migrates all collections to .vecdb
3. ✅ Verifies integrity
4. ✅ Keeps legacy files for safety
5. ℹ️ User can manually delete legacy files after verification

**Rollback:** Legacy files remain available if needed.

## File Size Estimates

Typical compression ratios:

- **Vector data (.bin):** 40-60% reduction
- **Metadata (.json):** 70-80% reduction
- **Overall:** 20-30% reduction

Example: 100MB dataset → ~70MB .vecdb file

## Future Improvements

Planned enhancements:

1. **Lazy Loading** - Load collections on-demand
2. **Memory Mapping** - Direct access without full decompression
3. **Incremental Updates** - Append-only mode for dynamic collections
4. **Multi-threaded Compression** - Parallel compression for large datasets
5. **Zstd Dictionary** - Better compression for similar data

## Troubleshooting

### Migration Failed

```bash
# Check what went wrong
vectorizer storage info

# Verify legacy data is intact
ls -lh data/

# Try migration with lower compression
vectorizer storage migrate --level 1
```

### Snapshot Restore Issues

```bash
# List available snapshots
vectorizer snapshot list --detailed

# Verify snapshot integrity
vectorizer storage verify --fix

# Use older snapshot
vectorizer snapshot restore --id <older_snapshot> --force
```

### Performance Issues

```bash
# Check cache utilization
vectorizer storage info --detailed

# Compact manually
vectorizer storage compact --force

# Consider disabling compression for large active datasets
# Edit config.yml: storage.compression.enabled: false
```

## API Integration

The storage layer is transparent to the API. Collections work identically regardless of format:

```rust
// Create store - automatically detects format
let store = VectorStore::new();

// No code changes needed!
store.create_collection("my_collection", config)?;
store.insert("my_collection", vectors)?;
```

On startup, VectorStore automatically:
1. Detects storage format
2. Logs format information
3. Suggests migration if beneficial

## Security Considerations

- ✅ SHA-256 checksums for integrity verification
- ✅ Atomic writes prevent partial updates
- ✅ Snapshots enable point-in-time recovery
- ✅ Backups created before migrations
- ⚠️ No encryption at rest (use filesystem encryption if needed)

## Version Compatibility

- **Storage Version:** 1.0
- **Minimum Vectorizer:** v0.8.0
- **Format:** Forward and backward compatible
- **Migration:** One-way (legacy → compact)

---

For migration guide, see [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)

