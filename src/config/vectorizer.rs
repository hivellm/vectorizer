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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 15002,
            mcp_port: 15003,
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
}
