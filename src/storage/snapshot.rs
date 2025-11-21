//! Snapshot management for .vecdb archives

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{Result, VectorizerError};
use crate::storage::StorageIndex;

/// Snapshot manager for creating and managing backups
pub struct SnapshotManager {
    /// Data directory containing .vecdb file
    data_dir: PathBuf,

    /// Snapshots directory
    snapshots_dir: PathBuf,

    /// Maximum number of snapshots to keep
    max_snapshots: usize,

    /// Retention period in hours
    retention_hours: i64,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(
        data_dir: impl AsRef<Path>,
        snapshots_path: impl AsRef<Path>,
        max_snapshots: usize,
        retention_hours: u64,
    ) -> Self {
        let snapshots_dir = snapshots_path.as_ref().to_path_buf();

        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            snapshots_dir,
            max_snapshots,
            retention_hours: retention_hours as i64,
        }
    }

    /// Create a new snapshot
    pub fn create_snapshot(&self) -> Result<SnapshotInfo> {
        // Ensure data directory exists first (parent of snapshots)
        if let Some(parent) = self.snapshots_dir.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                VectorizerError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create parent directory {:?}: {} (kind: {:?})",
                        parent,
                        e,
                        e.kind()
                    ),
                ))
            })?;
        }

        // Ensure snapshots directory exists
        fs::create_dir_all(&self.snapshots_dir).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to create snapshots directory {:?}: {} (kind: {:?})",
                    self.snapshots_dir,
                    e,
                    e.kind()
                ),
            ))
        })?;

        let timestamp = Utc::now();
        let snapshot_id = timestamp.format("%Y%m%d_%H%M%S").to_string();
        let snapshot_dir = self.snapshots_dir.join(&snapshot_id);

        info!(
            "📸 Creating snapshot: {} in {:?}",
            snapshot_id, snapshot_dir
        );

        fs::create_dir_all(&snapshot_dir).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to create snapshot directory {:?}: {} (kind: {:?})",
                    snapshot_dir,
                    e,
                    e.kind()
                ),
            ))
        })?;

        // Copy .vecdb file
        let vecdb_src = self.data_dir.join(crate::storage::VECDB_FILE);
        let vecdb_dst = snapshot_dir.join(crate::storage::VECDB_FILE);

        if vecdb_src.exists() {
            // Check source file size and permissions
            let src_metadata = fs::metadata(&vecdb_src).map_err(|e| {
                VectorizerError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to read source file metadata {:?}: {} (kind: {:?})",
                        vecdb_src,
                        e,
                        e.kind()
                    ),
                ))
            })?;

            // Copy with better error handling
            fs::copy(&vecdb_src, &vecdb_dst).map_err(|e| {
                VectorizerError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to copy {:?} to {:?}: {} (kind: {:?}, src_size: {} bytes)",
                        vecdb_src,
                        vecdb_dst,
                        e,
                        e.kind(),
                        src_metadata.len()
                    ),
                ))
            })?;
        } else {
            return Err(VectorizerError::Storage(format!(
                "No .vecdb file to snapshot at {:?}",
                vecdb_src
            )));
        }

        // Copy .vecidx file
        let vecidx_src = self.data_dir.join(crate::storage::VECIDX_FILE);
        let vecidx_dst = snapshot_dir.join(crate::storage::VECIDX_FILE);

        if vecidx_src.exists() {
            fs::copy(&vecidx_src, &vecidx_dst).map_err(|e| {
                VectorizerError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to copy {:?} to {:?}: {} (kind: {:?})",
                        vecidx_src,
                        vecidx_dst,
                        e,
                        e.kind()
                    ),
                ))
            })?;
        }

        // Get file size
        let vecdb_size = fs::metadata(&vecdb_dst).map(|m| m.len()).unwrap_or(0);

        // Create snapshot metadata
        let snapshot = SnapshotInfo {
            id: snapshot_id.clone(),
            created_at: timestamp,
            size_bytes: vecdb_size,
            path: snapshot_dir,
            index_version: crate::storage::STORAGE_VERSION.to_string(),
        };

        // Save metadata
        self.save_snapshot_metadata(&snapshot).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to save snapshot metadata for {:?}: {}",
                    snapshot.path, e
                ),
            ))
        })?;

        info!(
            "✅ Snapshot created: {} ({} MB)",
            snapshot.id,
            snapshot.size_bytes / 1_048_576
        );

        // Clean up old snapshots (non-critical, log but don't fail)
        if let Err(e) = self.cleanup_old_snapshots() {
            warn!("⚠️  Snapshot: Failed to cleanup old snapshots: {}", e);
        }

        Ok(snapshot)
    }

    /// List all available snapshots
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        if !self.snapshots_dir.exists() {
            return Ok(Vec::new());
        }

        // Verify it's actually a directory
        let metadata = fs::metadata(&self.snapshots_dir).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Cannot access snapshots directory {:?}: {}",
                    self.snapshots_dir, e
                ),
            ))
        })?;

        if !metadata.is_dir() {
            return Err(VectorizerError::Storage(format!(
                "Snapshots path {:?} is not a directory",
                self.snapshots_dir
            )));
        }

        let mut snapshots = Vec::new();

        let read_dir = fs::read_dir(&self.snapshots_dir).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Cannot read snapshots directory {:?}: {}",
                    self.snapshots_dir, e
                ),
            ))
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                if let Ok(snapshot) = self.load_snapshot_metadata(&path) {
                    snapshots.push(snapshot);
                }
            }
        }

        // Sort by creation time (newest first)
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(snapshots)
    }

    /// Get a specific snapshot by ID
    pub fn get_snapshot(&self, id: &str) -> Result<Option<SnapshotInfo>> {
        let snapshots = self.list_snapshots()?;
        Ok(snapshots.into_iter().find(|s| s.id == id))
    }

    /// Restore from a snapshot
    pub fn restore_snapshot(&self, id: &str) -> Result<()> {
        let snapshot = self
            .get_snapshot(id)?
            .ok_or_else(|| VectorizerError::Storage(format!("Snapshot not found: {}", id)))?;

        info!("🔄 Restoring from snapshot: {}", id);

        // Copy snapshot files to data directory
        let vecdb_src = snapshot.path.join(crate::storage::VECDB_FILE);
        let vecdb_dst = self.data_dir.join(crate::storage::VECDB_FILE);

        if vecdb_src.exists() {
            fs::copy(&vecdb_src, &vecdb_dst).map_err(|e| VectorizerError::Io(e))?;
        } else {
            return Err(VectorizerError::Storage(
                "Snapshot .vecdb file not found".to_string(),
            ));
        }

        let vecidx_src = snapshot.path.join(crate::storage::VECIDX_FILE);
        let vecidx_dst = self.data_dir.join(crate::storage::VECIDX_FILE);

        if vecidx_src.exists() {
            fs::copy(&vecidx_src, &vecidx_dst).map_err(|e| VectorizerError::Io(e))?;
        }

        info!("✅ Snapshot restored successfully");
        Ok(())
    }

    /// Delete a specific snapshot
    pub fn delete_snapshot(&self, id: &str) -> Result<bool> {
        let snapshot = match self.get_snapshot(id)? {
            Some(s) => s,
            None => return Ok(false),
        };

        info!("🗑️  Deleting snapshot: {}", id);

        fs::remove_dir_all(&snapshot.path).map_err(|e| VectorizerError::Io(e))?;

        Ok(true)
    }

    /// Clean up old snapshots based on retention policy
    /// Simple approach: parse date from directory name (format: %Y%m%d_%H%M%S) and remove if older than retention period
    pub fn cleanup_old_snapshots(&self) -> Result<usize> {
        // Check if snapshots directory exists
        if !self.snapshots_dir.exists() {
            debug!(
                "🧹 No snapshots directory found at {:?} - nothing to clean up",
                self.snapshots_dir
            );
            return Ok(0);
        }

        // Ensure the directory is actually a directory and readable
        let metadata = match fs::metadata(&self.snapshots_dir) {
            Ok(m) => m,
            Err(e) => {
                warn!(
                    "⚠️  Cannot access snapshots directory {:?}: {} - skipping cleanup",
                    self.snapshots_dir, e
                );
                return Ok(0);
            }
        };

        if !metadata.is_dir() {
            warn!(
                "⚠️  Snapshots path {:?} is not a directory - skipping cleanup",
                self.snapshots_dir
            );
            return Ok(0);
        }

        let mut deleted = 0;
        let cutoff_date = Utc::now() - Duration::hours(self.retention_hours);

        info!(
            "🧹 Cleaning up snapshots older than {} hours (cutoff: {})",
            self.retention_hours, cutoff_date
        );

        // Read all directories in snapshots folder
        let read_dir = match fs::read_dir(&self.snapshots_dir) {
            Ok(dir) => dir,
            Err(e) => {
                warn!(
                    "⚠️  Cannot read snapshots directory {:?}: {} - skipping cleanup",
                    self.snapshots_dir, e
                );
                return Ok(0);
            }
        };

        // Collect all snapshot directories with their parsed dates
        let mut snapshots: Vec<(PathBuf, DateTime<Utc>, String)> = Vec::new();

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("⚠️  Error reading snapshot directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Extract snapshot ID from directory name (format: %Y%m%d_%H%M%S)
            let snapshot_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(id) => id.to_string(),
                None => {
                    warn!("⚠️  Invalid snapshot directory name: {:?}", path);
                    continue;
                }
            };

            // Parse date from snapshot ID (format: %Y%m%d_%H%M%S)
            let created_at = match NaiveDateTime::parse_from_str(&snapshot_id, "%Y%m%d_%H%M%S") {
                Ok(naive_dt) => DateTime::<Utc>::from_utc(naive_dt, Utc),
                Err(_) => {
                    // If parsing fails, try to use directory modification time as fallback
                    match fs::metadata(&path).and_then(|m| m.modified()) {
                        Ok(modified) => {
                            let duration = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default();
                            DateTime::<Utc>::from_timestamp(
                                duration.as_secs() as i64,
                                duration.subsec_nanos(),
                            )
                            .unwrap_or_else(|| Utc::now())
                        }
                        Err(_) => {
                            warn!(
                                "⚠️  Cannot determine creation date for snapshot {:?} - skipping",
                                path
                            );
                            continue;
                        }
                    }
                }
            };

            snapshots.push((path, created_at, snapshot_id));
        }

        // Sort by creation time (newest first)
        snapshots.sort_by(|a, b| b.1.cmp(&a.1));

        // Delete snapshots older than retention period
        for (path, created_at, snapshot_id) in &snapshots {
            if *created_at < cutoff_date {
                let age_hours = (Utc::now() - *created_at).num_hours();
                info!(
                    "🗑️  Deleting snapshot {} (age: {} hours)",
                    snapshot_id, age_hours
                );

                match fs::remove_dir_all(path) {
                    Ok(_) => {
                        deleted += 1;
                        debug!("✅ Removed snapshot directory: {:?}", path);
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to remove snapshot directory {:?}: {}", path, e);
                    }
                }
            }
        }

        // Enforce max_snapshots limit (keep only the most recent N snapshots)
        // Re-read directory to get remaining snapshots after deletion
        let remaining_snapshots: Vec<_> = snapshots
            .into_iter()
            .filter(|(path, _, _)| path.exists())
            .collect();

        if remaining_snapshots.len() > self.max_snapshots {
            let to_delete = remaining_snapshots.len() - self.max_snapshots;
            info!(
                "🗑️  Enforcing max_snapshots limit: {} snapshots, keeping {} newest",
                remaining_snapshots.len(),
                self.max_snapshots
            );

            for (path, _, snapshot_id) in remaining_snapshots.iter().skip(self.max_snapshots) {
                info!(
                    "🗑️  Deleting snapshot {} (exceeds max_snapshots limit)",
                    snapshot_id
                );
                match fs::remove_dir_all(path) {
                    Ok(_) => {
                        deleted += 1;
                        debug!("✅ Removed snapshot directory: {:?}", path);
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to remove snapshot directory {:?}: {}", path, e);
                    }
                }
            }
        }

        if deleted > 0 {
            info!(
                "✅ Cleaned up {} old snapshots (retention: {} hours, max: {})",
                deleted, self.retention_hours, self.max_snapshots
            );
        } else {
            info!("✅ No snapshots to clean up (all snapshots within retention period)");
        }

        Ok(deleted)
    }

    /// Clean up empty directories in the snapshots folder
    fn cleanup_empty_directories(&self) -> Result<usize> {
        if !self.snapshots_dir.exists() {
            return Ok(0);
        }

        let mut removed = 0;

        // Read all entries in snapshots directory
        let mut entries = Vec::new();
        for entry_result in fs::read_dir(&self.snapshots_dir).map_err(|e| VectorizerError::Io(e))? {
            entries.push(entry_result.map_err(|e| VectorizerError::Io(e))?);
        }

        for entry in entries {
            let path = entry.path();

            // Only check directories
            if !path.is_dir() {
                continue;
            }

            // Check if directory is empty or only contains metadata files
            let is_empty = match fs::read_dir(&path) {
                Ok(dir) => {
                    // Check if directory has any data files (.vecdb or .vecidx)
                    let mut has_data_files = false;
                    for entry_result in dir {
                        if let Ok(entry) = entry_result {
                            let file_path = entry.path();
                            if file_path.is_file() {
                                // Check if it's a real data file, not just metadata
                                let file_name =
                                    file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                // Real data files: .vecdb, .vecidx
                                if file_name == crate::storage::VECDB_FILE
                                    || file_name == crate::storage::VECIDX_FILE
                                {
                                    has_data_files = true;
                                    break;
                                }
                            }
                        }
                    }
                    !has_data_files
                }
                Err(_) => false, // Can't read, skip
            };

            if is_empty {
                // Try to remove empty directory
                match fs::remove_dir(&path) {
                    Ok(_) => {
                        debug!("🗑️  Removed empty snapshot directory: {:?}", path);
                        removed += 1;
                    }
                    Err(e) => {
                        // If removal fails, try remove_dir_all in case there are hidden files
                        if let Err(e2) = fs::remove_dir_all(&path) {
                            warn!(
                                "⚠️  Failed to remove empty directory {:?}: {} (remove_dir_all: {})",
                                path, e, e2
                            );
                        } else {
                            debug!(
                                "🗑️  Removed empty snapshot directory (with hidden files): {:?}",
                                path
                            );
                            removed += 1;
                        }
                    }
                }
            }
        }

        if removed > 0 {
            info!("🧹 Removed {} empty snapshot directories", removed);
        }

        Ok(removed)
    }

    /// Save snapshot metadata
    fn save_snapshot_metadata(&self, snapshot: &SnapshotInfo) -> Result<()> {
        let metadata_path = snapshot.path.join("snapshot.json");
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;

        fs::write(&metadata_path, json).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to write snapshot metadata to {:?}: {} (kind: {:?})",
                    metadata_path,
                    e,
                    e.kind()
                ),
            ))
        })?;

        Ok(())
    }

    /// Load snapshot metadata
    fn load_snapshot_metadata(&self, snapshot_dir: &Path) -> Result<SnapshotInfo> {
        let metadata_path = snapshot_dir.join("snapshot.json");

        if !metadata_path.exists() {
            // Create metadata from directory if missing
            // Use directory modification time as creation date (more accurate for old snapshots)
            let id = snapshot_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| VectorizerError::Storage("Invalid snapshot directory".to_string()))?
                .to_string();

            let vecdb_path = snapshot_dir.join(crate::storage::VECDB_FILE);
            let size_bytes = fs::metadata(&vecdb_path).map(|m| m.len()).unwrap_or(0);

            // Use directory modification time as creation date (fallback for old snapshots without metadata)
            let created_at = fs::metadata(snapshot_dir)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|system_time| {
                    // Convert SystemTime to DateTime<Utc>
                    let duration = system_time.duration_since(std::time::UNIX_EPOCH).ok()?;
                    DateTime::<Utc>::from_timestamp(
                        duration.as_secs() as i64,
                        duration.subsec_nanos(),
                    )
                })
                .unwrap_or_else(|| Utc::now());

            return Ok(SnapshotInfo {
                id,
                created_at,
                size_bytes,
                path: snapshot_dir.to_path_buf(),
                index_version: crate::storage::STORAGE_VERSION.to_string(),
            });
        }

        let content = fs::read_to_string(&metadata_path).map_err(|e| VectorizerError::Io(e))?;

        let snapshot: SnapshotInfo = serde_json::from_str(&content)
            .map_err(|e| VectorizerError::Deserialization(e.to_string()))?;

        Ok(snapshot)
    }
}

