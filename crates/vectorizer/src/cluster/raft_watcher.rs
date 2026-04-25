//! Raft leadership watcher — bridges Raft consensus with HA role transitions.
//!
//! Subscribes to `openraft` server metrics via `Raft::server_metrics()` and
//! reacts to leadership changes by calling [`HaManager::on_become_leader`] or
//! [`HaManager::on_become_follower`], updating the [`LeaderRouter`], and
//! ensuring the replication data-plane always matches the Raft consensus role.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

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

/// Default REST API port (used to build leader redirect URL when no port is
/// explicitly configured).
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
    /// The HTTP port this node is listening on (for leader redirect URLs).
    http_port: u16,
}

impl RaftWatcher {
    pub fn new(raft_manager: Arc<RaftManager>, ha_manager: Arc<HaManager>) -> Self {
        Self {
            raft_manager,
            ha_manager,
            http_port: DEFAULT_HTTP_PORT,
        }
    }

    /// Create a watcher with an explicit HTTP port for leader redirect URLs.
    pub fn with_http_port(
        raft_manager: Arc<RaftManager>,
        ha_manager: Arc<HaManager>,
        http_port: u16,
    ) -> Self {
        Self {
            raft_manager,
            ha_manager,
            http_port,
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
        let http_port = self.http_port;

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
                                let self_url = build_self_http_url(http_port);
                                ha.leader_router.set_leader(node_id, self_url);

                                // Start MasterNode, stop ReplicaNode.
                                ha.on_become_leader().await;
                            }
                        }
                        ServerState::Follower => {
                            if prev_state != Some(ServerState::Follower) || leader_changed {
                                // Either we just stepped down, or the leader changed.
                                let leader_addr =
                                    resolve_leader_addr(&raft, &state_machine, current_leader).await;

                                if let Some(ref addr) = leader_addr {
                                    let leader_id = current_leader.unwrap_or(0);
                                    let leader_http_url = format!("http://{}:{}", addr, http_port);
                                    ha.leader_router.set_leader(leader_id, leader_http_url);

                                    info!(
                                        node_id,
                                        leader_id,
                                        leader_addr = %addr,
                                        "📡 Following new leader"
                                    );
                                } else if current_leader.is_some() {
                                    // Address not yet in state machine — AddNode
                                    // commands may still be propagating. Retry a
                                    // few times before giving up.
                                    let mut resolved = false;
                                    for attempt in 1..=6 {
                                        info!(
                                            node_id,
                                            attempt,
                                            "Waiting for leader address in state machine..."
                                        );
                                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                        if let Some(addr) =
                                            resolve_leader_addr(&raft, &state_machine, current_leader)
                                                .await
                                        {
                                            let leader_id = current_leader.unwrap_or(0);
                                            let leader_http_url =
                                                format!("http://{}:{}", addr, http_port);
                                            ha.leader_router.set_leader(leader_id, leader_http_url);

                                            info!(
                                                node_id,
                                                leader_id,
                                                leader_addr = %addr,
                                                "📡 Following new leader (resolved after {}s)",
                                                attempt * 2
                                            );

                                            let repl =
                                                format!("{}:{}", addr, DEFAULT_REPLICATION_PORT);
                                            ha.on_become_follower(Some(repl)).await;
                                            resolved = true;
                                            break;
                                        }
                                    }
                                    if !resolved {
                                        warn!(
                                            node_id,
                                            ?current_leader,
                                            "Leader address not found after retries"
                                        );
                                    }
                                } else {
                                    ha.leader_router.clear_leader();
                                }

                                // If leader_addr was resolved on the first try (not via
                                // the retry loop above), start the ReplicaNode now.
                                if leader_addr.is_some() {
                                    let repl_addr = leader_addr.map(|addr| {
                                        format!("{}:{}", addr, DEFAULT_REPLICATION_PORT)
                                    });
                                    ha.on_become_follower(repl_addr).await;
                                } else if current_leader.is_none() {
                                    // No leader at all — just transition to follower
                                    ha.on_become_follower(None).await;
                                }
                                // If current_leader.is_some() but leader_addr was None,
                                // the retry loop above already called on_become_follower.
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
/// Reads the address from the openraft membership config — populated by
/// `initialize_cluster()` before the first election and always reflects the
/// authoritative member set, so followers can route the moment a leader is
/// known. The legacy fallback to the state-machine `nodes` map (populated by
/// the post-bootstrap `AddNode` proposal) handles dynamically-added nodes that
/// were never in the initial membership.
async fn resolve_leader_addr(
    raft: &super::raft_node::VectorizerRaft,
    state_machine: &Arc<super::raft_node::ClusterStateMachine>,
    leader_id: Option<u64>,
) -> Option<String> {
    let leader_id = leader_id?;

    // Primary: openraft membership. Set by `initialize_cluster` and replicated
    // as part of the Raft log, so every node sees the same map within one
    // round of `AppendEntries` — no reliance on the state-machine `AddNode`
    // round-trip that previously stalled before the first stable leader.
    {
        let metrics = raft.metrics().borrow_watched().clone();
        if let Some(info) = metrics.membership_config.membership().get_node(&leader_id)
            && !info.address.is_empty()
        {
            return Some(info.address.clone());
        }
    }

    // Secondary: state-machine nodes map, kept for backward compat with
    // dynamic AddNode flows that introduce members after bootstrap.
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
fn build_self_http_url(port: u16) -> String {
    // Try to use HOSTNAME env var (set by Kubernetes) for a routable address.
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        // In K8s with a headless service, the hostname is the pod name.
        // If VECTORIZER_SERVICE_NAME is set, build a fully qualified DNS name.
        if let Ok(svc) = std::env::var("VECTORIZER_SERVICE_NAME") {
            return format!("http://{}.{}:{}", hostname, svc, port);
        }
        return format!("http://{}:{}", hostname, port);
    }

    // Fallback: use POD_IP if available (injected via Kubernetes downward API).
    if let Ok(pod_ip) = std::env::var("POD_IP") {
        return format!("http://{}:{}", pod_ip, port);
    }

    // Last resort: localhost (only works for single-node testing).
    format!("http://127.0.0.1:{}", port)
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

        let url = build_self_http_url(15002);
        assert!(url.contains("127.0.0.1"));
        assert!(url.contains("15002"));
    }

    #[test]
    fn test_build_self_http_url_custom_port() {
        // SAFETY: test is single-threaded; no concurrent env access.
        unsafe {
            std::env::remove_var("HOSTNAME");
            std::env::remove_var("VECTORIZER_SERVICE_NAME");
            std::env::remove_var("POD_IP");
        }

        let url = build_self_http_url(8080);
        assert!(url.contains("127.0.0.1:8080"));
    }
}
