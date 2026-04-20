//! Cluster state synchronization

use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::collection_sync::CollectionSynchronizer;
use super::manager::ClusterManager;
use super::node::{ClusterNode, NodeId, NodeStatus};
use super::server_client::ClusterClientPool;
use crate::db::VectorStore;

// Cluster proto types live in the `vectorizer-protocol` crate after
// phase4_split-vectorizer-workspace sub-phase 2.
use crate::grpc::cluster as cluster_proto;

/// Cluster state synchronizer
#[derive(Debug, Clone)]
pub struct ClusterStateSynchronizer {
    /// Cluster manager
    manager: Arc<ClusterManager>,
    /// Client pool for gRPC communication
    client_pool: Arc<ClusterClientPool>,
    /// Vector store used for collection consistency repair
    store: Arc<VectorStore>,
    /// Synchronization interval
    sync_interval: Duration,
    /// Whether synchronization is running
    running: Arc<RwLock<bool>>,
}

impl ClusterStateSynchronizer {
    /// Create a new cluster state synchronizer
    pub fn new(
        manager: Arc<ClusterManager>,
        client_pool: Arc<ClusterClientPool>,
        store: Arc<VectorStore>,
        sync_interval: Duration,
    ) -> Self {
        Self {
            manager,
            client_pool,
            store,
            sync_interval,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start state synchronization
    pub async fn start(&self) {
        let mut running = self.running.write();
        if *running {
            warn!("Cluster state synchronizer is already running");
            return;
        }
        *running = true;
        drop(running);

        info!(
            "Starting cluster state synchronizer with interval: {:?}",
            self.sync_interval
        );

        let sync = self.clone();
        let sync_interval = self.sync_interval;
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = interval(sync_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                // Check if still running
                {
                    let is_running = running.read();
                    if !*is_running {
                        info!("Cluster state synchronizer stopped");
                        break;
                    }
                    drop(is_running); // Explicitly drop to ensure Send
                }

                // Perform synchronization
                if let Err(e) = sync.sync_state().await {
                    error!("Failed to synchronize cluster state: {}", e);
                }
            }
        });
    }

    /// Stop state synchronization
    pub fn stop(&self) {
        let mut running = self.running.write();
        *running = false;
        info!("Stopping cluster state synchronizer");
    }

    /// Synchronize cluster state with other nodes
    async fn sync_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Synchronizing cluster state");

        // Get all nodes
        let nodes = self.manager.get_nodes();
        let local_node_id = self.manager.local_node_id().clone();

