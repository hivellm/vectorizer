//! Raft leadership watcher — bridges Raft consensus with HA role transitions.
//!
//! Subscribes to `openraft` server metrics via `Raft::server_metrics()` and
//! reacts to leadership changes by calling [`HaManager::on_become_leader`] or
//! [`HaManager::on_become_follower`], updating the [`LeaderRouter`], and
//! ensuring the replication data-plane always matches the Raft consensus role.

use std::sync::Arc;

use openraft::ServerState;
// WatchReceiver trait must be in scope for `borrow_watched()` and `changed()`.
use openraft::rt::WatchReceiver;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use super::ha_manager::HaManager;
use super::raft_node::{RaftManager, TypeConfig};

/// Default replication port used by the master TCP listener.
const DEFAULT_REPLICATION_PORT: u16 = 7001;

/// Default REST API port (used to build leader redirect URL).
const DEFAULT_HTTP_PORT: u16 = 15002;

/// Watches Raft server metrics for leadership changes and drives HA transitions.
///
/// Once started, the watcher runs a background task that:
/// 1. Subscribes to `raft.server_metrics()` (a `tokio::sync::watch` channel).
/// 2. Detects transitions: Follower → Leader, Leader → Follower, etc.
/// 3. Calls `HaManager::on_become_leader()` / `on_become_follower()` accordingly.
/// 4. Updates the `LeaderRouter` so write requests are routed correctly.
pub struct RaftWatcher {
    raft_manager: Arc<RaftManager>,
    ha_manager: Arc<HaManager>,
}

impl RaftWatcher {
    pub fn new(raft_manager: Arc<RaftManager>, ha_manager: Arc<HaManager>) -> Self {
        Self {
            raft_manager,
            ha_manager,
        }
    }

    /// Spawn the background watcher task.
    ///
    /// Returns a `JoinHandle` that runs until the Raft node shuts down or the
    /// metrics channel is closed.
    pub fn start(&self) -> JoinHandle<()> {
        let raft = self.raft_manager.raft.clone();
        let node_id = self.raft_manager.node_id;
        let ha = self.ha_manager.clone();
        let state_machine = self.raft_manager.state_machine.clone();

        tokio::spawn(async move {
            info!(
                node_id,
                "🔭 Raft watcher started — monitoring leadership changes"
            );

            let mut rx = raft.server_metrics();

            // Track previous state so we only react to actual transitions.
            let mut prev_state: Option<ServerState> = None;
            let mut prev_leader: Option<u64> = None;

            loop {
                // Read current metrics snapshot.
                let (current_state, current_leader) = {
                    let metrics = rx.borrow_watched();
                    (metrics.state, metrics.current_leader)
                };

                let state_changed = prev_state.as_ref() != Some(&current_state);
                let leader_changed = prev_leader != current_leader;

                if state_changed || leader_changed {
                    info!(
                        node_id,
                        ?current_state,
                        ?current_leader,
                        ?prev_state,
                        ?prev_leader,
                        "🔄 Raft state transition detected"
                    );

                    match current_state {
                        ServerState::Leader => {
                            if prev_state != Some(ServerState::Leader) {
                                // This node just won an election.
                                info!(node_id, "👑 This node became LEADER — starting MasterNode");

                                // Update LeaderRouter first so writes are accepted immediately.
                                let self_url = build_self_http_url();
                                ha.leader_router.set_leader(node_id, self_url);

                                // Start MasterNode, stop ReplicaNode.
                                ha.on_become_leader().await;
                            }
                        }
                        ServerState::Follower => {
                            if prev_state != Some(ServerState::Follower) || leader_changed {
                                // Either we just stepped down, or the leader changed.
                                let leader_addr =
                                    resolve_leader_addr(&state_machine, current_leader).await;

                                if let Some(ref addr) = leader_addr {
                                    let leader_id = current_leader.unwrap_or(0);
                                    let leader_http_url =
                                        format!("http://{}:{}", addr, DEFAULT_HTTP_PORT);
                                    ha.leader_router.set_leader(leader_id, leader_http_url);

                                    info!(
                                        node_id,
                                        leader_id,
                                        leader_addr = %addr,
                                        "📡 Following new leader"
                                    );
                                } else if current_leader.is_some() {
                                    warn!(
                                        node_id,
                                        ?current_leader,
                                        "Leader elected but address not yet in state machine"
                                    );
                                } else {
                                    ha.leader_router.clear_leader();
                                }

                                // Build the master replication address for the ReplicaNode.
                                let repl_addr = leader_addr
                                    .map(|addr| format!("{}:{}", addr, DEFAULT_REPLICATION_PORT));

                                ha.on_become_follower(repl_addr).await;
                            }
                        }
                        ServerState::Candidate => {
                            if prev_state != Some(ServerState::Candidate) {
                                info!(node_id, "🗳️  Election in progress — node is Candidate");
                                ha.leader_router.clear_leader();
                            }
                        }
                        ServerState::Learner => {
                            info!(node_id, "📚 Node is Learner (non-voting)");
                        }
                        ServerState::Shutdown => {
                            info!(node_id, "🛑 Raft shutting down — watcher exiting");
                            break;
                        }
                    }

                    prev_state = Some(current_state);
                    prev_leader = current_leader;
                }

                // Wait for the next metrics change. This suspends the task
                // (zero CPU) until openraft publishes a new value.
                if rx.changed().await.is_err() {
                    info!(node_id, "Raft metrics channel closed — watcher exiting");
                    break;
                }
            }
        })
    }
}

