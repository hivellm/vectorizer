//! Qdrant point models
//!
//! This module provides data structures for Qdrant point operations,
//! including point data, payload, and vector information.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Qdrant point structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPointStruct {
    /// Point ID
    pub id: QdrantPointId,
    /// Point vector
    pub vector: QdrantVector,
    /// Point payload
    pub payload: Option<HashMap<String, QdrantValue>>,
}

/// Point ID
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantPointId {
    /// Numeric ID
    Numeric(u64),
    /// UUID string
    Uuid(String),
}

/// Vector data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantVector {
    /// Dense vector
    Dense(Vec<f32>),
    /// Named vectors
    Named(HashMap<String, Vec<f32>>),
}

/// Payload value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<QdrantValue>),
    /// Object value
    Object(HashMap<String, QdrantValue>),
    /// Null value
    Null,
}

/// Point upsert request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantUpsertPointsRequest {
    /// Points to upsert
    pub points: Vec<QdrantPointStruct>,
    /// Wait for completion
    pub wait: Option<bool>,
}

/// Point delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeletePointsRequest {
    /// Points to delete
    pub points: Vec<QdrantPointId>,
    /// Wait for completion
    pub wait: Option<bool>,
}

/// Point retrieve request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRetrievePointsRequest {
    /// Point IDs to retrieve
    pub ids: Vec<QdrantPointId>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
}

/// Point scroll request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollPointsRequest {
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<QdrantPointId>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
}

/// Point count request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountPointsRequest {
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Exact count
    pub exact: Option<bool>,
}

// Re-export filter types from filter module to avoid duplication
pub use super::filter::{QdrantCondition, QdrantFilter};

/// Match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantMatch {
    /// Value match
    Value(QdrantValue),
    /// Text match
    Text(QdrantTextMatch),
    /// Any match
    Any(QdrantAnyMatch),
    /// Except match
    Except(QdrantExceptMatch),
}

/// Text match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantTextMatch {
    /// Text to match
    pub text: String,
    /// Type of match
    pub r#type: QdrantTextMatchType,
}

/// Text match type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantTextMatchType {
    /// Exact match
    #[serde(rename = "exact")]
    Exact,
    /// Word match
    #[serde(rename = "word")]
    Word,
    /// Prefix match
    #[serde(rename = "prefix")]
    Prefix,
}

/// Any match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantAnyMatch {
    /// Values to match
    pub any: Vec<QdrantValue>,
}

/// Except match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantExceptMatch {
    /// Values to exclude
    pub except: Vec<QdrantValue>,
}

/// Is null condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantIsNull {
    /// Field name
    pub key: String,
}

/// Is empty condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantIsEmpty {
    /// Field name
    pub key: String,
}

/// Point upsert response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantUpsertPointsResponse {
    /// Operation ID
    pub operation_id: u64,
    /// Operation status
    pub status: QdrantOperationStatus,
}

/// Point delete response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeletePointsResponse {
    /// Operation ID
    pub operation_id: u64,
    /// Operation status
    pub status: QdrantOperationStatus,
}

/// Point retrieve response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRetrievePointsResponse {
    /// Retrieved points
    pub result: Vec<QdrantPointStruct>,
}

/// Point scroll response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollPointsResponse {
    /// Retrieved points
    pub result: QdrantScrollResult,
}

/// Scroll result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollResult {
    /// Points
    pub points: Vec<QdrantPointStruct>,
    /// Next page offset
    pub next_page_offset: Option<QdrantPointId>,
}

/// Point count response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountPointsResponse {
    /// Count result
    pub result: QdrantCountResult,
}

/// Count result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountResult {
    /// Number of points
    pub count: u64,
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

/// Point upsert request (alias for compatibility)
pub type QdrantPointUpsertRequest = QdrantUpsertPointsRequest;

/// Point operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPointOperationResult {
    /// Operation status
    pub status: QdrantOperationStatus,
    /// Operation ID
    pub operation_id: Option<u64>,
}

/// Point retrieve request (alias for compatibility)
pub type QdrantPointRetrieveRequest = QdrantRetrievePointsRequest;

/// Point retrieve response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPointRetrieveResponse {
    /// Retrieved points
    pub result: Vec<QdrantPointStruct>,
}

/// Point delete request (alias for compatibility)
pub type QdrantPointDeleteRequest = QdrantDeletePointsRequest;

/// Point scroll request (alias for compatibility)
pub type QdrantPointScrollRequest = QdrantScrollPointsRequest;

/// Point scroll response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPointScrollResponse {
    /// Scroll result
    pub result: QdrantScrollResult,
}

/// Point count request (alias for compatibility)
pub type QdrantPointCountRequest = QdrantCountPointsRequest;

/// Point count response (alias for compatibility)
pub type QdrantPointCountResponse = QdrantCountPointsResponse;
