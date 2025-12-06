//! Main Vectorizer configuration structure

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::auth::AuthConfig;
use crate::config::FileWatcherYamlConfig;
use crate::hub::HubConfig;
use crate::storage::StorageConfig;
use crate::summarization::SummarizationConfig;

/// Main Vectorizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorizerConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// File watcher configuration
    pub file_watcher: FileWatcherYamlConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// GPU configuration
    #[serde(default)]
    pub gpu: GpuConfig,
    /// Summarization configuration
    #[serde(default)]
    pub summarization: SummarizationConfig,
    /// Transmutation configuration
    #[serde(default)]
    pub transmutation: TransmutationConfig,
    /// Storage configuration (vecdb compaction)
    #[serde(default)]
    pub storage: StorageConfig,
    /// Projects configuration
    #[serde(default)]
    pub projects: Vec<ProjectConfig>,
    /// Cluster configuration (for distributed sharding)
    #[serde(default)]
    pub cluster: crate::cluster::ClusterConfig,
    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
    /// HiveHub Cloud integration configuration
    #[serde(default)]
    pub hub: HubConfig,
    /// File upload configuration
    #[serde(default)]
    pub file_upload: FileUploadConfig,
}

/// File upload configuration for direct file indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadConfig {
    /// Maximum file size in bytes (default: 10MB)
    #[serde(default = "FileUploadConfig::default_max_file_size")]
    pub max_file_size: usize,

    /// List of allowed file extensions (without dot)
    #[serde(default = "FileUploadConfig::default_allowed_extensions")]
    pub allowed_extensions: Vec<String>,

    /// Reject binary files (default: true)
    #[serde(default = "FileUploadConfig::default_reject_binary")]
    pub reject_binary: bool,

    /// Default chunk size for uploaded files
    #[serde(default = "FileUploadConfig::default_chunk_size")]
    pub default_chunk_size: usize,

    /// Default chunk overlap for uploaded files
    #[serde(default = "FileUploadConfig::default_chunk_overlap")]
    pub default_chunk_overlap: usize,
}

impl FileUploadConfig {
    fn default_max_file_size() -> usize {
        10 * 1024 * 1024 // 10MB
    }

    fn default_allowed_extensions() -> Vec<String> {
        vec![
            // Text files
            "txt".to_string(),
            "md".to_string(),
            "rst".to_string(),
            "text".to_string(),
            // Code files
            "rs".to_string(),
            "py".to_string(),
            "js".to_string(),
            "ts".to_string(),
            "jsx".to_string(),
            "tsx".to_string(),
            "go".to_string(),
            "java".to_string(),
            "c".to_string(),
            "cpp".to_string(),
            "h".to_string(),
            "hpp".to_string(),
            "cs".to_string(),
            "rb".to_string(),
            "php".to_string(),
            "swift".to_string(),
            "kt".to_string(),
            "scala".to_string(),
            "r".to_string(),
            "sql".to_string(),
            "sh".to_string(),
            "bash".to_string(),
            "zsh".to_string(),
            "ps1".to_string(),
            "bat".to_string(),
            "cmd".to_string(),
            // Config files
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            "toml".to_string(),
            "xml".to_string(),
            "ini".to_string(),
            "cfg".to_string(),
            "conf".to_string(),
            // Web files
            "html".to_string(),
            "htm".to_string(),
            "css".to_string(),
            "scss".to_string(),
            "sass".to_string(),
            "less".to_string(),
            // Other text files
            "csv".to_string(),
            "log".to_string(),
            "env".to_string(),
            "gitignore".to_string(),
            "dockerignore".to_string(),
            "makefile".to_string(),
            "dockerfile".to_string(),
        ]
    }

    fn default_reject_binary() -> bool {
        true
    }

    fn default_chunk_size() -> usize {
        2048
    }

    fn default_chunk_overlap() -> usize {
        256
    }
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: Self::default_max_file_size(),
            allowed_extensions: Self::default_allowed_extensions(),
            reject_binary: Self::default_reject_binary(),
            default_chunk_size: Self::default_chunk_size(),
            default_chunk_overlap: Self::default_chunk_overlap(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// MCP port
    pub mcp_port: u16,
    /// Cleanup empty collections on startup
    #[serde(default)]
    pub startup_cleanup_empty: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 15002,
            mcp_port: 15003,
            startup_cleanup_empty: false,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log requests
    pub log_requests: bool,
    /// Log responses
    pub log_responses: bool,
    /// Log errors
    pub log_errors: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_requests: true,
            log_responses: false,
            log_errors: true,
        }
    }
}

