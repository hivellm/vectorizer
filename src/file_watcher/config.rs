//! Configuration for File Watcher System

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the File Watcher System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherConfig {
    /// Paths to watch for changes (optional - auto-discovered if not provided)
    pub watch_paths: Option<Vec<PathBuf>>,
    
    /// File patterns to include (glob patterns)
    pub include_patterns: Vec<String>,
    
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,
    
    /// Debounce delay in milliseconds
    pub debounce_delay_ms: u64,
    
    /// Maximum file size to process (in bytes)
    pub max_file_size: u64,
    
    /// Enable content hash validation
    pub enable_hash_validation: bool,
    
    /// Collection name for indexed files
    pub collection_name: String,
    
    /// Enable recursive directory watching
    pub recursive: bool,
    
    /// Maximum number of concurrent file processing tasks
    pub max_concurrent_tasks: usize,
    
    /// Enable real-time indexing
    pub enable_realtime_indexing: bool,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    
    /// Log level for file watcher operations
    pub log_level: String,
    
    /// Enable auto-discovery of files
    pub auto_discovery: bool,
    
    /// Enable auto-update of collections
    pub enable_auto_update: bool,
    
    /// Enable hot reload
    pub hot_reload: bool,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            watch_paths: None, // Auto-discovered from indexed files
            include_patterns: vec![
                "*.md".to_string(),
                "*.txt".to_string(),
                "*.rs".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.json".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
            ],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/.*".to_string(),
                "**/*.tmp".to_string(),
                "**/*.log".to_string(),
                "**/*.part".to_string(),
                "**/*.lock".to_string(),
                "**/~*".to_string(),
                "**/.#*".to_string(),
                "**/*.swp".to_string(),
                "**/*.swo".to_string(),
                "**/Cargo.lock".to_string(),
                "**/.DS_Store".to_string(),
                "**/Thumbs.db".to_string(),
            ],
            debounce_delay_ms: 1000,
            max_file_size: 10 * 1024 * 1024, // 10MB
            enable_hash_validation: true,
            collection_name: "watched_files".to_string(),
            recursive: true,
            max_concurrent_tasks: 4,
            enable_realtime_indexing: true,
            batch_size: 100,
            enable_monitoring: true,
            log_level: "info".to_string(),
            auto_discovery: true,
            enable_auto_update: true,
            hot_reload: true,
        }
    }
}

impl FileWatcherConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from YAML file
    pub fn from_yaml_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn to_yaml_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // watch_paths is now optional (auto-discovered)
        // if self.watch_paths.is_empty() {
        //     return Err("At least one watch path must be specified".to_string());
        // }

        if self.debounce_delay_ms == 0 {
            return Err("Debounce delay must be greater than 0".to_string());
        }

        if self.max_file_size == 0 {
            return Err("Max file size must be greater than 0".to_string());
        }

        if self.max_concurrent_tasks == 0 {
            return Err("Max concurrent tasks must be greater than 0".to_string());
        }

        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        if self.collection_name.is_empty() {
            return Err("Collection name cannot be empty".to_string());
        }

        // Validate watch paths exist (if provided)
        if let Some(watch_paths) = &self.watch_paths {
            for path in watch_paths {
                if !path.exists() {
                    return Err(format!("Watch path does not exist: {:?}", path));
                }
            }
        }

        Ok(())
    }

    /// Get debounce duration
    pub fn debounce_duration(&self) -> Duration {
        Duration::from_millis(self.debounce_delay_ms)
    }

    /// Check if a file should be processed based on patterns
    pub fn should_process_file(&self, file_path: &std::path::Path) -> bool {
        let file_path_str = file_path.to_string_lossy();

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&file_path_str))
                .unwrap_or(false)
            {
                tracing::info!("🚫 File excluded by pattern '{}': {:?}", pattern, file_path);
                return false;
            }
        }

        // Check include patterns
        if self.include_patterns.is_empty() {
            tracing::debug!("No include patterns, allowing file: {:?}", file_path);
            return true; // No include patterns means include all
        }

        for pattern in &self.include_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&file_path_str))
                .unwrap_or(false)
            {
                tracing::info!("✅ File included by pattern '{}': {:?}", pattern, file_path);
                return true;
            }
        }

        tracing::info!("❌ File doesn't match any include patterns: {:?}", file_path);
        false
    }

    /// Check if a file should be processed based on patterns (silent version - no logging)
    pub fn should_process_file_silent(&self, file_path: &std::path::Path) -> bool {
        let file_path_str = file_path.to_string_lossy();

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&file_path_str))
                .unwrap_or(false)
            {
                return false;
            }
        }

        // Check include patterns
        if self.include_patterns.is_empty() {
            return true; // No include patterns means include all
        }

        for pattern in &self.include_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&file_path_str))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileWatcherConfig::default();
        // watch_paths is now optional
        // assert!(!config.watch_paths.is_empty());
        assert!(!config.include_patterns.is_empty());
        assert!(!config.exclude_patterns.is_empty());
        assert!(config.debounce_delay_ms > 0);
        assert!(config.max_file_size > 0);
        assert!(!config.collection_name.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = FileWatcherConfig::default();
        assert!(config.validate().is_ok());

        // Test empty watch paths (now optional, so no error)
        // config.watch_paths.clear();
        // assert!(config.validate().is_err());

        // Test zero debounce delay
        config.watch_paths = Some(vec![PathBuf::from(".")]);
        config.debounce_delay_ms = 0;
        assert!(config.validate().is_err());

        // Test zero max file size
        config.debounce_delay_ms = 1000;
        config.max_file_size = 0;
        assert!(config.validate().is_err());

        // Test empty collection name
        config.max_file_size = 1024;
        config.collection_name = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_file_pattern_matching() {
        let config = FileWatcherConfig::default();
        
        // Test include patterns
        assert!(config.should_process_file(std::path::Path::new("test.md")));
        assert!(config.should_process_file(std::path::Path::new("test.rs")));
        assert!(config.should_process_file(std::path::Path::new("test.py")));
        
        // Test exclude patterns
        assert!(!config.should_process_file(std::path::Path::new("target/debug/test")));
        assert!(!config.should_process_file(std::path::Path::new("node_modules/test")));
        assert!(!config.should_process_file(std::path::Path::new(".git/config")));
    }

    #[test]
    fn test_duration_conversion() {
        let config = FileWatcherConfig::default();
        assert_eq!(config.debounce_duration(), Duration::from_millis(1000));
    }
}
