//! Storage module for vectorizer database
//!
//! This module provides a unified storage format (.vecdb/.vecidx) with compression,
//! snapshots, and automatic migration from legacy file structures.

pub mod compact;
pub mod config;
pub mod index;
pub mod migration;
pub mod reader;
pub mod snapshot;
pub mod writer;

use std::path::{Path, PathBuf};

pub use compact::StorageCompactor;
pub use config::StorageConfig;
pub use index::{CollectionIndex, FileEntry, StorageIndex};
pub use migration::StorageMigrator;
pub use reader::StorageReader;
pub use snapshot::{SnapshotInfo, SnapshotManager};
pub use writer::StorageWriter;

use crate::error::{Result, VectorizerError};

/// Storage format version
pub const STORAGE_VERSION: &str = "1.0";

/// Default .vecdb file name
pub const VECDB_FILE: &str = "vectorizer.vecdb";

/// Default .vecidx file name
pub const VECIDX_FILE: &str = "vectorizer.vecidx";

/// Temporary file suffix for atomic writes
pub const TEMP_SUFFIX: &str = ".tmp";

/// Snapshot directory name
pub const SNAPSHOT_DIR: &str = "snapshots";

/// Storage format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageFormat {
    /// Legacy format (individual files in data/ directory)
    Legacy,
    /// Compact format (.vecdb archive)
    Compact,
}

/// Detect storage format in the given directory
pub fn detect_format(data_dir: &Path) -> StorageFormat {
    let vecdb_path = data_dir.join(VECDB_FILE);
    if vecdb_path.exists() {
        StorageFormat::Compact
    } else {
        // Check if legacy format exists (files with _vector_store.bin pattern)
        if let Ok(entries) = std::fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if name_str.ends_with("_vector_store.bin") {
                        return StorageFormat::Legacy;
                    }
                }
            }
        }
        StorageFormat::Legacy
    }
}

/// Get the path to .vecdb file
pub fn vecdb_path(data_dir: &Path) -> PathBuf {
    data_dir.join(VECDB_FILE)
}

/// Get the path to .vecidx file
pub fn vecidx_path(data_dir: &Path) -> PathBuf {
    data_dir.join(VECIDX_FILE)
}

/// Get the path to snapshots directory
pub fn snapshots_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(SNAPSHOT_DIR)
}

/// Load or initialize storage with automatic format detection
///
/// This is a convenience function that:
/// 1. Detects the storage format (vecdb vs raw)
/// 2. Loads appropriately  
/// 3. Compacts if necessary
/// 4. Returns the number of collections loaded
pub fn load_or_initialize(data_dir: &Path) -> Result<usize> {
    use tracing::info;

    if !data_dir.exists() {
        info!(
            "📁 Data directory does not exist, creating: {}",
            data_dir.display()
        );
        std::fs::create_dir_all(data_dir)?;
        return Ok(0);
    }

    let format = detect_format(data_dir);

    match format {
        StorageFormat::Compact => {
            info!("📦 Found vectorizer.vecdb - using compressed storage");

            // Verify integrity
            let reader = StorageReader::new(data_dir)?;
            let collections = reader.list_collections()?;

            info!("✅ Verified {} collections in archive", collections.len());
            Ok(collections.len())
        }
        StorageFormat::Legacy => {
            info!("📁 Found legacy format - will migrate on first load");

            // Count legacy collections
            let mut count = 0;
            if let Ok(entries) = std::fs::read_dir(data_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if let Some(name_str) = name.to_str() {
                        if name_str.ends_with("_vector_store.bin") {
                            count += 1;
                        }
                    }
                }
            }

            info!("📊 Found {} collections in legacy format", count);
            Ok(count)
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_detect_format_legacy() {
        let temp_dir = TempDir::new().unwrap();
        assert_eq!(detect_format(temp_dir.path()), StorageFormat::Legacy);
    }

    #[test]
    fn test_detect_format_compact() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(vecdb_path(temp_dir.path()), b"test").unwrap();
        assert_eq!(detect_format(temp_dir.path()), StorageFormat::Compact);
    }

    #[test]
    fn test_paths() {
        let data_dir = Path::new("/data");
        assert_eq!(vecdb_path(data_dir), Path::new("/data/vectorizer.vecdb"));
        assert_eq!(vecidx_path(data_dir), Path::new("/data/vectorizer.vecidx"));
        assert_eq!(snapshots_dir(data_dir), Path::new("/data/snapshots"));
    }
}
