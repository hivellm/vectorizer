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
            include_patterns: vec![], // Will be loaded from workspace config
            exclude_patterns: Self::get_hardcoded_exclude_patterns(), // Hardcoded patterns
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

    /// Get hardcoded exclude patterns to prevent processing of system files
    pub fn get_hardcoded_exclude_patterns() -> Vec<String> {
        vec![
            // Version control and build artifacts
            "**/.git/**".to_string(),
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.*".to_string(),
            
            // Temporary and cache files
            "**/*.tmp".to_string(),
            "**/*.log".to_string(),
            "**/*.part".to_string(),
            "**/*.lock".to_string(),
            "**/~*".to_string(),
            "**/.#*".to_string(),
            "**/*.swp".to_string(),
            "**/*.swo".to_string(),
            "**/Cargo.lock".to_string(),
            
            // System files
            "**/.DS_Store".to_string(),
            "**/Thumbs.db".to_string(),
            "**/.fingerprint/**".to_string(),
            
            // Vectorizer data files (CRITICAL - prevents loops)
            "**/data/**".to_string(),
            "**/*_metadata.json".to_string(),
            "**/*_tokenizer.json".to_string(),
            "**/*_vector_store.bin".to_string(),
            
            // Additional system patterns
            "**/__pycache__/**".to_string(),
            "**/*.pyc".to_string(),
            "**/*.pyo".to_string(),
            "**/venv/**".to_string(),
            "**/env/**".to_string(),
            "**/.env".to_string(),
            "**/dist/**".to_string(),
            "**/build/**".to_string(),
            "**/out/**".to_string(),
            "**/bin/**".to_string(),
            "**/obj/**".to_string(),
        ]
    }

    /// Load configuration from workspace file
    pub async fn from_workspace() -> Result<Self, Box<dyn std::error::Error>> {
        let workspace_file = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("vectorize-workspace.yml");
        
        if !workspace_file.exists() {
            return Err(format!("Workspace file not found: {:?}", workspace_file).into());
        }
        
        let content = tokio::fs::read_to_string(&workspace_file).await?;
        let workspace: serde_yaml::Value = serde_yaml::from_str(&content)?;
        
        let mut config = Self::default();
        
        // Load global file watcher settings
        if let Some(global_settings) = workspace.get("global_settings") {
            if let Some(file_watcher) = global_settings.get("file_watcher") {
                // Load watch paths
                if let Some(paths) = file_watcher.get("watch_paths") {
                    if let Some(paths_array) = paths.as_sequence() {
                        config.watch_paths = Some(paths_array.iter()
                            .filter_map(|p| p.as_str())
                            .map(|p| PathBuf::from(p))
                            .collect());
                    }
                }
                
                // Load include patterns
                if let Some(patterns) = file_watcher.get("include_patterns") {
                    if let Some(patterns_array) = patterns.as_sequence() {
                        config.include_patterns = patterns_array.iter()
                            .filter_map(|p| p.as_str())
                            .map(|s| s.to_string())
                            .collect();
                    }
                }
                
                // Exclude patterns are now hardcoded - ignore any from YAML
                // config.exclude_patterns = Self::get_hardcoded_exclude_patterns();
                
                // Load other settings
                if let Some(debounce) = file_watcher.get("debounce_delay_ms") {
                    if let Some(debounce_val) = debounce.as_u64() {
                        config.debounce_delay_ms = debounce_val;
                    }
                }
                
                if let Some(auto_discovery) = file_watcher.get("auto_discovery") {
                    if let Some(auto_discovery_val) = auto_discovery.as_bool() {
                        config.auto_discovery = auto_discovery_val;
                    }
                }
                
                if let Some(enable_auto_update) = file_watcher.get("enable_auto_update") {
                    if let Some(enable_auto_update_val) = enable_auto_update.as_bool() {
                        config.enable_auto_update = enable_auto_update_val;
                    }
                }
                
                if let Some(hot_reload) = file_watcher.get("hot_reload") {
                    if let Some(hot_reload_val) = hot_reload.as_bool() {
                        config.hot_reload = hot_reload_val;
                    }
                }
            }
        }
        
        // If no patterns were loaded, use sensible defaults
        if config.include_patterns.is_empty() {
            config.include_patterns = vec![
                "*.md".to_string(),
                "*.txt".to_string(),
                "*.rs".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.json".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
            ];
        }
        
        // Exclude patterns are always hardcoded - no need to check if empty
        
        Ok(config)
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
        // Note: include_patterns and exclude_patterns are now loaded from workspace config
        // assert!(!config.include_patterns.is_empty());
        // assert!(!config.exclude_patterns.is_empty());
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
        let mut config = FileWatcherConfig::default();
        
        // Set up test patterns
        config.include_patterns = vec![
            "*.md".to_string(),
            "*.rs".to_string(),
            "*.py".to_string(),
        ];
        config.exclude_patterns = vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
        ];
        
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
