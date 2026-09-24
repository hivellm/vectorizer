//! HA (High Availability) manager for Raft-driven role transitions
//!
//! Manages the lifecycle of MasterNode and ReplicaNode instances as this
//! node's role changes between Leader and Follower in the Raft cluster.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{error, info, warn};

use super::leader_router::LeaderRouter;
use crate::db::VectorStore;
use crate::replication::{MasterNode, ReplicaNode, ReplicationConfig};

/// Manages HA role transitions and replication lifecycle.
///
/// When notified by Raft callbacks, `HaManager` starts or stops the
/// appropriate replication node (`MasterNode` or `ReplicaNode`) so that
/// the data-plane always reflects the current consensus role.
pub struct HaManager {
    pub leader_router: Arc<LeaderRouter>,
    store: Arc<VectorStore>,
    /// Active master node (present only when this node is leader)
    master_node: Arc<RwLock<Option<Arc<MasterNode>>>>,
    /// Active replica node (present only when this node is follower)
    replica_node: Arc<RwLock<Option<Arc<ReplicaNode>>>>,
    /// Base replication configuration (role is overridden on transition)
    repl_config: ReplicationConfig,
}

impl HaManager {
    /// Create a new `HaManager` for the given `local_node_id`.
    pub fn new(
        local_node_id: u64,
        store: Arc<VectorStore>,
        repl_config: ReplicationConfig,
    ) -> Self {
        Self {
            leader_router: Arc::new(LeaderRouter::new(local_node_id)),
            store,
            master_node: Arc::new(RwLock::new(None)),
            replica_node: Arc::new(RwLock::new(None)),
            repl_config,
        }
    }

    /// Called when this node wins a Raft election and becomes leader.
    ///
    /// Stops any running `ReplicaNode` and starts a `MasterNode`.
    pub async fn on_become_leader(&self) {
        info!("This node is now LEADER - starting MasterNode");

        // `shutdown` is required: the replica's reconnect loop holds its own
        // Arc, so dropping ours would leave it running.
        let old_replica = self.replica_node.write().take();
        if let Some(replica) = old_replica {
            info!("Stopping ReplicaNode (transitioning to Leader)");
            replica.shutdown();
        }

        if self.master_node.read().is_some() {
            info!("MasterNode already running");
            return;
        }

        let mut config = self.repl_config.clone();
        config.role = crate::replication::NodeRole::Master;

        let master = match MasterNode::new(config, self.store.clone()) {
            Ok(master) => Arc::new(master),
            Err(e) => {
                error!("Failed to start MasterNode: {}", e);
                return;
            }
        };
        // `start` only binds and spawns its tasks, so await it here: a bind
        // failure must not leave behind a master that no replica can reach.
        match master.start().await {
            Ok(()) => {
                *self.master_node.write() = Some(master);
                info!("MasterNode started (accepting writes)");
            }
            Err(e) => {
                error!("MasterNode failed: {}", e);
            }
        }
    }

    /// Called when this node steps down and becomes a follower.
    ///
    /// Stops any running `MasterNode` and starts a `ReplicaNode` that
    /// connects to the new leader at `leader_addr`.
    pub async fn on_become_follower(&self, leader_addr: Option<String>) {
        info!("This node is now FOLLOWER");

        // Take the Arc out before awaiting so the lock is not held across
        // the listener teardown.
        let old_master = self.master_node.write().take();
        if let Some(master) = old_master {
            info!("Stopping MasterNode (transitioning to Follower)");
            master.shutdown().await;
        }

        // The leader may have changed while this node was already a follower.
        let old_replica = self.replica_node.write().take();
        if let Some(replica) = old_replica {
            info!("Stopping ReplicaNode for the previous leader");
            replica.shutdown();
        }

        // Start replica connecting to leader
        if let Some(addr) = leader_addr {
            let mut config = self.repl_config.clone();
            config.role = crate::replication::NodeRole::Replica;
            config.master_address_raw = Some(addr.clone());
            config.master_address = addr.parse().ok();

            let replica = Arc::new(ReplicaNode::new(config, self.store.clone()));
            let replica_clone = replica.clone();
            tokio::spawn(async move {
                if let Err(e) = replica_clone.start().await {
                    error!("ReplicaNode failed: {}", e);
                }
            });
            *self.replica_node.write() = Some(replica);
            info!("ReplicaNode started (connecting to leader at {})", addr);
        } else {
            warn!("No leader address available, ReplicaNode not started");
        }
    }

    /// Returns a reference to the active `MasterNode`, if this node is leader.
    pub fn master_node(&self) -> Option<Arc<MasterNode>> {
        self.master_node.read().clone()
    }

