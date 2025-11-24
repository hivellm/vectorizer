//! Distributed shard router for routing shards across cluster nodes

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{debug, info, warn};

use super::node::{ClusterNode, NodeId};
use crate::db::sharding::ShardId;

/// Distributed shard router that maps shards to cluster nodes
#[derive(Debug, Clone)]
pub struct DistributedShardRouter {
    /// Consistent hash ring: hash -> (shard_id, node_id)
    ring: Arc<RwLock<BTreeMap<u64, (ShardId, NodeId)>>>,
    /// Shard to node mapping (for quick lookup)
    shard_to_node: Arc<RwLock<HashMap<ShardId, NodeId>>>,
    /// Node to shards mapping
    node_to_shards: Arc<RwLock<HashMap<NodeId, HashSet<ShardId>>>>,
    /// Virtual nodes per shard (for better distribution)
    virtual_nodes_per_shard: usize,
}

impl DistributedShardRouter {
    /// Create a new distributed shard router
    pub fn new(virtual_nodes_per_shard: usize) -> Self {
        Self {
            ring: Arc::new(RwLock::new(BTreeMap::new())),
            shard_to_node: Arc::new(RwLock::new(HashMap::new())),
            node_to_shards: Arc::new(RwLock::new(HashMap::new())),
            virtual_nodes_per_shard,
        }
    }

    /// Get the node ID for a shard
    pub fn get_node_for_shard(&self, shard_id: &ShardId) -> Option<NodeId> {
        let shard_to_node = self.shard_to_node.read();
        shard_to_node.get(shard_id).cloned()
    }

    /// Get the node ID for a vector (via shard routing)
    pub fn get_node_for_vector(&self, vector_id: &str) -> Option<NodeId> {
        // Hash vector ID to determine shard
        let shard_id = self.get_shard_for_vector(vector_id);
        self.get_node_for_shard(&shard_id)
    }

    /// Get the shard for a vector ID (using consistent hashing)
    pub fn get_shard_for_vector(&self, vector_id: &str) -> ShardId {
        let hash = Self::hash_vector_id(vector_id);
        let ring = self.ring.read();

        // Find the first shard with hash >= vector hash (circular)
        match ring.range(hash..).next() {
            Some((_, (shard_id, _))) => *shard_id,
            None => {
                // Wrap around to the beginning of the ring
                ring.iter()
                    .next()
                    .map(|(_, (shard_id, _))| *shard_id)
                    .unwrap_or_else(|| ShardId::new(0))
            }
        }
    }

    /// Assign a shard to a node
    pub fn assign_shard(&self, shard_id: ShardId, node_id: NodeId) {
        let mut shard_to_node = self.shard_to_node.write();
        let mut node_to_shards = self.node_to_shards.write();
        let mut ring = self.ring.write();

        // Remove old assignment if exists
        if let Some(old_node) = shard_to_node.get(&shard_id) {
            if let Some(shards) = node_to_shards.get_mut(old_node) {
                shards.remove(&shard_id);
            }
            // Remove from ring
            ring.retain(|_, (s, _)| *s != shard_id);
        }

        // Add new assignment
        shard_to_node.insert(shard_id, node_id.clone());
        node_to_shards
            .entry(node_id.clone())
            .or_insert_with(HashSet::new)
            .insert(shard_id);

        // Add virtual nodes to ring
        for i in 0..self.virtual_nodes_per_shard {
            let hash = Self::hash_shard_vnode(&shard_id, i);
            ring.insert(hash, (shard_id, node_id.clone()));
        }

        info!("Assigned shard {} to node {}", shard_id.as_u32(), node_id);
    }

    /// Remove shard assignment
    pub fn remove_shard(&self, shard_id: &ShardId) -> Option<NodeId> {
        let mut shard_to_node = self.shard_to_node.write();
        let mut node_to_shards = self.node_to_shards.write();
        let mut ring = self.ring.write();

        if let Some(node_id) = shard_to_node.remove(shard_id) {
            if let Some(shards) = node_to_shards.get_mut(&node_id) {
                shards.remove(shard_id);
            }
            // Remove from ring
            ring.retain(|_, (s, _)| *s != *shard_id);
            Some(node_id)
        } else {
            None
        }
    }

