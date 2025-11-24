//! Cluster manager for membership and state coordination

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

use super::node::{ClusterNode, NodeId, NodeStatus};
use super::shard_router::DistributedShardRouter;
use super::{ClusterConfig, DiscoveryMethod};
use crate::error::{Result, VectorizerError};

/// Cluster manager for membership and state coordination
#[derive(Debug, Clone)]
pub struct ClusterManager {
    /// This node's ID
    local_node_id: NodeId,
    /// Cluster configuration
    config: ClusterConfig,
    /// Cluster nodes (node_id -> ClusterNode)
    nodes: Arc<RwLock<HashMap<NodeId, ClusterNode>>>,
    /// Distributed shard router
    shard_router: Arc<DistributedShardRouter>,
    /// Heartbeat timeout
    heartbeat_timeout: Duration,
}

impl ClusterManager {
    /// Create a new cluster manager
    pub fn new(config: ClusterConfig) -> Result<Self> {
        // Generate or use configured node ID
        let node_id = config
            .node_id
            .clone()
            .unwrap_or_else(|| format!("node-{}", uuid::Uuid::new_v4()));

        let local_node_id = NodeId::new(node_id.clone());

        info!("Initializing cluster manager with node ID: {}", node_id);

        let manager = Self {
            local_node_id: local_node_id.clone(),
            config: config.clone(),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            shard_router: Arc::new(DistributedShardRouter::new(100)), // 100 virtual nodes per shard
            heartbeat_timeout: Duration::from_millis(config.timeout_ms),
        };

        // Add local node
        manager.add_local_node()?;

        // Discover and add other nodes if cluster is enabled
        if config.enabled {
            manager.discover_nodes()?;
        }

        Ok(manager)
    }

    /// Add local node to cluster
    fn add_local_node(&self) -> Result<()> {
        let mut nodes = self.nodes.write();

        // For now, we'll need the server address from config or environment
        // This is a placeholder - will be properly initialized when server starts
        let mut local_node = ClusterNode::new(
            self.local_node_id.clone(),
            "127.0.0.1".to_string(), // Will be set properly during server initialization
            0,                       // Will be set properly during server initialization
        );
        local_node.mark_active();

        nodes.insert(self.local_node_id.clone(), local_node);
        info!("Added local node {} to cluster", self.local_node_id);

        Ok(())
    }

    /// Discover cluster nodes based on discovery method
    fn discover_nodes(&self) -> Result<()> {
        match self.config.discovery {
            DiscoveryMethod::Static => self.discover_static_nodes(),
            DiscoveryMethod::Dns => {
                warn!("DNS discovery not yet implemented");
                Ok(())
            }
            DiscoveryMethod::ServiceRegistry => {
                warn!("Service registry discovery not yet implemented");
                Ok(())
            }
        }
    }

    /// Discover nodes from static configuration
    fn discover_static_nodes(&self) -> Result<()> {
        let mut nodes = self.nodes.write();

        for server_config in &self.config.servers {
            // Skip local node
            if server_config.id == self.local_node_id.as_str() {
                continue;
            }

            let node_id = NodeId::new(server_config.id.clone());
            let node = ClusterNode::new(
                node_id.clone(),
                server_config.address.clone(),
                server_config.grpc_port,
            );

            nodes.insert(node_id.clone(), node);
            debug!("Discovered node {} from static config", node_id);
        }

        info!("Discovered {} nodes from static configuration", nodes.len());
        Ok(())
    }

    /// Get local node ID
    pub fn local_node_id(&self) -> &NodeId {
        &self.local_node_id
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &NodeId) -> Option<ClusterNode> {
        let nodes = self.nodes.read();
        nodes.get(node_id).cloned()
    }

    /// Update a node's status
    pub fn update_node_status(&self, node_id: &NodeId, status: NodeStatus) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.get_mut(node_id) {
            node.status = status;
            match status {
                NodeStatus::Active => {
                    node.update_heartbeat();
                }
                _ => {}
            }
        }
    }

    /// Mark a node as unavailable
    pub fn mark_node_unavailable(&self, node_id: &NodeId) {
        self.update_node_status(node_id, NodeStatus::Unavailable);
    }

    /// Mark a node as active
    pub fn mark_node_active(&self, node_id: &NodeId) {
        self.update_node_status(node_id, NodeStatus::Active);
    }

    /// Get all nodes
    pub fn get_nodes(&self) -> Vec<ClusterNode> {
        let nodes = self.nodes.read();
        nodes.values().cloned().collect()
    }

    /// Get active nodes only
    pub fn get_active_nodes(&self) -> Vec<ClusterNode> {
        let nodes = self.nodes.read();
        nodes
            .values()
            .filter(|n| n.status == NodeStatus::Active)
            .cloned()
            .collect()
    }

    /// Add a node to the cluster
    pub fn add_node(&self, node: ClusterNode) {
        let mut nodes = self.nodes.write();
        nodes.insert(node.id.clone(), node.clone());
        info!("Added node {} to cluster", node.id);
    }

    /// Remove a node from the cluster
    pub fn remove_node(&self, node_id: &NodeId) -> Option<ClusterNode> {
        let mut nodes = self.nodes.write();
        let node = nodes.remove(node_id);
        if node.is_some() {
            info!("Removed node {} from cluster", node_id);
        }
        node
    }

    /// Update node heartbeat
    pub fn update_node_heartbeat(&self, node_id: &NodeId) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.get_mut(node_id) {
            node.update_heartbeat();
        }
    }

    /// Check node health and mark unavailable if needed
    pub fn check_node_health(&self, node_id: &NodeId) -> bool {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.get_mut(node_id) {
            if !node.is_healthy(self.heartbeat_timeout) {
                node.mark_unavailable();
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Get the distributed shard router
    pub fn shard_router(&self) -> Arc<DistributedShardRouter> {
        self.shard_router.clone()
    }

    /// Check if cluster is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}
