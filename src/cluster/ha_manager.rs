//! HA (High Availability) manager for Raft-driven role transitions
//!
//! Manages the lifecycle of MasterNode and ReplicaNode instances as this
//! node's role changes between Leader and Follower in the Raft cluster.

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

        // Stop replica if running
        {
            let mut replica = self.replica_node.write();
            if replica.is_some() {
                info!("Stopping ReplicaNode (transitioning to Leader)");
                *replica = None; // Drop stops the replica
            }
        }

        // Start master
        let mut config = self.repl_config.clone();
        config.role = crate::replication::NodeRole::Master;

        match MasterNode::new(config, self.store.clone()) {
            Ok(master) => {
                let master = Arc::new(master);
                let master_clone = master.clone();
                tokio::spawn(async move {
                    if let Err(e) = master_clone.start().await {
                        error!("MasterNode failed: {}", e);
                    }
                });
                *self.master_node.write() = Some(master);
                info!("MasterNode started (accepting writes)");
            }
            Err(e) => {
                error!("Failed to start MasterNode: {}", e);
            }
        }
    }

    /// Called when this node steps down and becomes a follower.
    ///
    /// Stops any running `MasterNode` and starts a `ReplicaNode` that
    /// connects to the new leader at `leader_addr`.
    pub async fn on_become_follower(&self, leader_addr: Option<String>) {
        info!("This node is now FOLLOWER");

        // Stop master if running
        {
            let mut master = self.master_node.write();
            if master.is_some() {
                info!("Stopping MasterNode (transitioning to Follower)");
                *master = None;
            }
        }

        // Start replica connecting to leader
        if let Some(addr) = leader_addr {
            let mut config = self.repl_config.clone();
            config.role = crate::replication::NodeRole::Replica;
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
