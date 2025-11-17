//! Storage configuration structures

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Compression settings
    #[serde(default)]
    pub compression: CompressionConfig,

    /// Snapshot settings
    #[serde(default)]
    pub snapshots: SnapshotConfig,

    /// Compaction settings
    #[serde(default)]
    pub compaction: CompactionConfig,

    /// Advanced storage settings
    #[serde(default)]
    pub advanced: AdvancedStorageConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            compression: CompressionConfig::default(),
            snapshots: SnapshotConfig::default(),
            compaction: CompactionConfig::default(),
            advanced: AdvancedStorageConfig::default(),
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Compression format (currently only "zstd" is supported)
    #[serde(default = "default_format")]
    pub format: String,

    /// Compression level (1-22 for zstd, 3 is balanced)
    #[serde(default = "default_level")]
    pub level: i32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: "zstd".to_string(),
            level: 3,
        }
    }
}

/// Snapshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Enable automatic snapshots
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Interval between snapshots in hours
    #[serde(default = "default_interval_hours")]
    pub interval_hours: u64,

    /// Retention period in days
    #[serde(default = "default_retention_days")]
    pub retention_days: u64,

    /// Maximum number of snapshots to keep
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots: usize,

    /// Path to snapshots directory
    #[serde(default = "default_snapshot_path")]
    pub path: String,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 1,
            retention_days: 2,
            max_snapshots: 48,
            path: "./data/snapshots".to_string(),
        }
    }
}

/// Compaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Number of operations to batch before consolidating
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Enable automatic compaction
    #[serde(default = "default_enabled")]
    pub auto_compact: bool,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            auto_compact: true,
        }
    }
}

// Default value functions for serde
fn default_enabled() -> bool {
    true
}

fn default_format() -> String {
    "zstd".to_string()
}

fn default_level() -> i32 {
    3
}

fn default_interval_hours() -> u64 {
    1
}

fn default_retention_days() -> u64 {
    2 // 48 hours retention (2 days)
}

fn default_max_snapshots() -> usize {
    48 // 24 snapshots/day * 2 days = 48 snapshots max
}

fn default_snapshot_path() -> String {
    "./data/snapshots".to_string()
}

fn default_batch_size() -> usize {
    1000
}

/// Advanced storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedStorageConfig {
    /// Enable on-disk vector storage
    #[serde(default = "default_enabled")]
    pub on_disk_storage: bool,

    /// Enable memory-mapped files for efficient access
    #[serde(default = "default_enabled")]
    pub memory_mapped_files: bool,

    /// Maximum cache size for memory-mapped files in MB
    #[serde(default = "default_mmap_cache_size_mb")]
    pub mmap_cache_size_mb: usize,

    /// Enable storage optimization (compaction, defragmentation)
    #[serde(default = "default_enabled")]
    pub optimization_enabled: bool,

    /// Automatic optimization interval in hours
    #[serde(default = "default_optimization_interval_hours")]
    pub optimization_interval_hours: u64,

    /// Enable storage logging
    #[serde(default = "default_enabled")]
    pub logging_enabled: bool,

    /// Enable storage metrics
    #[serde(default = "default_enabled")]
    pub metrics_enabled: bool,
}

impl Default for AdvancedStorageConfig {
    fn default() -> Self {
        Self {
            on_disk_storage: true,
            memory_mapped_files: true,
            mmap_cache_size_mb: 1024, // 1GB
            optimization_enabled: true,
            optimization_interval_hours: 24,
            logging_enabled: true,
            metrics_enabled: true,
        }
    }
}

fn default_mmap_cache_size_mb() -> usize {
    1024
}

fn default_optimization_interval_hours() -> u64 {
    24
}

impl StorageConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.compression.enabled && self.compression.format != "zstd" {
            return Err(crate::error::VectorizerError::Configuration(format!(
                "Unsupported compression format: {}",
                self.compression.format
            )));
        }

        if self.compression.level < 1 || self.compression.level > 22 {
            return Err(crate::error::VectorizerError::Configuration(
                "Compression level must be between 1 and 22".to_string(),
            ));
        }

        if self.compaction.batch_size == 0 {
            return Err(crate::error::VectorizerError::Configuration(
                "Batch size must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the snapshots directory path
    pub fn snapshots_path(&self) -> PathBuf {
        PathBuf::from(&self.snapshots.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StorageConfig::default();
        assert!(config.compression.enabled);
        assert_eq!(config.compression.format, "zstd");
        assert_eq!(config.compression.level, 3);
        assert!(config.snapshots.enabled);
        assert_eq!(config.snapshots.interval_hours, 1);
        assert_eq!(config.compaction.batch_size, 1000);
    }

    #[test]
    fn test_config_validation() {
        let config = StorageConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid = config.clone();
        invalid.compression.format = "invalid".to_string();
        assert!(invalid.validate().is_err());

        let mut invalid_level = config.clone();
        invalid_level.compression.level = 0;
        assert!(invalid_level.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = StorageConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("compression"));
        assert!(yaml.contains("snapshots"));
        assert!(yaml.contains("compaction"));

        let deserialized: StorageConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.compression.level, config.compression.level);
    }
}
