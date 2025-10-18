# Storage Migration Guide

## Overview

This guide helps you migrate from the legacy file-based storage format to the new `.vecdb` compact format.

## Quick Start

```bash
# 1. Check current format
vectorizer storage info

# 2. Run migration
vectorizer storage migrate

# 3. Verify migration
vectorizer storage verify

# 4. Test your application
# (legacy files are kept as backup)

# 5. Clean up legacy files (optional, after verification)
# rm -rf data/collections  # Manual cleanup
```

## Pre-Migration Checklist

Before migrating, ensure:

- [ ] ✅ Vectorizer v0.8.0 or later installed
- [ ] ✅ Sufficient disk space (requires ~2x current data size temporarily)
- [ ] ✅ No active writes to the database
- [ ] ✅ Recent backup exists
- [ ] ✅ Read config.yml and understand storage settings

## Migration Steps

### Step 1: Check Current Status

```bash
vectorizer storage info
```

**Expected output:**
```
💾 Storage Information:
  Format: Legacy
  Using legacy file structure
  Run 'vectorizer storage migrate' to convert to .vecdb format
```

### Step 2: Review Configuration

Check `config.yml` for storage settings:

```yaml
storage:
  compression:
    enabled: true          # Must be true for .vecdb
    level: 3               # Adjust based on CPU/space tradeoff
  
  snapshots:
    enabled: true          # Recommended for production
    interval_hours: 1      # Adjust as needed
    retention_days: 2
```

### Step 3: Run Migration

```bash
# Standard migration (compression level 3)
vectorizer storage migrate

# Custom compression (1=fast, 22=max compression)
vectorizer storage migrate --level 5
```

**During migration:**
```
🔄 Starting storage migration to .vecdb format...
📦 Backup created: ./data/backup_before_migration_20241014_120000
🗜️  Starting compaction of directory: ./data/collections
✅ Compaction complete:
   Collections: 5
   Total vectors: 50000
   Original size: 150 MB
   Compressed size: 105 MB
   Compression ratio: 70.00%
🔍 Verifying archive integrity...
✅ Archive integrity verified
✅ Migration completed successfully!
   Migrated 5 collections
   Total vectors: 50000
   Space saved: 45 MB
ℹ️  Legacy files kept in: ./data/backup_before_migration_20241014_120000
   You can safely delete them after verifying the migration
```

### Step 4: Verify Migration

```bash
# Basic verification
vectorizer storage verify

# Verify and fix any issues
vectorizer storage verify --fix
```

### Step 5: Test Application

Start your application and verify:

```bash
# Start vectorizer
cargo run --release

# Test basic operations
curl http://localhost:15002/api/v1/collections

# Run your test suite
cargo test --release
```

### Step 6: Monitor First Run

Watch logs for:

```
✅ Using .vecdb compact storage format
```

