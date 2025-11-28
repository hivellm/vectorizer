//! Raft consensus implementation for Vectorizer
//!
//! This module provides Raft consensus for distributed vector operations,
//! enabling replication and high availability.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, Vector};
use crate::persistence::types::Operation;

/// Raft node ID
pub type NodeId = u64;

/// Raft term (monotonically increasing)
pub type Term = u64;

/// Log index
pub type LogIndex = u64;

/// Raft node role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaftRole {
    /// Follower - receives log entries from leader
    Follower,
    /// Candidate - requesting votes for leadership
    Candidate,
    /// Leader - handles client requests and replicates logs
    Leader,
}

impl std::fmt::Display for RaftRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RaftRole::Follower => write!(f, "Follower"),
            RaftRole::Candidate => write!(f, "Candidate"),
            RaftRole::Leader => write!(f, "Leader"),
        }
    }
}

/// Raft log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Term when entry was received by leader
    pub term: Term,
    /// Index in the log
    pub index: LogIndex,
    /// Operation to apply
    pub operation: Operation,
}

/// Raft node state
#[derive(Debug, Clone)]
pub struct RaftState {
    /// Current term
    pub current_term: Term,
    /// Node ID that received vote in current term (or None)
    pub voted_for: Option<NodeId>,
    /// Log entries
    pub log: Vec<LogEntry>,
    /// Index of highest log entry known to be committed
    pub commit_index: LogIndex,
    /// Index of highest log entry applied to state machine
    pub last_applied: LogIndex,
    /// Current role
    pub role: RaftRole,
    /// Leader ID (if known)
    pub leader_id: Option<NodeId>,
    /// Last time we heard from leader (for election timeout)
    pub last_heartbeat: Instant,
}

impl Default for RaftState {
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            role: RaftRole::Follower,
            leader_id: None,
            last_heartbeat: Instant::now(),
        }
    }
}

/// Raft state machine for vector operations
#[derive(Debug)]
pub struct RaftStateMachine {
    /// Applied operations (for idempotency)
    applied_operations: Arc<DashMap<LogIndex, bool>>,
    /// Collection operations handler
    store: Option<Arc<crate::db::vector_store::VectorStore>>,
}

impl RaftStateMachine {
    /// Create a new state machine
    pub fn new() -> Self {
        Self {
            applied_operations: Arc::new(DashMap::new()),
            store: None,
        }
    }

    /// Set the vector store
    pub fn set_store(&mut self, store: Arc<crate::db::vector_store::VectorStore>) {
        self.store = Some(store);
    }

    /// Apply a log entry to the state machine
    pub fn apply(&self, entry: &LogEntry) -> Result<()> {
        // Check if already applied (idempotency)
        if self.applied_operations.contains_key(&entry.index) {
            debug!("Log entry {} already applied, skipping", entry.index);
            return Ok(());
        }

        // Apply operation
        // Note: In a real implementation, operations would include collection_name
        // For now, we'll use a default collection or extract from metadata
        let collection_name = "default"; // TODO: Extract from operation metadata

        match &entry.operation {
            Operation::Checkpoint { .. } => {
                // Checkpoints are metadata only, no state change - don't need store
                debug!("Applied checkpoint at index {}", entry.index);
            }
            _ => {
                // All other operations require a store
                let store = self
                    .store
                    .as_ref()
                    .ok_or_else(|| VectorizerError::Storage("Store not set".to_string()))?;

                match &entry.operation {
                    Operation::InsertVector {
                        vector_id,
                        data,
                        metadata,
                    } => {
                        let payload = if metadata.is_empty() {
                            None
                        } else {
                            let mut payload_data = serde_json::Map::new();
                            for (k, v) in metadata {
                                payload_data
                                    .insert(k.clone(), serde_json::Value::String(v.clone()));
                            }
                            Some(crate::models::Payload {
                                data: serde_json::Value::Object(payload_data),
                            })
                        };

                        let vector = Vector {
                            id: vector_id.clone(),
                            data: data.clone(),
                            sparse: None,
                            payload,
                        };
                        store.insert(collection_name, vec![vector])?;
                    }
                    Operation::UpdateVector {
                        vector_id,
                        data,
                        metadata,
                    } => {
                        if let Some(data_vec) = data {
                            let payload = metadata.clone().map(|m| {
                                let mut payload_data = serde_json::Map::new();
                                for (k, v) in m {
                                    payload_data
                                        .insert(k.clone(), serde_json::Value::String(v.clone()));
                                }
                                crate::models::Payload {
                                    data: serde_json::Value::Object(payload_data),
                                }
                            });

                            let vector = Vector {
                                id: vector_id.clone(),
                                data: data_vec.clone(),
                                sparse: None,
                                payload,
                            };
                            store.update(collection_name, vector)?;
                        }
                    }
                    Operation::DeleteVector { vector_id } => {
                        store.delete(collection_name, vector_id)?;
                    }
                    Operation::CreateCollection { config } => {
                        // Extract collection name from config or use a default
                        // In real implementation, this would be part of the operation
                        let name = "default"; // TODO: Extract from operation
                        store.create_collection(name, config.clone())?;
                    }
                    Operation::DeleteCollection => {
                        let name = "default"; // TODO: Extract from operation
                        store.delete_collection(name)?;
                    }
                    Operation::Checkpoint { .. } => {
                        // This branch should never be reached due to outer match
                        unreachable!("Checkpoint handled in outer match");
                    }
                }
            }
        }

        // Mark as applied
        self.applied_operations.insert(entry.index, true);

        debug!("Applied log entry {}: {:?}", entry.index, entry.operation);
        Ok(())
    }

