//! Qdrant Sharding API models
//!
//! This module contains the request/response types for the Qdrant Sharding API.

use serde::{Deserialize, Serialize};

/// Shard key type (matches Qdrant API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantShardKeyValue {
    /// String shard key
    String(String),
    /// Integer shard key
    Integer(i64),
}

impl std::fmt::Display for QdrantShardKeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QdrantShardKeyValue::String(s) => write!(f, "{}", s),
            QdrantShardKeyValue::Integer(i) => write!(f, "{}", i),
        }
    }
}

/// Request to create a shard key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCreateShardKeyRequest {
    /// The shard key to create
    pub shard_key: QdrantShardKeyValue,
    /// Number of shards to create for this key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shards_number: Option<u32>,
    /// Replication factor for the shard
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication_factor: Option<u32>,
    /// Placement strategy for the shard
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<Vec<u64>>,
}

/// Request to delete a shard key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeleteShardKeyRequest {
    /// The shard key to delete
    pub shard_key: QdrantShardKeyValue,
}

/// Shard key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantShardKeyInfo {
    /// The shard key value
    pub shard_key: QdrantShardKeyValue,
    /// Number of shards for this key
    pub shards_number: u32,
    /// Replication factor
    pub replication_factor: u32,
    /// Local shards for this key
    pub local_shards: Vec<QdrantLocalShardInfo>,
    /// Remote shards for this key
    pub remote_shards: Vec<QdrantRemoteShardInfo>,
}

/// Local shard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantLocalShardInfo {
    /// Shard ID
    pub shard_id: u32,
    /// Number of points in the shard
    pub points_count: u64,
    /// Shard state
    pub state: QdrantShardState,
}

/// Remote shard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRemoteShardInfo {
    /// Shard ID
    pub shard_id: u32,
    /// Peer ID where this shard is located
    pub peer_id: u64,
    /// Shard state
    pub state: QdrantShardState,
}

/// Shard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantShardState {
    /// Shard is active and serving requests
    #[serde(rename = "Active")]
    Active,
    /// Shard is dead and not serving requests
    #[serde(rename = "Dead")]
    Dead,
    /// Shard is being initialized
    #[serde(rename = "Initializing")]
    Initializing,
    /// Shard is being recovered
    #[serde(rename = "Recovering")]
    Recovering,
    /// Shard is partially active
    #[serde(rename = "Partial")]
    Partial,
}

/// Response for create shard key operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCreateShardKeyResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}

/// Response for delete shard key operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeleteShardKeyResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}

/// Response for list shard keys operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantListShardKeysResponse {
    pub result: QdrantShardKeysResult,
    pub status: String,
    pub time: f64,
}

/// Shard keys result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantShardKeysResult {
    /// List of shard keys
    pub keys: Vec<QdrantShardKeyInfo>,
}

/// Collection cluster info (for sharding status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionClusterInfo {
    /// Peer ID of the current node
    pub peer_id: u64,
    /// Total number of shards
    pub shard_count: u32,
    /// Local shards
    pub local_shards: Vec<QdrantLocalShardInfo>,
    /// Remote shards
    pub remote_shards: Vec<QdrantRemoteShardInfo>,
    /// Shard transfers in progress
    pub shard_transfers: Vec<QdrantShardTransfer>,
}

/// Shard transfer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantShardTransfer {
    /// Shard ID being transferred
    pub shard_id: u32,
    /// Source peer ID
    pub from: u64,
    /// Destination peer ID
    pub to: u64,
    /// Transfer progress (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<bool>,
}

/// Response for collection cluster info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionClusterInfoResponse {
    pub result: QdrantCollectionClusterInfo,
    pub status: String,
    pub time: f64,
}
