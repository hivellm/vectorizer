//! Qdrant Snapshot API models
//!
//! This module contains the request/response types for the Qdrant Snapshots API.

use serde::{Deserialize, Serialize};

/// Snapshot description (matches Qdrant API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSnapshotDescription {
    /// Snapshot name (unique identifier)
    pub name: String,
    /// Creation time in RFC 3339 format
    pub creation_time: Option<String>,
    /// Size in bytes
    pub size: u64,
    /// Optional checksum
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Response for list snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantListSnapshotsResponse {
    pub result: Vec<QdrantSnapshotDescription>,
    pub status: String,
    pub time: f64,
}

/// Response for create snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCreateSnapshotResponse {
    pub result: QdrantSnapshotDescription,
    pub status: String,
    pub time: f64,
}

/// Response for delete snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeleteSnapshotResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}

/// Request for creating snapshot (optional wait parameter)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantCreateSnapshotRequest {
    /// Whether to wait for the snapshot to complete
    #[serde(default)]
    pub wait: Option<bool>,
}

/// Request for recovering from snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecoverSnapshotRequest {
    /// Location of the snapshot (URL or local path)
    pub location: String,
    /// Optional priority for recovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    /// Optional checksum for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Response for recover snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecoverSnapshotResponse {
    pub result: bool,
    pub status: String,
    pub time: f64,
}
