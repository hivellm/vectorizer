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

pub use grpc_service::ClusterGrpcService;
pub use manager::ClusterManager;
pub use node::{ClusterNode, NodeId, NodeStatus};
pub use server_client::{ClusterClient, ClusterClientPool};
pub use shard_router::DistributedShardRouter;
pub use state_sync::ClusterStateSynchronizer;

use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, warn, error};

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
        }
    }
}