        // Exchange cluster state with other nodes
        for node in &nodes {
            if node.id == local_node_id {
                continue; // Skip local node
            }

            // Try to get cluster state from remote node
            match self
                .client_pool
                .get_client(&node.id, &node.grpc_address())
                .await
            {
                Ok(client) => {
                    match client.get_cluster_state().await {
                        Ok(remote_state) => {
                            debug!("Received cluster state from node {}", node.id);

                            // Update heartbeat - node is reachable
                            self.manager.update_node_heartbeat(&node.id);

                            // Merge remote nodes into local cluster state
                            for proto_node in &remote_state.nodes {
                                let node_id = NodeId::new(proto_node.id.clone());

                                // Skip if it's the local node
                                if node_id == local_node_id {
                                    continue;
                                }

                                // Convert proto node to ClusterNode
                                let status = match proto_node.status {
                                    x if x == cluster_proto::NodeStatus::Active as i32 => {
                                        NodeStatus::Active
                                    }
                                    x if x == cluster_proto::NodeStatus::Joining as i32 => {
                                        NodeStatus::Joining
                                    }
                                    x if x == cluster_proto::NodeStatus::Leaving as i32 => {
                                        NodeStatus::Leaving
                                    }
                                    _ => NodeStatus::Unavailable,
                                };

                                let mut cluster_node = ClusterNode::new(
                                    node_id.clone(),
                                    proto_node.address.clone(),
                                    proto_node.grpc_port as u16,
                                );

                                match status {
                                    NodeStatus::Active => cluster_node.mark_active(),
                                    NodeStatus::Joining => {
                                        cluster_node.status = NodeStatus::Joining;
                                    }
                                    NodeStatus::Leaving => {
                                        cluster_node.status = NodeStatus::Leaving;
                                    }
                                    NodeStatus::Unavailable => cluster_node.mark_unavailable(),
                                }

                                // Add shards
                                for shard_id in &proto_node.shards {
                                    cluster_node
                                        .add_shard(crate::db::sharding::ShardId::new(*shard_id));
                                }

                                // Add or update node in cluster
                                self.manager.add_node(cluster_node);
                            }

                            // Update shard assignments from remote state using epoch-based
                            // conflict resolution. Higher epoch wins; ties are broken
                            // lexicographically by node ID (smaller ID keeps its assignment),
                            // mirroring Redis configEpoch semantics.
                            let shard_router = self.manager.shard_router();
                            for (shard_id_u32, node_id_str) in &remote_state.shard_to_node {
                                let shard_id = crate::db::sharding::ShardId::new(*shard_id_u32);
                                let assigned_node_id = NodeId::new(node_id_str.clone());

                                match shard_router.get_node_for_shard(&shard_id) {
                                    None => {
                                        // No local assignment yet — adopt the remote one
                                        let remote_epoch = remote_state
                                            .shard_epochs
                                            .get(shard_id_u32)
                                            .copied()
                                            .unwrap_or(0);
                                        shard_router.apply_if_higher_epoch(
                                            shard_id,
                                            assigned_node_id,
                                            remote_epoch,
                                        );
                                    }
                                    Some(current_node) if current_node != assigned_node_id => {
                                        let local_epoch =
                                            shard_router.get_shard_epoch(&shard_id).unwrap_or(0);
                                        let remote_epoch = remote_state
                                            .shard_epochs
                                            .get(shard_id_u32)
                                            .copied()
                                            .unwrap_or(0);

                                        if remote_epoch > local_epoch {
                                            info!(
                                                "Shard {} conflict resolved: remote epoch {} > \
                                                 local epoch {}, adopting remote assignment to {}",
                                                shard_id_u32,
                                                remote_epoch,
                                                local_epoch,
                                                assigned_node_id
                                            );
                                            shard_router.apply_if_higher_epoch(
                                                shard_id,
                                                assigned_node_id,
                                                remote_epoch,
                                            );
                                        } else if remote_epoch == local_epoch {
                                            // Tie-break: the node with the lexicographically
                                            // smaller ID is authoritative and should eventually
                                            // increment its epoch. Until then both sides keep
                                            // their current assignment.
                                            if assigned_node_id.as_str() < current_node.as_str() {
                                                debug!(
                                                    "Shard {} epoch tie at {}: remote node '{}' \
                                                     has smaller ID — keeping local assignment \
                                                     on '{}' until remote increments",
                                                    shard_id_u32,
                                                    local_epoch,
                                                    assigned_node_id,
                                                    current_node
                                                );
                                            } else {
                                                debug!(
                                                    "Shard {} epoch tie at {}: keeping local \
                                                     assignment on '{}' (smaller ID wins)",
                                                    shard_id_u32, local_epoch, current_node
                                                );
                                            }
                                        } else {
                                            debug!(
                                                "Shard {} conflict resolved: local epoch {} > \
                                                 remote epoch {}, keeping local assignment on {}",
                                                shard_id_u32,
                                                local_epoch,
                                                remote_epoch,
                                                current_node
                                            );
                                        }
                                    }
                                    Some(_) => {
                                        // Assignments agree — nothing to do
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get cluster state from node {}: {}", node.id, e);
                            // Mark node as potentially unhealthy
                            if !self.manager.check_node_health(&node.id) {
                                warn!("Node {} marked as unhealthy", node.id);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get client for node {}: {}", node.id, e);
                    // Mark node as potentially unhealthy
                    if !self.manager.check_node_health(&node.id) {
                        warn!("Node {} marked as unhealthy", node.id);
                    }
                }
            }
        }

        debug!("Cluster state synchronization complete");

        // Repair any collections that are missing from remote nodes
        let collection_sync = CollectionSynchronizer::new(
            self.manager.clone(),
            self.client_pool.clone(),
            self.store.clone(),
        );
        match collection_sync.sync_collections().await {
            Ok(report) if report.repaired_count > 0 => {
                info!(
                    "Collection sync repaired {} collection(s) across cluster nodes",
                    report.repaired_count
                );
            }
            Ok(_) => {
                debug!("Collection sync complete: no repairs needed");
            }
            Err(e) => {
                error!("Collection sync encountered an error: {}", e);
            }
        }

        Ok(())
    }

    /// Broadcast cluster state to all nodes
    pub async fn broadcast_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Broadcasting cluster state to all nodes");

        let nodes = self.manager.get_nodes();
        let local_node_id = self.manager.local_node_id().clone();
        let shard_router = self.manager.shard_router();

        // Build local cluster state
        let mut shard_to_node = std::collections::HashMap::new();
        for node in &nodes {
            let shards = shard_router.get_shards_for_node(&node.id);
            for shard_id in shards {
                shard_to_node.insert(shard_id.as_u32(), node.id.as_str().to_string());
            }
        }

        // Build local node description for broadcast
        use super::server_client::BroadcastNode;

        let local_broadcast_node = nodes.iter().find(|n| n.id == local_node_id).map(|n| {
            let status = match n.status {
                NodeStatus::Active => cluster_proto::NodeStatus::Active as i32,
                NodeStatus::Joining => cluster_proto::NodeStatus::Joining as i32,
                NodeStatus::Leaving => cluster_proto::NodeStatus::Leaving as i32,
                NodeStatus::Unavailable => cluster_proto::NodeStatus::Unavailable as i32,
            };
            BroadcastNode {
                id: n.id.as_str().to_string(),
                address: n.address.clone(),
                grpc_port: n.grpc_port as u32,
                status,
                shards: n.shards.iter().map(|s| s.as_u32()).collect(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                capabilities: vec!["vector_search".to_string(), "sharding".to_string()],
            }
        });

        // Build shard assignment tuples: (shard_id, node_id, epoch)
        let all_shard_epochs = shard_router.get_all_shard_epochs();
        let shard_assignment_tuples: Vec<(u32, String, u64)> = shard_to_node
            .iter()
            .map(|(shard_id, node_id)| {
                let epoch = all_shard_epochs
                    .get(&crate::db::sharding::ShardId::new(*shard_id))
                    .copied()
                    .unwrap_or(0);
                (*shard_id, node_id.clone(), epoch)
            })
            .collect();

        // Broadcast to all remote nodes
        for node in &nodes {
            if node.id == local_node_id {
                continue; // Skip local node
            }

            match self
                .client_pool
                .get_client(&node.id, &node.grpc_address())
                .await
            {
                Ok(client) => {
                    match client
                        .broadcast_cluster_state(
                            local_broadcast_node.clone(),
                            &shard_assignment_tuples,
                        )
                        .await
                    {
                        Ok((success, message)) => {
                            debug!(
                                "Broadcast cluster state to node {}: success={}, message={}",
                                node.id, success, message
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to broadcast cluster state to node {}: {}",
                                node.id, e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to get client for broadcasting to node {}: {}",
                        node.id, e
                    );
                }
            }
        }

        debug!("Cluster state broadcast complete");
        Ok(())
    }

    /// Request cluster state from a specific node
    pub async fn request_state_from_node(
        &self,
        node_id: &NodeId,
    ) -> Result<ClusterNode, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Requesting cluster state from node {}", node_id);

        // Get node info to find address
        let node = self
            .manager
            .get_node(node_id)
            .ok_or_else(|| format!("Node {} not found locally", node_id))?;

        // Get client and request cluster state
        match self
            .client_pool
            .get_client(node_id, &node.grpc_address())
            .await
        {
            Ok(client) => {
                match client.get_cluster_state().await {
                    Ok(state) => {
                        // Find the requested node in the response
                        for proto_node in &state.nodes {
                            if proto_node.id == node_id.as_str() {
                                // Convert proto node to ClusterNode
                                let status = match proto_node.status {
                                    x if x == cluster_proto::NodeStatus::Active as i32 => {
                                        NodeStatus::Active
                                    }
                                    x if x == cluster_proto::NodeStatus::Joining as i32 => {
                                        NodeStatus::Joining
                                    }
                                    x if x == cluster_proto::NodeStatus::Leaving as i32 => {
                                        NodeStatus::Leaving
                                    }
                                    _ => NodeStatus::Unavailable,
                                };

                                let mut cluster_node = ClusterNode::new(
                                    NodeId::new(proto_node.id.clone()),
                                    proto_node.address.clone(),
                                    proto_node.grpc_port as u16,
                                );

                                match status {
                                    NodeStatus::Active => cluster_node.mark_active(),
                                    NodeStatus::Joining => {
                                        cluster_node.status = NodeStatus::Joining;
                                    }
                                    NodeStatus::Leaving => {
                                        cluster_node.status = NodeStatus::Leaving;
                                    }
                                    NodeStatus::Unavailable => cluster_node.mark_unavailable(),
                                }

                                // Add shards
                                for shard_id in &proto_node.shards {
                                    cluster_node
                                        .add_shard(crate::db::sharding::ShardId::new(*shard_id));
                                }

                                return Ok(cluster_node);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!(
                            "Failed to get cluster state from node {}: {}",
                            node_id, e
                        )
                        .into());
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to get client for node {}: {}", node_id, e).into());
            }
        }

        Err(format!("Node {} not found in cluster state", node_id).into())
    }
}
