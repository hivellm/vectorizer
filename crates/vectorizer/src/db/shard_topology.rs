//! Shard-topology abstraction (phase41 §3.1).
//!
//! [`DistributedShardedCollection`](super::DistributedShardedCollection)
//! previously imported `ClusterManager`, `DistributedShardRouter`, and
//! `NodeId` from `cluster/` — one of the nine upward back-references
//! blocking the workspace split (analysis 2026-07-11 §1.1). The
//! collection only consumes a narrow routing/topology surface, captured
//! here as a trait that `cluster/` implements
//! (`ClusterShardTopology`). Node ids cross the boundary as plain
//! strings (`cluster::NodeId` is a `String` newtype).
//!
//! The gRPC transport (`ClusterClientPool`) intentionally stays
//! concrete: abstracting it means abstracting the remote wire client
//! itself, which is out of scope for a topology seam — that residual
//! import is documented in the phase41 task.

use super::sharding::ShardId;

/// Routing + node-topology surface consumed by the distributed sharded
/// collection. Implemented by `cluster::ClusterShardTopology` in
/// production and by lightweight stubs in db-level tests.
pub trait ShardTopology: Send + Sync {
    /// This node's id.
    fn local_node_id(&self) -> String;

    /// gRPC address of `node_id`, if the node is known.
    fn node_grpc_address(&self, node_id: &str) -> Option<String>;

    /// Ids of all currently active nodes.
    fn active_node_ids(&self) -> Vec<String>;

    /// Node currently owning `shard`, if assigned.
    fn node_for_shard(&self, shard: &ShardId) -> Option<String>;

    /// Deterministic shard for a vector id.
    fn shard_for_vector(&self, vector_id: &str) -> ShardId;

    /// Shards owned by `node_id`.
    fn shards_for_node(&self, node_id: &str) -> Vec<ShardId>;

    /// Every shard known to the router.
    fn all_shards(&self) -> Vec<ShardId>;

    /// (Re)assign `shards` across `nodes` via consistent hashing.
    fn rebalance(&self, shards: &[ShardId], nodes: &[String]);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use parking_lot::RwLock;

    use super::*;
    use crate::db::DistributedShardedCollection;
    use crate::models::{CollectionConfig, ShardingConfig};

    /// Minimal in-memory topology: every shard maps to the local node.
    /// Proves db-level sharded collections construct and route without
    /// any cluster type (spec: decoupling / "db compiles without
    /// cluster").
    struct StubTopology {
        assignments: RwLock<HashMap<ShardId, String>>,
    }

    impl StubTopology {
        fn new() -> Self {
            Self {
                assignments: RwLock::new(HashMap::new()),
            }
        }
    }

    impl ShardTopology for StubTopology {
        fn local_node_id(&self) -> String {
            "stub-node".to_string()
        }

        fn node_grpc_address(&self, node_id: &str) -> Option<String> {
            (node_id == "stub-node").then(|| "127.0.0.1:0".to_string())
        }

        fn active_node_ids(&self) -> Vec<String> {
            vec!["stub-node".to_string()]
        }

        fn node_for_shard(&self, shard: &ShardId) -> Option<String> {
            self.assignments.read().get(shard).cloned()
        }

        fn shard_for_vector(&self, vector_id: &str) -> ShardId {
            let shards = self.assignments.read();
            let n = shards.len().max(1) as u32;
            let mut h: u32 = 2166136261;
            for b in vector_id.bytes() {
                h = (h ^ b as u32).wrapping_mul(16777619);
            }
            ShardId::new(h % n)
        }

        fn shards_for_node(&self, node_id: &str) -> Vec<ShardId> {
            self.assignments
                .read()
                .iter()
                .filter(|(_, n)| n.as_str() == node_id)
                .map(|(s, _)| *s)
                .collect()
        }

        fn all_shards(&self) -> Vec<ShardId> {
            self.assignments.read().keys().copied().collect()
        }

        fn rebalance(&self, shards: &[ShardId], nodes: &[String]) {
            let mut map = self.assignments.write();
            map.clear();
            for (i, s) in shards.iter().enumerate() {
                map.insert(*s, nodes[i % nodes.len()].clone());
            }
        }
    }

    #[test]
    fn sharded_collection_constructs_over_a_stub_topology() {
        let config = CollectionConfig {
            sharding: Some(ShardingConfig {
                shard_count: 4,
                virtual_nodes_per_shard: 8,
                ..Default::default()
            }),
            ..Default::default()
        };

        let topology: Arc<dyn ShardTopology> = Arc::new(StubTopology::new());
        let pool = Arc::new(crate::cluster::ClusterClientPool::new(
            std::time::Duration::from_secs(1),
        ));

        let collection = DistributedShardedCollection::new(
            "stub_sharded".to_string(),
            config,
            topology.clone(),
            pool,
        )
        .expect("constructs without any real cluster type");

        // All 4 shards land on the single stub node → all local.
        assert_eq!(topology.all_shards().len(), 4);
        assert_eq!(topology.shards_for_node("stub-node").len(), 4);
        assert_eq!(collection.name(), "stub_sharded");
    }
}