/// Information about a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Unique snapshot ID (timestamp-based)
    pub id: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Size in bytes
    pub size_bytes: u64,

    /// Path to snapshot directory
    #[serde(skip)]
    pub path: PathBuf,

    /// Storage index version
    pub index_version: String,
}

impl SnapshotInfo {
    /// Get human-readable size
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / 1_048_576.0
    }

    /// Get age in hours
    pub fn age_hours(&self) -> i64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);
        duration.num_hours()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;
    use crate::storage::StorageWriter;

    fn create_test_vecdb(data_dir: &Path) {
        let collections_dir = data_dir.join("collections");
        let collection_dir = collections_dir.join("test");
        fs::create_dir_all(&collection_dir).unwrap();

        let test_file = collection_dir.join("test.bin");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"test data").unwrap();

        let writer = StorageWriter::new(data_dir, 3);
        writer.write_archive(&collections_dir).unwrap();
    }

    #[test]
    fn test_snapshot_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let snapshots_dir = temp_dir.path().join("snapshots");

        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 48);
        assert_eq!(manager.max_snapshots, 48);
        assert_eq!(manager.retention_hours, 48);
    }

    #[test]
    fn test_create_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        create_test_vecdb(temp_dir.path());

        let snapshots_dir = temp_dir.path().join("snapshots");
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 48);

        let result = manager.create_snapshot();
        assert!(result.is_ok());

        let snapshot = result.unwrap();
        assert!(!snapshot.id.is_empty());
        assert!(snapshot.size_bytes > 0);
    }

    #[test]
    fn test_list_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        create_test_vecdb(temp_dir.path());

        let snapshots_dir = temp_dir.path().join("snapshots");
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 48);

        manager.create_snapshot().unwrap();

        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 1);
    }

    #[test]
    #[ignore = "Integration test - requires full archive setup"]
    fn test_restore_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        create_test_vecdb(temp_dir.path());

        let snapshots_dir = temp_dir.path().join("snapshots");
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 48);

        let snapshot = manager.create_snapshot().unwrap();
        let result = manager.restore_snapshot(&snapshot.id);

        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Integration test - requires full archive setup"]
    fn test_delete_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        create_test_vecdb(temp_dir.path());

        let snapshots_dir = temp_dir.path().join("snapshots");
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 48);

        let snapshot = manager.create_snapshot().unwrap();
        assert!(manager.delete_snapshot(&snapshot.id).unwrap());

        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 0);
    }
}