    /// Get all shards for a node
    pub fn get_shards_for_node(&self, node_id: &NodeId) -> Vec<ShardId> {
        let node_to_shards = self.node_to_shards.read();
        node_to_shards
            .get(node_id)
            .map(|shards| shards.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get all nodes that have shards
    pub fn get_nodes(&self) -> Vec<NodeId> {
        let node_to_shards = self.node_to_shards.read();
        node_to_shards.keys().cloned().collect()
    }

    /// Rebalance shards across nodes (simple round-robin for now)
    pub fn rebalance(&self, shard_ids: &[ShardId], node_ids: &[NodeId]) {
        if node_ids.is_empty() {
            warn!("Cannot rebalance: no nodes available");
            return;
        }

        info!(
            "Rebalancing {} shards across {} nodes",
            shard_ids.len(),
            node_ids.len()
        );

        // First, remove shards from nodes that are no longer in the active list
        let active_node_set: std::collections::HashSet<NodeId> = node_ids.iter().cloned().collect();
        let shard_to_node = self.shard_to_node.read();
        let shards_to_remove: Vec<ShardId> = shard_to_node
            .iter()
            .filter(|(_, node_id)| !active_node_set.contains(node_id))
            .map(|(shard_id, _)| *shard_id)
            .collect();
        drop(shard_to_node);

        for shard_id in &shards_to_remove {
            self.remove_shard(shard_id);
        }

        // Simple round-robin assignment
        for (i, shard_id) in shard_ids.iter().enumerate() {
            let node_index = i % node_ids.len();
            let node_id = &node_ids[node_index];
            self.assign_shard(*shard_id, node_id.clone());
        }

        info!("Rebalancing complete");
    }

    /// Migrate a shard from one node to another
    /// Returns the previous node ID if the shard was assigned
    pub fn migrate_shard(
        &self,
        shard_id: ShardId,
        from_node: &NodeId,
        to_node: &NodeId,
    ) -> Option<NodeId> {
        let mut shard_to_node = self.shard_to_node.write();
        let mut node_to_shards = self.node_to_shards.write();

        // Verify the shard is currently on from_node
        if shard_to_node.get(&shard_id) != Some(from_node) {
            warn!(
                "Shard {} is not on node {}, cannot migrate",
                shard_id.0, from_node
            );
            return None;
        }

        // Remove shard from source node
        if let Some(shards) = node_to_shards.get_mut(from_node) {
            shards.retain(|&s| s != shard_id);
        }

        // Add shard to target node
        node_to_shards
            .entry(to_node.clone())
            .or_insert_with(std::collections::HashSet::new)
            .insert(shard_id);

        // Update shard-to-node mapping
        let previous_node = shard_to_node.insert(shard_id, to_node.clone());

        info!(
            "Migrated shard {} from node {} to node {}",
            shard_id.0, from_node, to_node
        );

        previous_node
    }

    /// Get migration plan for rebalancing shards
    /// Returns a list of (shard_id, from_node, to_node) tuples
    pub fn calculate_migration_plan(
        &self,
        shard_ids: &[ShardId],
        node_ids: &[NodeId],
    ) -> Vec<(ShardId, NodeId, NodeId)> {
        if node_ids.is_empty() {
            return Vec::new();
        }

        let target_shards_per_node = shard_ids.len() / node_ids.len();
        let mut migrations = Vec::new();

        // Count current shards per node
        let mut node_shard_counts: std::collections::HashMap<NodeId, usize> =
            node_ids.iter().map(|n| (n.clone(), 0)).collect();

        let shard_to_node = self.shard_to_node.read();
        for shard_id in shard_ids {
            if let Some(node_id) = shard_to_node.get(shard_id) {
                *node_shard_counts.entry(node_id.clone()).or_insert(0) += 1;
            }
        }

        // Find nodes that need to give up shards (overloaded)
        let mut overloaded_nodes: Vec<(NodeId, usize)> = node_shard_counts
            .iter()
            .filter(|&(_, count)| *count > target_shards_per_node)
            .map(|(node, &count)| (node.clone(), count - target_shards_per_node))
            .collect();
        overloaded_nodes.sort_by(|a, b| b.1.cmp(&a.1));

        // Find nodes that need shards (underloaded)
        let mut underloaded_nodes: Vec<(NodeId, usize)> = node_shard_counts
            .iter()
            .filter(|&(_, count)| *count < target_shards_per_node)
            .map(|(node, &count)| (node.clone(), target_shards_per_node - count))
            .collect();
        underloaded_nodes.sort_by(|a, b| b.1.cmp(&a.1));

        // Create migration plan
        let shard_to_node = self.shard_to_node.read();
        for (from_node, excess) in overloaded_nodes {
            let mut shards_to_migrate: Vec<ShardId> = shard_ids
                .iter()
                .filter(|&&shard_id| shard_to_node.get(&shard_id) == Some(&from_node))
                .take(excess)
                .copied()
                .collect();

            for (to_node, deficit) in &mut underloaded_nodes {
                if shards_to_migrate.is_empty() {
                    break;
                }

                let migrate_count = (*deficit).min(shards_to_migrate.len());
                for _ in 0..migrate_count {
                    if let Some(shard_id) = shards_to_migrate.pop() {
                        migrations.push((shard_id, from_node.clone(), to_node.clone()));
                        *deficit -= 1;
                    }
                }
            }
        }

        migrations
    }

    /// Hash a shard virtual node
    fn hash_shard_vnode(shard_id: &ShardId, vnode_index: usize) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        shard_id.as_u32().hash(&mut hasher);
        vnode_index.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash a vector ID
    fn hash_vector_id(vector_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        vector_id.hash(&mut hasher);
        hasher.finish()
    }
}
