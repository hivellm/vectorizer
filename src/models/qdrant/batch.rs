//! Qdrant batch operation models
//!
//! This module provides data structures for Qdrant batch operations,
//! including batch requests and responses.

use serde::{Deserialize, Serialize};

/// Batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum QdrantBatchOperation {
    /// Upsert points
    #[serde(rename = "upsert")]
    Upsert {
        collection: String,
        points: Vec<QdrantPointStruct>,
    },
    /// Delete points
    #[serde(rename = "delete")]
    Delete {
        collection: String,
        points: Vec<QdrantPointId>,
    },
    /// Set payload
    #[serde(rename = "set_payload")]
    SetPayload {
        collection: String,
        payload: QdrantPayload,
        points: Vec<QdrantPointId>,
    },
    /// Overwrite payload
    #[serde(rename = "overwrite_payload")]
    OverwritePayload {
        collection: String,
        payload: QdrantPayload,
        points: Vec<QdrantPointId>,
    },
    /// Delete payload
    #[serde(rename = "delete_payload")]
    DeletePayload {
        collection: String,
        keys: Vec<String>,
        points: Vec<QdrantPointId>,
    },
    /// Clear payload
    #[serde(rename = "clear_payload")]
    ClearPayload {
        collection: String,
        points: Vec<QdrantPointId>,
    },
    /// Update vectors
    #[serde(rename = "update_vectors")]
    UpdateVectors {
        collection: String,
        points: Vec<QdrantPointStruct>,
    },
    /// Delete vectors
    #[serde(rename = "delete_vectors")]
    DeleteVectors {
        collection: String,
        points: Vec<QdrantPointId>,
        vector: Option<String>,
    },
}

/// Point ID (re-exported from point module)
pub use super::point::QdrantPointId;
/// Point struct (re-exported from point module)
pub use super::point::QdrantPointStruct;
/// Payload value (re-exported from point module)
pub use super::point::QdrantValue;

/// Payload
pub type QdrantPayload = std::collections::HashMap<String, QdrantValue>;

/// Batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchRequest {
    /// Operations to perform
    pub operations: Vec<QdrantBatchOperation>,
    /// Wait for completion
    pub wait: Option<bool>,
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchResponse {
    /// Operation results
    pub result: Vec<QdrantBatchOperationResult>,
}

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchOperationResult {
    /// Operation ID
    pub operation_id: u64,
    /// Operation status
    pub status: QdrantOperationStatus,
}

/// Operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantOperationStatus {
    /// Acknowledged
    #[serde(rename = "acknowledged")]
    Acknowledged,
    /// Completed
    #[serde(rename = "completed")]
    Completed,
}
