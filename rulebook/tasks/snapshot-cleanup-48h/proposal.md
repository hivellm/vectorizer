# Snapshot Cleanup 48h Proposal

## Why

Snapshots are accumulating too much disk space. The current system maintains 7 days of snapshots (168 snapshots with 1 hour interval), which can occupy a lot of space. We need to reduce retention to 48 hours (2 days) to save disk space while maintaining sufficient history for recovery.

## What Changes

- **MODIFIED**: Default snapshot retention configuration from 7 days to 2 days (48 hours)
- **MODIFIED**: AutoSaveManager to use 48 hours retention instead of 7 days
- **MODIFIED**: SnapshotConfig default to retention_days: 2 and max_snapshots: 48 (24 snapshots/day × 2 days)
- **MODIFIED**: Automatic snapshot cleanup to keep only the last 48 hours

## Impact

- Affected specs: `storage`, `snapshot`
- Affected code: `src/storage/config.rs`, `src/storage/snapshot.rs`, `src/db/auto_save.rs`
- Breaking changes: No (default behavior change, but can be configured)
- User benefit: Disk space savings, more efficient automatic cleanup