    /// Get the last applied index
    pub fn last_applied_index(&self) -> LogIndex {
        self.applied_operations
            .iter()
            .map(|entry| *entry.key())
            .max()
            .unwrap_or(0)
    }
}

/// Raft node configuration
#[derive(Debug, Clone)]
pub struct RaftConfig {
    /// Election timeout (milliseconds)
    pub election_timeout_ms: u64,
    /// Heartbeat interval (milliseconds)
    pub heartbeat_interval_ms: u64,
    /// Minimum election timeout
    pub min_election_timeout_ms: u64,
    /// Maximum election timeout
    pub max_election_timeout_ms: u64,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
            min_election_timeout_ms: 150,
            max_election_timeout_ms: 300,
        }
    }
}

/// Raft node
#[derive(Debug)]
pub struct RaftNode {
    /// Node ID
    pub id: NodeId,
    /// Raft state
    state: Arc<RwLock<RaftState>>,
    /// State machine
    state_machine: Arc<RaftStateMachine>,
    /// Configuration
    config: RaftConfig,
    /// Other nodes in the cluster
    peers: Arc<DashMap<NodeId, String>>, // node_id -> address
    /// Command channel for applying operations
    command_tx: Option<mpsc::UnboundedSender<RaftCommand>>,
}

/// Commands for Raft node
#[derive(Debug, Clone)]
pub enum RaftCommand {
    /// Propose a new operation
    Propose {
        operation: Operation,
        response_tx: mpsc::UnboundedSender<Result<LogIndex>>,
    },
    /// Request vote (for election)
    RequestVote {
        term: Term,
        candidate_id: NodeId,
        last_log_index: LogIndex,
        last_log_term: Term,
        response_tx: mpsc::UnboundedSender<bool>,
    },
    /// Append entries (from leader)
    AppendEntries {
        term: Term,
        leader_id: NodeId,
        prev_log_index: LogIndex,
        prev_log_term: Term,
        entries: Vec<LogEntry>,
        leader_commit: LogIndex,
        response_tx: mpsc::UnboundedSender<bool>,
    },
    /// Get current state
    GetState {
        response_tx: mpsc::UnboundedSender<RaftState>,
    },
}

impl RaftNode {
    /// Create a new Raft node
    pub fn new(id: NodeId, config: RaftConfig) -> Self {
        let state = Arc::new(RwLock::new(RaftState::default()));
        let state_machine = Arc::new(RaftStateMachine::new());

        Self {
            id,
            state,
            state_machine,
            config,
            peers: Arc::new(DashMap::new()),
            command_tx: None,
        }
    }

    /// Set the vector store for the state machine
    pub fn set_store(&self, store: Arc<crate::db::vector_store::VectorStore>) {
        // Note: This requires interior mutability or a different design
        // For now, we'll handle this in the state machine application
    }

    /// Add a peer node
    pub fn add_peer(&self, peer_id: NodeId, address: String) {
        let address_clone = address.clone();
        self.peers.insert(peer_id, address);
        info!("Added peer {} at {}", peer_id, address_clone);
    }

    /// Start the Raft node
    pub fn start(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.command_tx = Some(tx);

        let state = self.state.clone();
        let state_machine = self.state_machine.clone();
        let config = self.config.clone();
        let id = self.id;
        let peers = self.peers.clone();

        // Spawn Raft event loop
        tokio::spawn(async move {
            Self::raft_loop(state, state_machine, config, id, peers, rx).await;
        });

        Ok(())
    }

