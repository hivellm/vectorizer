//! Main Vectorizer configuration structure

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::config::FileWatcherYamlConfig;
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
    /// Summarization configuration
    pub summarization: SummarizationConfig,
    /// Projects configuration
    pub projects: Vec<ProjectConfig>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// GRPC port
    pub grpc_port: u16,
    /// MCP port
    pub mcp_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 15002,
            grpc_port: 15002,
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

impl Default for VectorizerConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            file_watcher: FileWatcherYamlConfig::default(),
            logging: LoggingConfig::default(),
            summarization: SummarizationConfig::default(),
            projects: Vec::new(),
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

        if let Ok(grpc_port) = std::env::var("VECTORIZER_GRPC_PORT") {
            if let Ok(port_num) = grpc_port.parse::<u16>() {
                config.server.grpc_port = port_num;
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

