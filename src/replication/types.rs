//! Replication types and structures
//!
//! Defines the core types used in master-replica replication

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use thiserror::Error;

/// Node role in replication topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    /// Master node - accepts writes, sends to replicas
    Master,
    /// Replica node - read-only, receives from master
    Replica,
    /// Standalone node - no replication
    #[default]
    Standalone,
}

/// Replication command (sent from master to replica)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationCommand {
    /// Full sync - initial snapshot transfer
    FullSync {
        snapshot_data: Vec<u8>,
        offset: u64,
    },

    /// Partial sync - incremental updates from offset
    PartialSync {
        from_offset: u64,
        operations: Vec<ReplicationOperation>,
    },

    /// Single operation replication
    Operation(ReplicationOperation),

    /// Heartbeat - keep connection alive, measure lag
    Heartbeat {
        master_offset: u64,
        timestamp: u64,
    },

    /// Acknowledge - replica confirms receipt
    Ack {
        replica_id: String,
        offset: u64,
    },
}

/// Operation to be replicated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationOperation {
    pub offset: u64,
    pub timestamp: u64,
    pub operation: VectorOperation,
}

/// Vector database operations that can be replicated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorOperation {
    /// Create a new collection
    CreateCollection {
        name: String,
        config: CollectionConfigData,
    },
    /// Delete a collection
    DeleteCollection {
        name: String,
    },
    /// Insert vector into collection
    InsertVector {
        collection: String,
        id: String,
        vector: Vec<f32>,
        payload: Option<Vec<u8>>,
    },
    /// Update vector in collection
    UpdateVector {
        collection: String,
        id: String,
        vector: Option<Vec<f32>>,
        payload: Option<Vec<u8>>,
    },
    /// Delete vector from collection
    DeleteVector {
        collection: String,
        id: String,
    },
}

/// Serializable collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfigData {
    pub dimension: usize,
    pub metric: String, // "cosine", "euclidean", "dot_product"
}

/// Replication statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplicationStats {
    /// Master offset (latest operation)
    pub master_offset: u64,
    /// Replica offset (last replicated operation)
    pub replica_offset: u64,
    /// Replication lag in operations
    pub lag_operations: u64,
    /// Replication lag in milliseconds
    pub lag_ms: u64,
    /// Total operations replicated
    pub total_replicated: u64,
    /// Total bytes replicated
    pub total_bytes: u64,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
    /// Connection status
    pub connected: bool,
}

/// Replica information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    pub id: String,
    pub address: SocketAddr,
    pub offset: u64,
    pub lag_operations: u64,
    pub lag_ms: u64,
    pub connected: bool,
    pub last_heartbeat: u64,
}

/// Replication errors
#[derive(Error, Debug)]
pub enum ReplicationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Replica not found: {0}")]
    ReplicaNotFound(String),

    #[error("Already connected: {0}")]
    AlreadyConnected(String),
}

pub type ReplicationResult<T> = Result<T, ReplicationError>;

