//! Snapshot management for .vecdb archives

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

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

    /// Retention period in days
    retention_days: i64,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(
        data_dir: impl AsRef<Path>,
        snapshots_path: impl AsRef<Path>,
        max_snapshots: usize,
        retention_days: u64,
    ) -> Self {
        let snapshots_dir = snapshots_path.as_ref().to_path_buf();

        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            snapshots_dir,
            max_snapshots,
            retention_days: retention_days as i64,
        }
    }

    /// Create a new snapshot
    pub fn create_snapshot(&self) -> Result<SnapshotInfo> {
        // Ensure data directory exists first (parent of snapshots)
        if let Some(parent) = self.snapshots_dir.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                VectorizerError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to create parent directory {:?}: {}", parent, e),
                ))
            })?;
        }

        // Ensure snapshots directory exists
        fs::create_dir_all(&self.snapshots_dir).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Failed to create snapshots directory {:?}: {}",
                    self.snapshots_dir, e
                ),
            ))
        })?;

        let timestamp = Utc::now();
        let snapshot_id = timestamp.format("%Y%m%d_%H%M%S").to_string();
        let snapshot_dir = self.snapshots_dir.join(&snapshot_id);

        info!("📸 Creating snapshot: {}", snapshot_id);

        fs::create_dir_all(&snapshot_dir).map_err(|e| {
            VectorizerError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Failed to create snapshot directory {:?}: {}",
                    snapshot_dir, e
                ),
            ))
        })?;

        // Copy .vecdb file
        let vecdb_src = self.data_dir.join(crate::storage::VECDB_FILE);
        let vecdb_dst = snapshot_dir.join(crate::storage::VECDB_FILE);

        if vecdb_src.exists() {
            fs::copy(&vecdb_src, &vecdb_dst).map_err(|e| VectorizerError::Io(e))?;
        } else {
            return Err(VectorizerError::Storage(
                "No .vecdb file to snapshot".to_string(),
            ));
        }

        // Copy .vecidx file
        let vecidx_src = self.data_dir.join(crate::storage::VECIDX_FILE);
        let vecidx_dst = snapshot_dir.join(crate::storage::VECIDX_FILE);

        if vecidx_src.exists() {
            fs::copy(&vecidx_src, &vecidx_dst).map_err(|e| VectorizerError::Io(e))?;
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
        self.save_snapshot_metadata(&snapshot)?;

        info!(
            "✅ Snapshot created: {} ({} MB)",
            snapshot.id,
            snapshot.size_bytes / 1_048_576
        );

        // Clean up old snapshots
        self.cleanup_old_snapshots()?;

        Ok(snapshot)
    }

    /// List all available snapshots
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        if !self.snapshots_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in fs::read_dir(&self.snapshots_dir).map_err(|e| VectorizerError::Io(e))? {
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
    pub fn cleanup_old_snapshots(&self) -> Result<usize> {
        let snapshots = self.list_snapshots()?;
        let mut deleted = 0;

        let cutoff_date = Utc::now() - Duration::days(self.retention_days);

        // Delete snapshots older than retention period
        for snapshot in &snapshots {
            if snapshot.created_at < cutoff_date {
                if self.delete_snapshot(&snapshot.id)? {
                    deleted += 1;
                }
            }
        }

        // Enforce max_snapshots limit
        let remaining_snapshots = self.list_snapshots()?;
        if remaining_snapshots.len() > self.max_snapshots {
            let to_delete = remaining_snapshots.len() - self.max_snapshots;

            for snapshot in remaining_snapshots.iter().skip(self.max_snapshots) {
                if self.delete_snapshot(&snapshot.id)? {
                    deleted += 1;
                }
                if deleted >= to_delete {
                    break;
                }
            }
        }

        if deleted > 0 {
            info!("🧹 Cleaned up {} old snapshots", deleted);
        }

        Ok(deleted)
    }

    /// Save snapshot metadata
    fn save_snapshot_metadata(&self, snapshot: &SnapshotInfo) -> Result<()> {
        let metadata_path = snapshot.path.join("snapshot.json");
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;

        fs::write(metadata_path, json).map_err(|e| VectorizerError::Io(e))?;

        Ok(())
    }

    /// Load snapshot metadata
    fn load_snapshot_metadata(&self, snapshot_dir: &Path) -> Result<SnapshotInfo> {
        let metadata_path = snapshot_dir.join("snapshot.json");

        if !metadata_path.exists() {
            // Create metadata from directory if missing
            let id = snapshot_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| VectorizerError::Storage("Invalid snapshot directory".to_string()))?
                .to_string();

            let vecdb_path = snapshot_dir.join(crate::storage::VECDB_FILE);
            let size_bytes = fs::metadata(&vecdb_path).map(|m| m.len()).unwrap_or(0);

            return Ok(SnapshotInfo {
                id,
                created_at: Utc::now(),
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

        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 2);
        assert_eq!(manager.max_snapshots, 48);
        assert_eq!(manager.retention_days, 2);
    }

    #[test]
    fn test_create_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        create_test_vecdb(temp_dir.path());

        let snapshots_dir = temp_dir.path().join("snapshots");
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 2);

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
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 2);

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
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 2);

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
        let manager = SnapshotManager::new(temp_dir.path(), snapshots_dir, 48, 2);

        let snapshot = manager.create_snapshot().unwrap();
        assert!(manager.delete_snapshot(&snapshot.id).unwrap());

        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 0);
    }
}
