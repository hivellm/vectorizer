//! Production [`ShardTopology`] implementation (phase41 §3.1).
//!
//! Bridges the db-layer topology trait onto the concrete cluster
//! types: `DistributedShardRouter` for shard placement and
//! `ClusterManager` for node identity/addresses. Node ids cross the
//! trait boundary as plain strings (`NodeId` is a `String` newtype).

use std::sync::Arc;

use super::manager::ClusterManager;
use super::node::NodeId;
use super::shard_router::DistributedShardRouter;
use crate::db::shard_topology::ShardTopology;
use crate::db::sharding::ShardId;

/// [`ShardTopology`] over a live cluster.
pub struct ClusterShardTopology {
    router: Arc<DistributedShardRouter>,
    manager: Arc<ClusterManager>,
}

impl ClusterShardTopology {
    /// Build a topology view over `manager` with a fresh consistent-hash
    /// router using `virtual_nodes_per_shard` (from the collection's
    /// sharding config).
    pub fn new(manager: Arc<ClusterManager>, virtual_nodes_per_shard: usize) -> Self {
        Self {
            router: Arc::new(DistributedShardRouter::new(virtual_nodes_per_shard)),
            manager,
        }
    }

    /// Wrap existing router/manager instances (cluster-internal wiring).
    pub fn from_parts(router: Arc<DistributedShardRouter>, manager: Arc<ClusterManager>) -> Self {
        Self { router, manager }
    }
}

impl ShardTopology for ClusterShardTopology {
    fn local_node_id(&self) -> String {
        self.manager.local_node_id().0.clone()
    }

    fn node_grpc_address(&self, node_id: &str) -> Option<String> {
        self.manager
            .get_node(&NodeId(node_id.to_string()))
            .map(|n| n.grpc_address())
    }

    fn active_node_ids(&self) -> Vec<String> {
        self.manager
            .get_active_nodes()
            .into_iter()
            .map(|n| n.id.0)
            .collect()
    }

    fn node_for_shard(&self, shard: &ShardId) -> Option<String> {
        self.router.get_node_for_shard(shard).map(|n| n.0)
    }

    fn shard_for_vector(&self, vector_id: &str) -> ShardId {
        self.router.get_shard_for_vector(vector_id)
    }

    fn shards_for_node(&self, node_id: &str) -> Vec<ShardId> {
        self.router
            .get_shards_for_node(&NodeId(node_id.to_string()))
    }

    fn all_shards(&self) -> Vec<ShardId> {
        self.router.get_all_shards()
    }

    fn rebalance(&self, shards: &[ShardId], nodes: &[String]) {
        let node_ids: Vec<NodeId> = nodes.iter().map(|n| NodeId(n.clone())).collect();
        self.router.rebalance(shards, &node_ids);
    }
}
