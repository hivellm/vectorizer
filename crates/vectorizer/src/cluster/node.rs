//! Cluster node representation and status management

use std::collections::HashSet;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::db::sharding::ShardId;

/// Unique node identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create a new node ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the node ID as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Node status in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Node is active and healthy
    Active,
    /// Node is joining the cluster
    Joining,
    /// Node is leaving the cluster
    Leaving,
    /// Node is unavailable (failed or unreachable)
    Unavailable,
}

/// Cluster node representing a server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// Unique node identifier
    pub id: NodeId,
    /// Server address (host:port)
    pub address: String,
    /// gRPC port for inter-server communication
    pub grpc_port: u16,
    /// Current node status
    pub status: NodeStatus,
    /// Shards assigned to this node
    pub shards: HashSet<ShardId>,
    /// Last heartbeat timestamp
    #[serde(skip)]
    pub last_heartbeat: Option<Instant>,
    /// Node metadata (version, capabilities, etc.)
    pub metadata: NodeMetadata,
}

/// Node metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetadata {
    /// Server version
    pub version: Option<String>,
    /// Server capabilities
    pub capabilities: Vec<String>,
    /// Number of vectors stored on this node
    pub vector_count: usize,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f32,
}

impl ClusterNode {
    /// Create a new cluster node
    pub fn new(id: NodeId, address: String, grpc_port: u16) -> Self {
        Self {
            id,
            address,
            grpc_port,
            status: NodeStatus::Joining,
            shards: HashSet::new(),
            last_heartbeat: Some(Instant::now()),
            metadata: NodeMetadata::default(),
        }
    }

    /// Get the gRPC address for this node
    pub fn grpc_address(&self) -> String {
        format!("{}:{}", self.address, self.grpc_port)
    }

    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Some(Instant::now());
        if self.status == NodeStatus::Unavailable {
            debug!("Node {} recovered, marking as active", self.id);
            self.status = NodeStatus::Active;
        }
    }

    /// Check if node is healthy (heartbeat within timeout)
    pub fn is_healthy(&self, timeout: Duration) -> bool {
        match self.last_heartbeat {
            Some(last) => last.elapsed() < timeout,
            None => false,
        }
    }

    /// Mark node as unavailable
    pub fn mark_unavailable(&mut self) {
        if self.status != NodeStatus::Unavailable {
            warn!("Node {} marked as unavailable", self.id);
            self.status = NodeStatus::Unavailable;
        }
    }

    /// Mark node as active
    pub fn mark_active(&mut self) {
        if self.status != NodeStatus::Active {
            debug!("Node {} marked as active", self.id);
            self.status = NodeStatus::Active;
            self.update_heartbeat();
        }
    }

    /// Add a shard to this node
    pub fn add_shard(&mut self, shard_id: ShardId) {
        self.shards.insert(shard_id);
    }

    /// Remove a shard from this node
    pub fn remove_shard(&mut self, shard_id: &ShardId) -> bool {
        self.shards.remove(shard_id)
    }

    /// Get the number of shards assigned to this node
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    /// Check if node has a specific shard
    pub fn has_shard(&self, shard_id: &ShardId) -> bool {
        self.shards.contains(shard_id)
    }
}