/// GPU configuration for Metal acceleration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Enable GPU acceleration (default: true on macOS, false on other platforms)
    #[serde(default = "GpuConfig::default_enabled")]
    pub enabled: bool,

    /// Batch size for GPU batch operations (default: 1000)
    #[serde(default = "GpuConfig::default_batch_size")]
    pub batch_size: usize,

    /// Fallback to CPU if GPU initialization fails (default: true)
    #[serde(default = "GpuConfig::default_fallback_to_cpu")]
    pub fallback_to_cpu: bool,

    /// Preferred backend (auto/metal/cpu)
    #[serde(default = "GpuConfig::default_preferred_backend")]
    pub preferred_backend: String,
}

impl GpuConfig {
    fn default_enabled() -> bool {
        // Enable by default only on macOS (Metal support)
        cfg!(target_os = "macos")
    }

    fn default_batch_size() -> usize {
        1000
    }

    fn default_fallback_to_cpu() -> bool {
        true
    }

    fn default_preferred_backend() -> String {
        "auto".to_string()
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            batch_size: Self::default_batch_size(),
            fallback_to_cpu: Self::default_fallback_to_cpu(),
            preferred_backend: Self::default_preferred_backend(),
        }
    }
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project path
    pub path: String,
    /// Collections in this project
    pub collections: Vec<CollectionConfig>,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Collection name
    pub name: String,
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Chunk size
    pub chunk_size: usize,
    /// Chunk overlap
    pub chunk_overlap: usize,
    /// Embedding provider
    pub embedding_provider: String,
    /// Vector dimension
    pub dimension: u32,
}

/// Transmutation configuration for document conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmutationConfig {
    /// Enable transmutation document conversion
    #[serde(default = "default_transmutation_enabled")]
    pub enabled: bool,
    /// Maximum file size in MB for conversion
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: usize,
    /// Conversion timeout in seconds
    #[serde(default = "default_conversion_timeout_secs")]
    pub conversion_timeout_secs: u64,
    /// Preserve images during conversion
    #[serde(default)]
    pub preserve_images: bool,
}

fn default_transmutation_enabled() -> bool {
    cfg!(feature = "transmutation")
}

fn default_max_file_size_mb() -> usize {
    50
}

fn default_conversion_timeout_secs() -> u64 {
    300
}

impl Default for TransmutationConfig {
    fn default() -> Self {
        Self {
            enabled: default_transmutation_enabled(),
            max_file_size_mb: default_max_file_size_mb(),
            conversion_timeout_secs: default_conversion_timeout_secs(),
            preserve_images: false,
        }
    }
}

impl Default for VectorizerConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            file_watcher: FileWatcherYamlConfig::default(),
            logging: LoggingConfig::default(),
            gpu: GpuConfig::default(),
            cluster: crate::cluster::ClusterConfig::default(),
            summarization: SummarizationConfig::default(),
            transmutation: TransmutationConfig::default(),
            storage: StorageConfig::default(),
            projects: Vec::new(),
            auth: AuthConfig::default(),
            hub: HubConfig::default(),
            file_upload: FileUploadConfig::default(),
        }
    }
}

impl VectorizerConfig {
    /// Load configuration from YAML file
    pub fn from_yaml_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn to_yaml_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(host) = std::env::var("VECTORIZER_HOST") {
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("VECTORIZER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.server.port = port_num;
            }
        }

        if let Ok(mcp_port) = std::env::var("VECTORIZER_MCP_PORT") {
            if let Ok(port_num) = mcp_port.parse::<u16>() {
                config.server.mcp_port = port_num;
            }
        }

        if let Ok(level) = std::env::var("VECTORIZER_LOG_LEVEL") {
            config.logging.level = level;
        }

        config
    }
}

