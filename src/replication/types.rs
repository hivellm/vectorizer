//! Replication types and structures
//!
//! Defines the core types used in master-replica replication

use std::net::SocketAddr;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
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
    FullSync { snapshot_data: Vec<u8>, offset: u64 },

    /// Partial sync - incremental updates from offset
    PartialSync {
        from_offset: u64,
        operations: Vec<ReplicationOperation>,
    },

    /// Single operation replication
    Operation(ReplicationOperation),

    /// Heartbeat - keep connection alive, measure lag
    Heartbeat { master_offset: u64, timestamp: u64 },

    /// Acknowledge - replica confirms receipt
    Ack { replica_id: String, offset: u64 },
}

/// Operation to be replicated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationOperation {
    pub offset: u64,
    pub timestamp: u64,
    pub operation: VectorOperation,
}

/// Vector database operations that can be replicated
///
/// Note: owner_id fields are always serialized (no skip_serializing_if) because
/// bincode requires all fields to be present. For JSON APIs, separate DTOs should
/// be used if field omission is desired.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorOperation {
    /// Create a new collection
    CreateCollection {
        name: String,
        config: CollectionConfigData,
        /// Owner/tenant ID for multi-tenant mode (HiveHub integration)
        #[serde(default)]
        owner_id: Option<String>,
    },
    /// Delete a collection
    DeleteCollection {
        name: String,
        /// Owner/tenant ID for multi-tenant mode (HiveHub integration)
        #[serde(default)]
        owner_id: Option<String>,
    },
    /// Insert vector into collection
    InsertVector {
        collection: String,
        id: String,
        vector: Vec<f32>,
        payload: Option<Vec<u8>>,
        /// Owner/tenant ID for multi-tenant mode (HiveHub integration)
        #[serde(default)]
        owner_id: Option<String>,
    },
    /// Update vector in collection
    UpdateVector {
        collection: String,
        id: String,
        vector: Option<Vec<f32>>,
        payload: Option<Vec<u8>>,
        /// Owner/tenant ID for multi-tenant mode (HiveHub integration)
        #[serde(default)]
        owner_id: Option<String>,
    },
    /// Delete vector from collection
    DeleteVector {
        collection: String,
        id: String,
        /// Owner/tenant ID for multi-tenant mode (HiveHub integration)
        #[serde(default)]
        owner_id: Option<String>,
    },
}

/// Serializable collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfigData {
    pub dimension: usize,
    pub metric: String, // "cosine", "euclidean", "dot_product"
}

/// Replication statistics - Complete monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStats {
    /// Current node role
    pub role: NodeRole,

    /// Replication lag in milliseconds
    pub lag_ms: u64,

    /// Total bytes sent (master only)
    pub bytes_sent: u64,

    /// Total bytes received
    pub bytes_received: u64,

    /// Last successful sync timestamp
    #[serde(with = "system_time_as_secs")]
    pub last_sync: SystemTime,

    /// Operations waiting to replicate
    pub operations_pending: usize,

    /// Size of last snapshot in bytes
    pub snapshot_size: usize,

    /// Number of connected replicas (master only)
    pub connected_replicas: Option<usize>,

    // Legacy fields (kept for backwards compatibility)
    /// Master offset (latest operation)
    pub master_offset: u64,
    /// Replica offset (last replicated operation)
    pub replica_offset: u64,
    /// Replication lag in operations
    pub lag_operations: u64,
    /// Total operations replicated
    pub total_replicated: u64,
}

impl Default for ReplicationStats {
    fn default() -> Self {
        Self {
            role: NodeRole::Standalone,
            lag_ms: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_sync: SystemTime::now(),
            operations_pending: 0,
            snapshot_size: 0,
            connected_replicas: None,
            master_offset: 0,
            replica_offset: 0,
            lag_operations: 0,
            total_replicated: 0,
        }
    }
}

/// Replica status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReplicaStatus {
    /// Replica is connected and syncing normally
    Connected,
    /// Replica is performing initial sync
    Syncing,
    /// Replica is lagging behind (lag > threshold)
    Lagging,
    /// Replica is disconnected
    Disconnected,
}

/// Replica information - Complete health tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    /// Replica identifier
    pub id: String,

    /// Replica host
    pub host: String,

    /// Replica port
    pub port: u16,

    /// Connection status
    pub status: ReplicaStatus,

    /// Current replication lag in milliseconds
    pub lag_ms: u64,

    /// Last heartbeat received
    #[serde(with = "system_time_as_secs")]
    pub last_heartbeat: SystemTime,

    /// Total operations successfully synced
    pub operations_synced: u64,

    // Legacy fields (kept for backwards compatibility)
    /// Replica address (deprecated, use host:port)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<SocketAddr>,
    /// Current offset
    pub offset: u64,
    /// Operations lag
    pub lag_operations: u64,
}

impl ReplicaInfo {
    /// Create new replica info from address
    pub fn new(id: String, address: SocketAddr) -> Self {
        let host = address.ip().to_string();
        let port = address.port();

        Self {
            id,
            host,
            port,
            status: ReplicaStatus::Connected,
            lag_ms: 0,
            last_heartbeat: SystemTime::now(),
            operations_synced: 0,
            address: Some(address),
            offset: 0,
            lag_operations: 0,
        }
    }

    /// Update replica status based on lag and heartbeat
    pub fn update_status(&mut self) {
        let now = SystemTime::now();
        let heartbeat_age = now
            .duration_since(self.last_heartbeat)
            .unwrap_or_default()
            .as_secs();

        // Mark disconnected if no heartbeat for 60 seconds
        if heartbeat_age > 60 {
            self.status = ReplicaStatus::Disconnected;
        } else if self.lag_ms > 1000 {
            self.status = ReplicaStatus::Lagging;
        } else if self.offset == 0 {
            self.status = ReplicaStatus::Syncing;
        } else {
            self.status = ReplicaStatus::Connected;
        }
    }
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

// Helper module for SystemTime serialization
mod system_time_as_secs {
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        serializer.serialize_u64(duration)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}
