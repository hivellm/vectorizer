//! Leader-aware routing for Raft-based HA cluster
//!
//! Tracks the current Raft leader and provides routing decisions for
//! write requests, enabling transparent redirect to the leader node.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{info, warn};

/// Internal state guarded by a single lock to avoid TOCTOU races.
#[derive(Debug, Clone)]
struct LeaderState {
    /// Current leader node ID (None if no leader elected)
    leader_id: Option<u64>,
    /// Current leader HTTP address (for redirects)
    leader_url: Option<String>,
    /// This node's current role
    role: NodeRole,
}

/// Tracks the current Raft leader and provides routing decisions.
#[derive(Debug, Clone)]
pub struct LeaderRouter {
    state: Arc<RwLock<LeaderState>>,
    /// This node's ID (immutable after creation)
    local_node_id: u64,
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
            state: Arc::new(RwLock::new(LeaderState {
                leader_id: None,
                leader_url: None,
                role: NodeRole::Follower,
            })),
            local_node_id,
        }
    }

    /// Update leader info. Called by Raft callbacks when a new leader is elected.
    ///
    /// All fields are updated atomically under a single lock acquisition.
    pub fn set_leader(&self, leader_id: u64, leader_url: String) {
        let mut state = self.state.write();
        state.leader_id = Some(leader_id);
        state.leader_url = Some(leader_url.clone());

        if leader_id == self.local_node_id {
            info!("This node is now the LEADER (id={})", leader_id);
            state.role = NodeRole::Leader;
        } else {
            info!("Leader changed to node {} (url: {})", leader_id, leader_url);
            state.role = NodeRole::Follower;
        }
    }

    /// Clear leader state. Called when no leader is currently elected.
    pub fn clear_leader(&self) {
        let mut state = self.state.write();
        state.leader_id = None;
        state.leader_url = None;
        state.role = NodeRole::Candidate;
        warn!("No leader elected – node entering Candidate state");
    }

    /// Returns `true` if this node is the current Raft leader.
    pub fn is_leader(&self) -> bool {
        self.state.read().role == NodeRole::Leader
    }

    /// Returns the current role of this node.
    pub fn role(&self) -> NodeRole {
        self.state.read().role
    }

    /// Returns the leader's HTTP URL to redirect write requests to.
    ///
    /// Returns `None` when this node IS the leader (no redirect needed) or
    /// when no leader has been elected yet.
    pub fn leader_redirect_url(&self) -> Option<String> {
        let state = self.state.read();
        if state.role == NodeRole::Leader {
            return None;
        }
        state.leader_url.clone()
    }

    /// Returns a snapshot of current leader information suitable for API responses.
    pub fn leader_info(&self) -> LeaderInfo {
        let state = self.state.read();
        LeaderInfo {
            leader_id: state.leader_id,
            leader_url: state.leader_url.clone(),
            local_node_id: self.local_node_id,
            role: state.role,
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
