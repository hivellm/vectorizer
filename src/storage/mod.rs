//! Storage module for vectorizer database
//!
//! This module provides a unified storage format (.vecdb/.vecidx) with compression,
//! snapshots, and automatic migration from legacy file structures.

pub mod config;
pub mod index;
pub mod reader;
pub mod writer;
pub mod compact;
pub mod migration;
pub mod snapshot;

pub use config::StorageConfig;
pub use index::{CollectionIndex, FileEntry, StorageIndex};
pub use reader::StorageReader;
pub use writer::StorageWriter;
pub use compact::StorageCompactor;
pub use migration::StorageMigrator;
pub use snapshot::{SnapshotManager, SnapshotInfo};

use crate::error::{Result, VectorizerError};
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

