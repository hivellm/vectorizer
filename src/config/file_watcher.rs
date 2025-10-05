//! File Watcher configuration structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File Watcher configuration for YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherYamlConfig {
    /// Enable file watcher
    pub enabled: bool,
    /// Paths to watch for changes (optional - auto-discovered if not provided)
    pub watch_paths: Option<Vec<String>>,
    /// Enable recursive directory watching
    pub recursive: Option<bool>,
    /// Debounce delay in milliseconds
    pub debounce_delay_ms: Option<u64>,
    /// File patterns to include (glob patterns)
    pub include_patterns: Option<Vec<String>>,
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Option<Vec<String>>,
    /// Minimum file size in bytes
    pub min_file_size_bytes: Option<u64>,
    /// Maximum file size in bytes
    pub max_file_size_bytes: Option<u64>,
    /// Enable content hash validation
    pub hash_validation_enabled: Option<bool>,
    /// Collection name for indexed files
    pub collection_name: Option<String>,
}

impl Default for FileWatcherYamlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watch_paths: None, // Auto-discovered from indexed files
            recursive: Some(true),
            debounce_delay_ms: Some(1000),
            include_patterns: Some(vec!["*.txt".to_string(), "*.md".to_string()]),
            exclude_patterns: Some(vec![
                "*.log".to_string(), 
                "*.tmp".to_string(),
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/.*".to_string(),
                "**/*.tmp*".to_string(),
                "**/*~".to_string(),
            ]),
            min_file_size_bytes: Some(1),
            max_file_size_bytes: Some(10 * 1024 * 1024), // 10MB
            hash_validation_enabled: Some(true),
            collection_name: Some("default_collection".to_string()),
        }
    }
}

impl FileWatcherYamlConfig {
    /// Convert to FileWatcherConfig
    pub fn to_file_watcher_config(&self) -> crate::file_watcher::FileWatcherConfig {
        crate::file_watcher::FileWatcherConfig {
            watch_paths: self.watch_paths.as_ref().map(|paths| paths.iter().map(|p| PathBuf::from(p)).collect()),
            include_patterns: self.include_patterns.clone().unwrap_or_else(|| vec!["*.md".to_string(), "*.txt".to_string(), "*.rs".to_string(), "*.py".to_string(), "*.js".to_string(), "*.ts".to_string(), "*.json".to_string(), "*.yaml".to_string(), "*.yml".to_string()]),
            exclude_patterns: self.exclude_patterns.clone().unwrap_or_else(|| vec!["**/target/**".to_string(), "**/node_modules/**".to_string(), "**/.git/**".to_string(), "**/.*".to_string(), "**/*.tmp".to_string(), "**/*.log".to_string(), "**/*.part".to_string(), "**/*.lock".to_string(), "**/~*".to_string(), "**/.#*".to_string(), "**/*.swp".to_string(), "**/*.swo".to_string(), "**/Cargo.lock".to_string(), "**/.DS_Store".to_string(), "**/Thumbs.db".to_string()]),
            debounce_delay_ms: self.debounce_delay_ms.unwrap_or(1000),
            max_file_size: self.max_file_size_bytes.unwrap_or(10 * 1024 * 1024),
            enable_hash_validation: self.hash_validation_enabled.unwrap_or(true),
            collection_name: self.collection_name.clone().unwrap_or_else(|| "default_collection".to_string()),
            recursive: self.recursive.unwrap_or(true),
            max_concurrent_tasks: 4,
            enable_realtime_indexing: true,
            batch_size: 10,
            enable_monitoring: true,
            log_level: "info".to_string(),
        }
    }
}