/// Resolve the leader's IP/hostname from the Raft state machine.
///
/// The state machine's `nodes` map stores `node_id → (address, grpc_port)`.
/// We use the address portion to build both the HTTP URL and the replication
/// TCP address.
async fn resolve_leader_addr(
    state_machine: &Arc<super::raft_node::ClusterStateMachine>,
    leader_id: Option<u64>,
) -> Option<String> {
    let leader_id = leader_id?;
    let state = state_machine.state().await;

    state
        .nodes
        .get(&leader_id)
        .map(|(address, _grpc_port)| address.clone())
}

/// Build the HTTP URL for this node.
///
/// In Kubernetes, the pod's hostname is typically set to the pod name, and
/// the headless service makes it resolvable as `<pod-name>.<service>.<namespace>.svc.cluster.local`.
/// We use `0.0.0.0` here because the LeaderRouter only needs this for the
/// "am I the leader?" check — followers get the leader URL from the state machine.
fn build_self_http_url() -> String {
    // Try to use HOSTNAME env var (set by Kubernetes) for a routable address.
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        // In K8s with a headless service, the hostname is the pod name.
        // If VECTORIZER_SERVICE_NAME is set, build a fully qualified DNS name.
        if let Ok(svc) = std::env::var("VECTORIZER_SERVICE_NAME") {
            return format!("http://{}.{}:{}", hostname, svc, DEFAULT_HTTP_PORT);
        }
        return format!("http://{}:{}", hostname, DEFAULT_HTTP_PORT);
    }

    // Fallback: use POD_IP if available (injected via Kubernetes downward API).
    if let Ok(pod_ip) = std::env::var("POD_IP") {
        return format!("http://{}:{}", pod_ip, DEFAULT_HTTP_PORT);
    }

    // Last resort: localhost (only works for single-node testing).
    format!("http://127.0.0.1:{}", DEFAULT_HTTP_PORT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_self_http_url_fallback() {
        // When no env vars are set, should fall back to localhost.
        // SAFETY: test is single-threaded; no concurrent env access.
        unsafe {
            std::env::remove_var("HOSTNAME");
            std::env::remove_var("VECTORIZER_SERVICE_NAME");
            std::env::remove_var("POD_IP");
        }

        let url = build_self_http_url();
        assert!(url.contains("127.0.0.1"));
        assert!(url.contains("15002"));
    }
}
