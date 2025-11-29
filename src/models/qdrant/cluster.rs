//! Qdrant Cluster API models
//!
//! This module contains the request/response types for the Qdrant Cluster API.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Peer state in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantPeerState {
    #[serde(rename = "Active")]
    Active,
    #[serde(rename = "Dead")]
    Dead,
    #[serde(rename = "Restarting")]
    Restarting,
}

/// Peer information in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPeerInfo {
    /// URI of the peer
    pub uri: String,
    /// State of the peer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<QdrantPeerState>,
}

/// Consensus thread status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConsensusThreadStatus {
    /// Is consensus thread running
    pub consensus_thread_status: String,
    /// Last known term
    pub term: u64,
    /// Last known commit
    pub commit: u64,
    /// Pending operations
    pub pending_operations: u64,
    /// Leader peer ID (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader: Option<u64>,
    /// Is this node the leader
    pub is_voter: bool,
}

/// Raft state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRaftInfo {
    /// Current term
    pub term: u64,
    /// Commit index
    pub commit: u64,
    /// Number of pending operations
    pub pending_operations: u64,
    /// Leader peer ID (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader: Option<u64>,
    /// Role of this peer (Leader, Follower, Candidate)
    pub role: Option<String>,
    /// Is this peer a voter
    pub is_voter: bool,
}

/// Cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantClusterStatus {
    /// Cluster status (enabled or disabled)
    pub status: String,
    /// Peer ID of this node
    pub peer_id: u64,
    /// Map of peer IDs to peer info
    pub peers: HashMap<String, QdrantPeerInfo>,
    /// Raft consensus info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raft_info: Option<QdrantRaftInfo>,
    /// Consensus thread status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus_thread_status: Option<QdrantConsensusThreadStatus>,
    /// Message queue size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_send_failures: Option<HashMap<String, u64>>,
}

/// Response for cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantClusterStatusResponse {
    pub result: QdrantClusterStatus,
    pub status: String,
    pub time: f64,
}

/// Response for cluster recover operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantClusterRecoverResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}

/// Response for remove peer operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRemovePeerResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}

/// Cluster metadata key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantMetadataValue {
    /// The value of the metadata
    pub value: serde_json::Value,
}

/// Response for list metadata keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantListMetadataKeysResponse {
    pub result: Vec<String>,
    pub status: String,
    pub time: f64,
}

/// Response for get metadata key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGetMetadataKeyResponse {
    pub result: serde_json::Value,
    pub status: String,
    pub time: f64,
}

/// Request for update metadata key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantUpdateMetadataKeyRequest {
    /// The value to set
    pub value: serde_json::Value,
}

/// Response for update metadata key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantUpdateMetadataKeyResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}