    /// Returns a reference to the active `ReplicaNode`, if this node is follower.
    pub fn replica_node(&self) -> Option<Arc<ReplicaNode>> {
        self.replica_node.read().clone()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::net::SocketAddr;
    use std::time::Duration;

    use super::*;
    use crate::replication::{CollectionConfigData, VectorOperation};

    /// Reserve a free loopback address for a replication listener.
    fn free_loopback_addr() -> SocketAddr {
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
    }

    /// 1s heartbeat/reconnect so disconnects and reconnects surface fast.
    fn fast_repl_config(bind_address: Option<SocketAddr>) -> ReplicationConfig {
        ReplicationConfig {
            bind_address,
            heartbeat_interval: 1,
            reconnect_interval: 1,
            wal_enabled: false,
            ..ReplicationConfig::default()
        }
    }

    /// Poll `cond` every 100ms until it holds or `timeout` elapses.
    async fn eventually(timeout: Duration, mut cond: impl FnMut() -> bool) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        while tokio::time::Instant::now() < deadline {
            if cond() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        cond()
    }

    fn connected_replicas(ha: &HaManager) -> usize {
        ha.master_node().map_or(0, |m| m.get_replicas().len())
    }

    /// A node that loses and regains leadership without restarting must
    /// keep replicating. The first term's MasterNode used to keep the
    /// replication port, so the second one failed with "Address in use"
    /// and replicas attached to the orphaned listener.
    #[tokio::test]
    async fn test_leader_follower_leader_flip_keeps_replicating() {
        let leader_addr = free_loopback_addr();
        let store_b = Arc::new(VectorStore::new());
        let ha_a = HaManager::new(
            1,
            Arc::new(VectorStore::new()),
            fast_repl_config(Some(leader_addr)),
        );
        let ha_b = HaManager::new(2, store_b.clone(), fast_repl_config(None));

        ha_a.on_become_leader().await;
        ha_b.on_become_follower(Some(leader_addr.to_string())).await;
        assert!(
            eventually(Duration::from_secs(10), || connected_replicas(&ha_a) == 1).await,
            "replica never connected to the first-term master"
        );

        ha_a.on_become_follower(None).await;
        ha_a.on_become_leader().await;
        assert!(
            eventually(Duration::from_secs(15), || connected_replicas(&ha_a) == 1).await,
            "replica never reconnected to the second-term master"
        );

        ha_a.master_node().expect("node A is leader").replicate(
            VectorOperation::CreateCollection {
                name: "after-flip".to_string(),
                config: CollectionConfigData {
                    dimension: 4,
                    metric: "cosine".to_string(),
                },
                owner_id: None,
            },
        );
        assert!(
            eventually(Duration::from_secs(10), || store_b
                .list_collections()
                .contains(&"after-flip".to_string()))
            .await,
            "a write on the re-elected leader never reached the follower"
        );
    }

    /// A follower that switches leaders, then wins an election itself, must
    /// disconnect from every previous master instead of leaving the old
    /// reconnect loop running alongside the new one.
    #[tokio::test]
    async fn test_role_transitions_stop_previous_replica() {
        let addr_1 = free_loopback_addr();
        let addr_2 = free_loopback_addr();
        let master_1 =
            MasterNode::new(fast_repl_config(Some(addr_1)), Arc::new(VectorStore::new())).unwrap();
        let master_2 =
            MasterNode::new(fast_repl_config(Some(addr_2)), Arc::new(VectorStore::new())).unwrap();
        master_1.start().await.unwrap();
        master_2.start().await.unwrap();

        let ha = HaManager::new(3, Arc::new(VectorStore::new()), fast_repl_config(None));

        ha.on_become_follower(Some(addr_1.to_string())).await;
        assert!(
            eventually(Duration::from_secs(10), || master_1.get_replicas().len()
                == 1)
            .await,
            "follower never connected to the first leader"
        );

        ha.on_become_follower(Some(addr_2.to_string())).await;
        assert!(
            eventually(Duration::from_secs(10), || master_2.get_replicas().len()
                == 1)
            .await,
            "follower never connected to the second leader"
        );
        assert!(
            eventually(Duration::from_secs(10), || master_1
                .get_replicas()
                .is_empty())
            .await,
            "the replica for the previous leader is still connected"
        );

        ha.on_become_leader().await;
        assert!(
            eventually(Duration::from_secs(10), || master_2
                .get_replicas()
                .is_empty())
            .await,
            "the replica kept running after the node became leader"
        );
    }
}
