//! File Watcher configuration structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File Watcher configuration for YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherYamlConfig {
    /// Enable file watcher
    pub enabled: bool,
    /// Paths to watch for changes
    pub watch_paths: Vec<String>,
    /// Enable recursive directory watching
    pub recursive: bool,
    /// Debounce delay in milliseconds
    pub debounce_delay_ms: u64,
    /// File patterns to include (glob patterns)
    pub include_patterns: Vec<String>,
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,
    /// Minimum file size in bytes
    pub min_file_size_bytes: u64,
    /// Maximum file size in bytes
    pub max_file_size_bytes: u64,
    /// Enable content hash validation
    pub hash_validation_enabled: bool,
    /// Enable GRPC integration
    pub grpc_enabled: bool,
    /// Collection name for indexed files
    pub collection_name: String,
}

impl Default for FileWatcherYamlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watch_paths: vec![".".to_string()],
            recursive: true,
            debounce_delay_ms: 1000,
            include_patterns: vec!["*.txt".to_string(), "*.md".to_string()],
            exclude_patterns: vec![
                "*.log".to_string(), 
                "*.tmp".to_string(),
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/.*".to_string(),
                "**/*.tmp*".to_string(),
                "**/*~".to_string(),
            ],
            min_file_size_bytes: 1,
            max_file_size_bytes: 10 * 1024 * 1024, // 10MB
            hash_validation_enabled: true,
            grpc_enabled: true,
            collection_name: "default_collection".to_string(),
        }
    }
}

impl FileWatcherYamlConfig {
    /// Convert to FileWatcherConfig
    pub fn to_file_watcher_config(&self) -> crate::file_watcher::FileWatcherConfig {
        crate::file_watcher::FileWatcherConfig {
            watch_paths: self.watch_paths.iter().map(|p| PathBuf::from(p)).collect(),
            include_patterns: self.include_patterns.clone(),
            exclude_patterns: self.exclude_patterns.clone(),
            debounce_delay_ms: self.debounce_delay_ms,
            max_file_size: self.max_file_size_bytes,
            enable_hash_validation: self.hash_validation_enabled,
            grpc_endpoint: None,
            collection_name: self.collection_name.clone(),
            recursive: self.recursive,
            max_concurrent_tasks: 4,
            enable_realtime_indexing: true,
            batch_size: 10,
            grpc_timeout_ms: 5000,
            enable_monitoring: true,
            log_level: "info".to_string(),
        }
    }
}