If you see warnings or errors, check [Troubleshooting](#troubleshooting) section.

### Step 7: Clean Up (Optional)

After 24-48 hours of successful operation:

```bash
# Delete legacy backup
rm -rf data/backup_before_migration_*

# Or keep it archived
tar -czf vectorizer_legacy_backup.tar.gz data/backup_before_migration_*
rm -rf data/backup_before_migration_*
```

## Configuration Guide

### Compression Settings

| Level | Speed | Ratio | Use Case |
|-------|-------|-------|----------|
| 1 | Fastest | ~60% | Development, frequent changes |
| 3 | Fast | ~70% | **Recommended for most cases** |
| 5 | Moderate | ~75% | Production, balanced |
| 9 | Slow | ~80% | Archival, infrequent access |
| 15+ | Very slow | ~85% | Cold storage only |

### Snapshot Configuration

**Recommended settings:**

**Development:**
```yaml
snapshots:
  enabled: false  # Optional for dev
  interval_hours: 6
  retention_days: 1
```

**Production:**
```yaml
snapshots:
  enabled: true
  interval_hours: 1    # Hourly snapshots
  retention_days: 2    # Keep 48 snapshots
  max_snapshots: 48
```

**High-Value Data:**
```yaml
snapshots:
  enabled: true
  interval_hours: 0.5  # Every 30 minutes
  retention_days: 7    # One week
  max_snapshots: 336
```

### Batch Compaction

```yaml
compaction:
  batch_size: 1000       # Default
  auto_compact: true
```

- **Small (100-500):** More frequent compaction, lower memory
- **Medium (1000):** Balanced - recommended
- **Large (5000+):** Better performance, higher memory use

## Rollback Procedure

If you need to rollback to legacy format:

### Option 1: Use Backup

```bash
# Stop vectorizer
killall vectorizer

# Remove .vecdb files
rm data/vectorizer.vecdb data/vectorizer.vecidx

# Restore from backup
cp -r data/backup_before_migration_*/data/* data/

# Restart vectorizer
cargo run --release
```

### Option 2: Use Snapshot (if using compact format)

```bash
# List snapshots
vectorizer snapshot list

# Restore to earlier state
vectorizer snapshot restore --id 20241014_120000 --force

# Restart vectorizer
```

## Migrating Large Datasets

For datasets over 1GB:

1. **Increase Timeout:**
   ```yaml
   storage:
     migration_timeout_secs: 3600  # 1 hour
   ```

2. **Lower Compression:**
   ```bash
   vectorizer storage migrate --level 1  # Faster
   ```

3. **Monitor Progress:**
   ```bash
   # In another terminal
   watch -n 5 "ls -lh data/"
   ```

4. **Consider Partial Migration:**
   - Migrate one collection at a time (manual process)
   - Keep frequently-accessed collections in legacy format

## Disk Space Requirements

During migration, you need:

- **Original data:** X MB
- **Compressed archive:** ~0.7X MB
- **Backup:** X MB
- **Working space:** ~0.3X MB

**Total:** ~2.0X MB temporary space

After cleanup: ~0.7X MB (30% savings)

## Performance Tuning

### For Fast Startup

```yaml
storage:
  compression:
    enabled: true
    level: 1           # Fastest decompression
```

### For Maximum Compression

```yaml
storage:
  compression:
    enabled: true
    level: 9           # Better compression
```

### For Active Databases

```yaml
storage:
  compression:
    enabled: false     # Use legacy format
```

## Troubleshooting

### Migration Takes Too Long

**Symptoms:** Migration running for >30 minutes

**Solutions:**
1. Lower compression level: `--level 1`
2. Check disk I/O: `iostat -x 1`
3. Ensure sufficient RAM
4. Close other applications

### Out of Disk Space

**Symptoms:** Migration fails with "No space left on device"

**Solutions:**
1. Free up space (need ~2x current data size)
2. Use external storage temporarily
3. Migrate to different volume with more space

### Corrupted Archive

**Symptoms:** `storage verify` fails

**Solutions:**
```bash
# Restore from backup
cp -r data/backup_before_migration_*/data/* data/

# Or rebuild archive
rm data/vectorizer.vecdb data/vectorizer.vecidx
vectorizer storage migrate
```

### Snapshot Failed

**Symptoms:** Cannot create snapshots

**Solutions:**
```bash
# Check permissions
ls -la data/snapshots/

# Create directory manually
mkdir -p data/snapshots
chmod 755 data/snapshots

# Try again
vectorizer snapshot create
```

### Slow Performance After Migration

**Symptoms:** Queries take longer after migration

**Current Limitation:**
- Large dataset decompression is not yet optimized
- Lazy loading will be implemented in future version

**Workarounds:**
1. Use snapshot for point-in-time loading
2. Disable compression for active collections
3. Keep hot collections in legacy format
4. Wait for lazy loading feature

**Revert if needed:**
```bash
# Disable compression temporarily
# Edit config.yml:
storage:
  compression:
    enabled: false

# Or rollback to legacy
rm data/vectorizer.vecdb data/vectorizer.vecidx
cp -r data/backup_before_migration_*/data/* data/
```

## Migration Scenarios

### Scenario 1: Small Database (<100MB)

**Recommended approach:**
- Direct migration with default settings
- Enable snapshots
- Monitor first 24 hours

```bash
vectorizer storage migrate
vectorizer snapshot create
```

### Scenario 2: Medium Database (100MB-1GB)

**Recommended approach:**
- Migrate during low-traffic period
- Test extensively before cleanup
- Keep backup for 1 week

```bash
# Off-peak hours
vectorizer storage migrate --level 5

# Test thoroughly
cargo test --release

# Monitor for a week before cleanup
```

### Scenario 3: Large Database (>1GB)

**Recommended approach:**
- Test on copy first
- Consider partial migration
- Evaluate performance impact
- Wait for lazy loading feature

```bash
# Test on copy
cp -r data data_test
VECTORIZER_DATA_DIR=./data_test vectorizer storage migrate

# Measure performance
vectorizer storage info --detailed

# Decide: migrate or wait
```

## Automated Migration

To enable automatic migration on startup:

```yaml
storage:
  compression:
    enabled: true
  auto_migrate: true     # Migrate on first startup
```

**Note:** Currently shows migration suggestion only. Full auto-migration will be implemented in future version.

## Backup Strategy

### Before Migration

```bash
# Full backup
tar -czf vectorizer_backup_$(date +%Y%m%d).tar.gz data/
```

### After Migration

```bash
# Backup .vecdb only (smaller)
cp data/vectorizer.vecdb backups/vectorizer_$(date +%Y%m%d).vecdb
cp data/vectorizer.vecidx backups/vectorizer_$(date +%Y%m%d).vecidx
```

### Automated Snapshots

Snapshots run automatically based on configuration. No manual intervention needed.

## Monitoring

### Check Snapshot Status

```bash
# List recent snapshots
vectorizer snapshot list | head -20

# Check disk usage
du -sh data/snapshots/

# Monitor growth
watch -n 60 "du -sh data/snapshots/"
```

### Check Compression Efficiency

```bash
vectorizer storage info
```

Look for:
- `Compression ratio`: Should be 0.60-0.80
- `Space saved`: Should be 20-40%

## FAQ

### Q: Can I use both formats simultaneously?

**A:** No, each Vectorizer instance uses one format. However, different collections can theoretically use different formats in future versions.

### Q: What happens if migration is interrupted?

**A:** The original data remains intact. Delete temporary files and try again:
```bash
rm data/vectorizer.vecdb.tmp data/vectorizer.vecidx.tmp
vectorizer storage migrate
```

### Q: Can I migrate back to legacy format?

**A:** Yes, use the backup created during migration, or extract files from .vecdb manually.

### Q: How much faster are snapshots vs full backups?

**A:** Snapshots are 10-100x faster (copy vs full backup). For 1GB data:
- Full backup: ~60 seconds
- Snapshot: ~1 second

### Q: Is .vecdb format portable?

**A:** Yes! Copy `.vecdb` and `.vecidx` files to another machine and they work immediately.

### Q: What if I need to access individual files?

**A:** `.vecdb` is a standard ZIP archive. You can extract files:
```bash
unzip -l data/vectorizer.vecdb  # List contents
unzip data/vectorizer.vecdb "data/my_collection/*" -d extracted/
```

---

For technical details, see [STORAGE.md](./STORAGE.md)

