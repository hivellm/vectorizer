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

## Index Format (.vecidx)

The `.vecidx` file contains metadata about the archive:

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
          "path": "data/my_collection/vectors.bin",
          "size": 1048576,
          "compressed_size": 524288,
          "checksum": "sha256_hash",
          "type": "vectors"
        }
      ],
      "vector_count": 1000,
      "dimension": 384
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