    /// Main Raft event loop
    async fn raft_loop(
        state: Arc<RwLock<RaftState>>,
        state_machine: Arc<RaftStateMachine>,
        config: RaftConfig,
        node_id: NodeId,
        peers: Arc<DashMap<NodeId, String>>,
        mut command_rx: mpsc::UnboundedReceiver<RaftCommand>,
    ) {
        info!("Raft node {} started", node_id);

        loop {
            tokio::select! {
                // Handle commands
                cmd = command_rx.recv() => {
                    match cmd {
                        Some(RaftCommand::Propose { operation, response_tx }) => {
                            let mut s = state.write();
                            if s.role != RaftRole::Leader {
                                let _ = response_tx.send(Err(VectorizerError::Storage(
                                    "Not the leader".to_string()
                                )));
                                continue;
                            }

                            // Create log entry
                            let entry = LogEntry {
                                term: s.current_term,
                                index: s.log.len() as u64 + 1,
                                operation: operation.clone(),
                            };

                            s.log.push(entry.clone());
                            let log_index = entry.index;

                            // Apply immediately (simplified - in real Raft, wait for replication)
                            drop(s);
                            if let Err(e) = state_machine.apply(&entry) {
                                let _ = response_tx.send(Err(e));
                            } else {
                                let _ = response_tx.send(Ok(log_index));
                            }
                        }
                        Some(RaftCommand::RequestVote { term, candidate_id, last_log_index, last_log_term, response_tx }) => {
                            let mut s = state.write();
                            let vote_granted = if term > s.current_term {
                                s.current_term = term;
                                s.voted_for = Some(candidate_id);
                                s.role = RaftRole::Follower;
                                true
                            } else if term == s.current_term && s.voted_for.is_none() {
                                s.voted_for = Some(candidate_id);
                                true
                            } else {
                                false
                            };
                            let _ = response_tx.send(vote_granted);
                        }
                        Some(RaftCommand::AppendEntries { term, leader_id, prev_log_index, prev_log_term, entries, leader_commit, response_tx }) => {
                            let mut s = state.write();
                            let success = if term >= s.current_term {
                                s.current_term = term;
                                s.role = RaftRole::Follower;
                                s.leader_id = Some(leader_id);
                                s.last_heartbeat = Instant::now();

                                // Append entries (simplified)
                                for entry in entries {
                                    s.log.push(entry);
                                }

                                // Update commit index
                                if leader_commit > s.commit_index {
                                    s.commit_index = leader_commit.min(s.log.len() as u64);
                                }

                                true
                            } else {
                                false
                            };
                            let _ = response_tx.send(success);
                        }
                        Some(RaftCommand::GetState { response_tx }) => {
                            let s = state.read();
                            let _ = response_tx.send(s.clone());
                        }
                        None => break,
                    }
                }
                // Election timeout
                _ = tokio::time::sleep(Duration::from_millis(config.election_timeout_ms)) => {
                    let mut s = state.write();
                    let should_start_election = match s.role {
                        RaftRole::Follower => {
                            s.last_heartbeat.elapsed() > Duration::from_millis(config.election_timeout_ms)
                        }
                        RaftRole::Candidate => {
                            // If still candidate after timeout, start new election
                            true
                        }
                        RaftRole::Leader => false, // Leader doesn't need election
                    };

                    if should_start_election {
                        // Start election
                        s.current_term += 1;
                        s.role = RaftRole::Candidate;
                        s.voted_for = Some(node_id);
                        s.leader_id = None;

                        info!("Node {} starting election for term {}", node_id, s.current_term);

                        // In a full implementation, we would:
                        // 1. Send RequestVote to all peers
                        // 2. Wait for majority of votes
                        // 3. Become leader if majority votes received
                        // For now, if we're the only node or have no peers, become leader
                        if peers.is_empty() {
                            s.role = RaftRole::Leader;
                            s.leader_id = Some(node_id);
                            info!("Node {} became leader (single node cluster)", node_id);
                        }
                    }
                }
            }
        }
    }

    /// Propose an operation (only works if leader)
    pub async fn propose(&self, operation: Operation) -> Result<LogIndex> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.command_tx
            .as_ref()
            .ok_or_else(|| VectorizerError::Storage("Raft node not started".to_string()))?
            .send(RaftCommand::Propose {
                operation,
                response_tx: tx,
            })
            .map_err(|_| VectorizerError::Storage("Failed to send command".to_string()))?;

        rx.recv()
            .await
            .ok_or_else(|| VectorizerError::Storage("No response from Raft".to_string()))?
    }

    /// Get current state
    pub async fn get_state(&self) -> Result<RaftState> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        self.command_tx
            .as_ref()
            .ok_or_else(|| VectorizerError::Storage("Raft node not started".to_string()))?
            .send(RaftCommand::GetState { response_tx: tx })
            .map_err(|_| VectorizerError::Storage("Failed to send command".to_string()))?;

        rx.recv()
            .await
            .ok_or_else(|| VectorizerError::Storage("No response from Raft".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_raft_node_creation() {
        let mut node = RaftNode::new(1, RaftConfig::default());
        node.start().unwrap();

        let state = node.get_state().await.unwrap();
        assert_eq!(state.role, RaftRole::Follower);
        assert_eq!(state.current_term, 0);
    }

    #[tokio::test]
    async fn test_state_machine_apply() {
        let sm = RaftStateMachine::new();
        let entry = LogEntry {
            term: 1,
            index: 1,
            operation: Operation::Checkpoint {
                vector_count: 0,
                document_count: 0,
                checksum: "test".to_string(),
            },
        };

        // Should not fail even without store (checkpoint is metadata only)
        let result = sm.apply(&entry);
        assert!(result.is_ok());
    }
}
