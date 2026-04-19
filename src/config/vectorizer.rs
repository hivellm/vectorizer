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
    /// Replication configuration (master-replica)
    #[serde(default)]
    pub replication: ReplicationYamlConfig,
    /// VectorizerRPC binary protocol listener (length-prefixed
    /// MessagePack over raw TCP; see `docs/specs/VECTORIZER_RPC.md`).
    #[serde(default)]
    pub rpc: RpcConfig,
}

/// VectorizerRPC listener configuration. Disabled by default in v1
/// while the SDK matrix catches up; flip to `enabled: true` to expose
/// the binary transport on its own port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// When false the bootstrap skips the listener entirely.
    #[serde(default = "RpcConfig::default_enabled")]
    pub enabled: bool,
    /// Bind host. `0.0.0.0` exposes the listener on every interface;
    /// `127.0.0.1` keeps it loopback-only for development.
    #[serde(default = "RpcConfig::default_host")]
    pub host: String,
    /// TCP port. Default `15503` per `docs/specs/VECTORIZER_RPC.md`
    /// § 12 ("Default port 15503") — Vectorizer's slot in the
    /// 15500-range binary-transport convention shared with Synap.
    #[serde(default = "RpcConfig::default_port")]
    pub port: u16,
}

impl RpcConfig {
    fn default_enabled() -> bool {
        false
    }
    fn default_host() -> String {
        "0.0.0.0".to_string()
    }
    fn default_port() -> u16 {
        15503
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            host: Self::default_host(),
            port: Self::default_port(),
        }
    }
}

/// YAML-friendly replication configuration
/// Maps to `crate::replication::ReplicationConfig` at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationYamlConfig {
    /// Enable replication
    #[serde(default)]
    pub enabled: bool,
    /// Node role: "standalone", "master", "replica"
    #[serde(default = "default_replication_role")]
    pub role: String,
    /// Master bind address for replicas to connect (e.g., "0.0.0.0:7001")
    #[serde(default)]
    pub bind_address: Option<String>,
    /// Master address for replica to connect to (e.g., "master-host:7001")
    #[serde(default)]
    pub master_address: Option<String>,
    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat", alias = "heartbeat_interval_secs")]
    pub heartbeat_interval: u64,
    /// Replica timeout in seconds
    #[serde(default = "default_replica_timeout", alias = "replica_timeout_secs")]
    pub replica_timeout: u64,
    /// Replication log size
    #[serde(default = "default_log_size")]
    pub log_size: usize,
    /// Reconnect interval in seconds
    #[serde(default = "default_reconnect", alias = "reconnect_interval_secs")]
    pub reconnect_interval: u64,
    /// Enable WAL for durable replication
    #[serde(default = "default_wal_enabled")]
    pub wal_enabled: bool,
    /// WAL directory
    #[serde(default)]
    pub wal_dir: Option<String>,
}

fn default_replication_role() -> String {
    "standalone".to_string()
}
fn default_heartbeat() -> u64 {
    5
}
fn default_replica_timeout() -> u64 {
    30
}
fn default_log_size() -> usize {
    1_000_000
}
fn default_reconnect() -> u64 {
    5
}
fn default_wal_enabled() -> bool {
    true
}

impl Default for ReplicationYamlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            role: default_replication_role(),
            bind_address: None,
            master_address: None,
            heartbeat_interval: default_heartbeat(),
            replica_timeout: default_replica_timeout(),
            log_size: default_log_size(),
            reconnect_interval: default_reconnect(),
            wal_enabled: default_wal_enabled(),
            wal_dir: None,
        }
    }
}

impl ReplicationYamlConfig {
    /// Convert to runtime ReplicationConfig.
    ///
    /// Addresses can be either `IP:port` or `hostname:port`.
    /// DNS hostnames are resolved synchronously at config load time.
    pub fn to_replication_config(&self) -> crate::replication::ReplicationConfig {
        let role = match self.role.as_str() {
            "master" => crate::replication::NodeRole::Master,
            "replica" => crate::replication::NodeRole::Replica,
            _ => crate::replication::NodeRole::Standalone,
        };

        let bind_address = self
            .bind_address
            .as_ref()
            .and_then(|addr| resolve_address(addr));
        let master_address = self
            .master_address
            .as_ref()
            .and_then(|addr| resolve_address(addr));

        // Preserve the raw master address string so that DNS hostnames
        // (e.g. K8s StatefulSet names) can be re-resolved on each reconnect.
        let master_address_raw = self.master_address.clone();

        crate::replication::ReplicationConfig {
            role,
            bind_address,
            master_address,
            master_address_raw,
            heartbeat_interval: self.heartbeat_interval,
            replica_timeout: self.replica_timeout,
            log_size: self.log_size,
            reconnect_interval: self.reconnect_interval,
            wal_enabled: self.wal_enabled,
            wal_dir: self.wal_dir.clone(),
        }
    }
}

/// Resolve an address string that may be `IP:port` or `hostname:port`.
/// Tries `SocketAddr::parse` first (fast), falls back to DNS resolution.
fn resolve_address(addr: &str) -> Option<std::net::SocketAddr> {
    // Try direct parse first (e.g., "127.0.0.1:7001")
    if let Ok(sock) = addr.parse::<std::net::SocketAddr>() {
        return Some(sock);
    }

    // Try DNS resolution (e.g., "vz-ha-master:7001")
    match std::net::ToSocketAddrs::to_socket_addrs(&addr) {
        Ok(mut addrs) => {
            if let Some(resolved) = addrs.next() {
                tracing::info!("Resolved '{}' → {}", addr, resolved);
                Some(resolved)
            } else {
                tracing::warn!("DNS resolution for '{}' returned no addresses", addr);
                None
            }
        }
        Err(e) => {
            tracing::warn!("Failed to resolve address '{}': {}", addr, e);
            None
        }
    }
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
            replication: ReplicationYamlConfig::default(),
            rpc: RpcConfig::default(),
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
#[path = "vectorizer_tests.rs"]
mod tests;
