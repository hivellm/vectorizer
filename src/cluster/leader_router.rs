//! Leader-aware routing for Raft-based HA cluster
//!
//! Tracks the current Raft leader and provides routing decisions for
//! write requests, enabling transparent redirect to the leader node.

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{info, warn};

/// Tracks the current Raft leader and provides routing decisions.
#[derive(Debug, Clone)]
pub struct LeaderRouter {
    /// Current leader node ID (None if no leader elected)
    current_leader_id: Arc<RwLock<Option<u64>>>,
    /// Current leader HTTP address (for redirects)
    current_leader_url: Arc<RwLock<Option<String>>>,
    /// This node's ID
    local_node_id: u64,
    /// This node's current role
    role: Arc<RwLock<NodeRole>>,
}

/// Role this node currently plays in the Raft cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    Leader,
    Follower,
    Learner,
    Candidate,
}

impl Default for NodeRole {
    fn default() -> Self {
        NodeRole::Follower
    }
}

impl LeaderRouter {
    /// Create a new `LeaderRouter` for the node identified by `local_node_id`.
    pub fn new(local_node_id: u64) -> Self {
        Self {
            current_leader_id: Arc::new(RwLock::new(None)),
            current_leader_url: Arc::new(RwLock::new(None)),
            local_node_id,
            role: Arc::new(RwLock::new(NodeRole::Follower)),
        }
    }

    /// Update leader info. Called by Raft callbacks when a new leader is elected.
    pub fn set_leader(&self, leader_id: u64, leader_url: String) {
        *self.current_leader_id.write() = Some(leader_id);
        *self.current_leader_url.write() = Some(leader_url.clone());

        if leader_id == self.local_node_id {
            info!("This node is now the LEADER (id={})", leader_id);
            *self.role.write() = NodeRole::Leader;
        } else {
            info!("Leader changed to node {} (url: {})", leader_id, leader_url);
            *self.role.write() = NodeRole::Follower;
        }
    }

    /// Clear leader state. Called when no leader is currently elected.
    pub fn clear_leader(&self) {
        *self.current_leader_id.write() = None;
        *self.current_leader_url.write() = None;
        *self.role.write() = NodeRole::Candidate;
        warn!("No leader elected – node entering Candidate state");
    }

    /// Returns `true` if this node is the current Raft leader.
    pub fn is_leader(&self) -> bool {
        *self.role.read() == NodeRole::Leader
    }

    /// Returns the current role of this node.
    pub fn role(&self) -> NodeRole {
        *self.role.read()
    }

    /// Returns the leader's HTTP URL to redirect write requests to.
    ///
    /// Returns `None` when this node IS the leader (no redirect needed) or
    /// when no leader has been elected yet.
    pub fn leader_redirect_url(&self) -> Option<String> {
        if self.is_leader() {
            return None;
        }
        self.current_leader_url.read().clone()
    }

    /// Returns a snapshot of current leader information suitable for API responses.
    pub fn leader_info(&self) -> LeaderInfo {
        LeaderInfo {
            leader_id: *self.current_leader_id.read(),
            leader_url: self.current_leader_url.read().clone(),
            local_node_id: self.local_node_id,
            role: self.role(),
        }
    }
}

/// Snapshot of leader information returned by REST endpoints.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LeaderInfo {
    pub leader_id: Option<u64>,
    pub leader_url: Option<String>,
    pub local_node_id: u64,
    pub role: NodeRole,
}