// ========================================
// Configuration Tests from /tests
// ========================================

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_transmutation_config_default() {
        let config = TransmutationConfig::default();

        #[cfg(feature = "transmutation")]
        {
            assert!(config.enabled, "Should be enabled when feature is compiled");
        }

        #[cfg(not(feature = "transmutation"))]
        {
            assert!(
                !config.enabled,
                "Should be disabled when feature is not compiled"
            );
        }

        assert_eq!(config.max_file_size_mb, 50);
        assert_eq!(config.conversion_timeout_secs, 300);
        assert!(!config.preserve_images);
    }

    #[test]
    fn test_transmutation_config_custom() {
        let config = TransmutationConfig {
            enabled: true,
            max_file_size_mb: 100,
            conversion_timeout_secs: 600,
            preserve_images: true,
        };

        assert!(config.enabled);
        assert_eq!(config.max_file_size_mb, 100);
        assert_eq!(config.conversion_timeout_secs, 600);
        assert!(config.preserve_images);
    }

    #[test]
    fn test_transmutation_config_serialization() {
        let config = TransmutationConfig {
            enabled: true,
            max_file_size_mb: 75,
            conversion_timeout_secs: 450,
            preserve_images: false,
        };

        // Test that config can be serialized
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("enabled"));
        assert!(serialized.contains("max_file_size_mb"));
        assert!(serialized.contains("conversion_timeout_secs"));
        assert!(serialized.contains("preserve_images"));
    }

    #[test]
    fn test_transmutation_config_deserialization() {
        let json = r#"{
            "enabled": true,
            "max_file_size_mb": 80,
            "conversion_timeout_secs": 500,
            "preserve_images": true
        }"#;

        let config: TransmutationConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.max_file_size_mb, 80);
        assert_eq!(config.conversion_timeout_secs, 500);
        assert!(config.preserve_images);
    }

    #[test]
    fn test_transmutation_config_in_vectorizer_config() {
        let config = VectorizerConfig::default();

        // Verify transmutation config is present
        #[cfg(feature = "transmutation")]
        {
            assert!(config.transmutation.enabled);
        }

        #[cfg(not(feature = "transmutation"))]
        {
            assert!(!config.transmutation.enabled);
        }
    }

    #[test]
    fn test_transmutation_config_boundaries() {
        // Test minimum values
        let config_min = TransmutationConfig {
            enabled: false,
            max_file_size_mb: 1,
            conversion_timeout_secs: 1,
            preserve_images: false,
        };
        assert_eq!(config_min.max_file_size_mb, 1);
        assert_eq!(config_min.conversion_timeout_secs, 1);

        // Test maximum reasonable values
        let config_max = TransmutationConfig {
            enabled: true,
            max_file_size_mb: 1000,        // 1GB
            conversion_timeout_secs: 3600, // 1 hour
            preserve_images: true,
        };
        assert_eq!(config_max.max_file_size_mb, 1000);
        assert_eq!(config_max.conversion_timeout_secs, 3600);
    }

    #[test]
    fn test_transmutation_config_yaml_format() {
        let config = TransmutationConfig {
            enabled: true,
            max_file_size_mb: 50,
            conversion_timeout_secs: 300,
            preserve_images: false,
        };

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("enabled"));
        assert!(yaml.contains("max_file_size_mb"));

        // Test YAML deserialization
        let config_from_yaml: TransmutationConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.enabled, config_from_yaml.enabled);
        assert_eq!(config.max_file_size_mb, config_from_yaml.max_file_size_mb);
    }

    // =========================================================================
    // FileUploadConfig Tests
    // =========================================================================

    #[test]
    fn test_file_upload_config_default() {
        let config = FileUploadConfig::default();

        assert_eq!(config.max_file_size, 10 * 1024 * 1024); // 10MB
        assert!(config.reject_binary);
        assert_eq!(config.default_chunk_size, 2048);
        assert_eq!(config.default_chunk_overlap, 256);
        assert!(!config.allowed_extensions.is_empty());
    }

    #[test]
    fn test_file_upload_config_default_extensions_count() {
        let config = FileUploadConfig::default();

        // Should have a reasonable number of extensions
        assert!(config.allowed_extensions.len() > 40);
        assert!(config.allowed_extensions.len() < 100);
    }

    #[test]
    fn test_file_upload_config_serialization() {
        let config = FileUploadConfig {
            max_file_size: 5 * 1024 * 1024,
            allowed_extensions: vec!["rs".to_string(), "py".to_string()],
            reject_binary: true,
            default_chunk_size: 1024,
            default_chunk_overlap: 128,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("max_file_size"));
        assert!(json.contains("5242880")); // 5MB in bytes
        assert!(json.contains("rs"));
        assert!(json.contains("py"));
        assert!(json.contains("reject_binary"));
        assert!(json.contains("default_chunk_size"));
        assert!(json.contains("default_chunk_overlap"));
    }

    #[test]
    fn test_file_upload_config_deserialization() {
        let json = r#"{
            "max_file_size": 20971520,
            "allowed_extensions": ["md", "txt", "json"],
            "reject_binary": false,
            "default_chunk_size": 4096,
            "default_chunk_overlap": 512
        }"#;

        let config: FileUploadConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_file_size, 20 * 1024 * 1024); // 20MB
        assert_eq!(config.allowed_extensions.len(), 3);
        assert!(config.allowed_extensions.contains(&"md".to_string()));
        assert!(!config.reject_binary);
        assert_eq!(config.default_chunk_size, 4096);
        assert_eq!(config.default_chunk_overlap, 512);
    }

    #[test]
    fn test_file_upload_config_yaml_format() {
        let config = FileUploadConfig {
            max_file_size: 10485760,
            allowed_extensions: vec!["rs".to_string(), "py".to_string(), "js".to_string()],
            reject_binary: true,
            default_chunk_size: 2048,
            default_chunk_overlap: 256,
        };

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("max_file_size"));
        assert!(yaml.contains("allowed_extensions"));
        assert!(yaml.contains("reject_binary"));

        // Test YAML deserialization
        let config_from_yaml: FileUploadConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.max_file_size, config_from_yaml.max_file_size);
        assert_eq!(config.reject_binary, config_from_yaml.reject_binary);
        assert_eq!(
            config.allowed_extensions.len(),
            config_from_yaml.allowed_extensions.len()
        );
    }

    #[test]
    fn test_file_upload_config_partial_deserialization() {
        // Test that missing fields use defaults
        let json = r#"{
            "max_file_size": 5242880
        }"#;

        let config: FileUploadConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        // Other fields should use defaults
        assert!(config.reject_binary); // default is true
        assert_eq!(config.default_chunk_size, 2048);
        assert_eq!(config.default_chunk_overlap, 256);
        assert!(!config.allowed_extensions.is_empty());
    }

    #[test]
    fn test_file_upload_config_empty_extensions() {
        let config = FileUploadConfig {
            max_file_size: 1024,
            allowed_extensions: vec![],
            reject_binary: true,
            default_chunk_size: 100,
            default_chunk_overlap: 10,
        };

        assert!(config.allowed_extensions.is_empty());
    }

    #[test]
    fn test_file_upload_config_in_vectorizer_config() {
        let config = VectorizerConfig::default();

        // Verify file_upload config is present and has correct defaults
        assert_eq!(config.file_upload.max_file_size, 10 * 1024 * 1024);
        assert!(config.file_upload.reject_binary);
        assert!(!config.file_upload.allowed_extensions.is_empty());
    }

    #[test]
    fn test_file_upload_config_clone() {
        let config = FileUploadConfig::default();
        let cloned = config.clone();

        assert_eq!(config.max_file_size, cloned.max_file_size);
        assert_eq!(config.reject_binary, cloned.reject_binary);
        assert_eq!(config.default_chunk_size, cloned.default_chunk_size);
        assert_eq!(
            config.allowed_extensions.len(),
            cloned.allowed_extensions.len()
        );
    }

    #[test]
    fn test_file_upload_config_debug() {
        let config = FileUploadConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("FileUploadConfig"));
        assert!(debug_str.contains("max_file_size"));
        assert!(debug_str.contains("reject_binary"));
    }

    #[test]
    fn test_file_upload_config_extension_categories() {
        let config = FileUploadConfig::default();

        // Text files
        assert!(config.allowed_extensions.contains(&"txt".to_string()));
        assert!(config.allowed_extensions.contains(&"md".to_string()));

        // Programming languages
        assert!(config.allowed_extensions.contains(&"rs".to_string()));
        assert!(config.allowed_extensions.contains(&"py".to_string()));
        assert!(config.allowed_extensions.contains(&"js".to_string()));
        assert!(config.allowed_extensions.contains(&"ts".to_string()));
        assert!(config.allowed_extensions.contains(&"go".to_string()));
        assert!(config.allowed_extensions.contains(&"java".to_string()));
        assert!(config.allowed_extensions.contains(&"c".to_string()));
        assert!(config.allowed_extensions.contains(&"cpp".to_string()));

        // Config files
        assert!(config.allowed_extensions.contains(&"json".to_string()));
        assert!(config.allowed_extensions.contains(&"yaml".to_string()));
        assert!(config.allowed_extensions.contains(&"yml".to_string()));
        assert!(config.allowed_extensions.contains(&"toml".to_string()));
        assert!(config.allowed_extensions.contains(&"xml".to_string()));

        // Web files
        assert!(config.allowed_extensions.contains(&"html".to_string()));
        assert!(config.allowed_extensions.contains(&"css".to_string()));
    }
}
