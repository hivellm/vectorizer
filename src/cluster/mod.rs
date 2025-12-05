//! Distributed cluster management for horizontal sharding
//!
//! This module provides cluster membership management, server discovery,
//! and distributed shard routing across multiple Vectorizer server instances.

mod grpc_service;
mod manager;
mod node;
mod server_client;
mod shard_router;
mod state_sync;
pub mod validator;

use std::sync::Arc;

pub use grpc_service::ClusterGrpcService;
pub use manager::ClusterManager;
pub use node::{ClusterNode, NodeId, NodeStatus};
use parking_lot::RwLock;
pub use server_client::{ClusterClient, ClusterClientPool};
pub use shard_router::DistributedShardRouter;
pub use state_sync::ClusterStateSynchronizer;
use tracing::{error, info, warn};
pub use validator::{
    ClusterConfigValidator, ClusterValidationError, ClusterValidationResult,
    ClusterValidationWarning,
};

/// Cluster configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClusterConfig {
    /// Whether cluster mode is enabled
    pub enabled: bool,
    /// This server's node ID
    pub node_id: Option<String>,
    /// List of cluster server addresses (for static discovery)
    pub servers: Vec<ServerConfig>,
    /// Discovery method (static, dns, etc.)
    #[serde(default = "default_discovery")]
    pub discovery: DiscoveryMethod,
    /// Cluster communication timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Retry count for failed operations
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    /// Memory limits configuration for cluster mode
    #[serde(default)]
    pub memory: ClusterMemoryConfig,
}

/// Memory configuration for cluster mode
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClusterMemoryConfig {
    /// Maximum total cache memory in bytes (default: 1GB)
    /// This limit applies globally across all caches in cluster mode
    #[serde(default = "default_max_cache_memory_bytes")]
    pub max_cache_memory_bytes: u64,
    /// Enforce MMap storage for all collections in cluster mode
    /// When true, Memory storage type will be rejected
    #[serde(default = "default_enforce_mmap_storage")]
    pub enforce_mmap_storage: bool,
    /// Disable file watcher in cluster mode
    /// File watcher is incompatible with distributed clusters
    #[serde(default = "default_disable_file_watcher")]
    pub disable_file_watcher: bool,
    /// Warning threshold for cache memory (percentage, 0-100)
    /// Emit warning when cache usage exceeds this percentage
    #[serde(default = "default_cache_warning_threshold")]
    pub cache_warning_threshold: u8,
    /// Enable strict validation of cluster configuration
    /// When true, server will fail to start if config violates cluster requirements
    #[serde(default = "default_strict_validation")]
    pub strict_validation: bool,
}

fn default_max_cache_memory_bytes() -> u64 {
    1024 * 1024 * 1024 // 1GB
}

fn default_enforce_mmap_storage() -> bool {
    true
}

fn default_disable_file_watcher() -> bool {
    true
}

fn default_cache_warning_threshold() -> u8 {
    80 // 80%
}

fn default_strict_validation() -> bool {
    true
}

impl Default for ClusterMemoryConfig {
    fn default() -> Self {
        Self {
            max_cache_memory_bytes: default_max_cache_memory_bytes(),
            enforce_mmap_storage: default_enforce_mmap_storage(),
            disable_file_watcher: default_disable_file_watcher(),
            cache_warning_threshold: default_cache_warning_threshold(),
            strict_validation: default_strict_validation(),
        }
    }
}

/// Server configuration for cluster membership
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    /// Server ID (unique identifier)
    pub id: String,
    /// Server address (host:port)
    pub address: String,
    /// gRPC port for inter-server communication
    pub grpc_port: u16,
}

/// Discovery method for cluster membership
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryMethod {
    /// Static configuration (server list in config file)
    Static,
    /// DNS-based discovery (future)
    Dns,
    /// Service registry discovery (future)
    ServiceRegistry,
}

fn default_discovery() -> DiscoveryMethod {
    DiscoveryMethod::Static
}

fn default_timeout_ms() -> u64 {
    5000 // 5 seconds
}

fn default_retry_count() -> u32 {
    3
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            node_id: None,
            servers: Vec::new(),
            discovery: DiscoveryMethod::Static,
            timeout_ms: 5000,
            retry_count: 3,
            memory: ClusterMemoryConfig::default(),
        }
    }
}
