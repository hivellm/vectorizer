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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qdrant_vector_dense_serialization() {
        let vector = QdrantVector::Dense(vec![0.1, 0.2, 0.3, 0.4]);
        let json = serde_json::to_string(&vector).unwrap();
        assert!(json.contains("0.1"));
        assert!(json.contains("0.4"));

        // Deserialize back
        let deserialized: QdrantVector = serde_json::from_str(&json).unwrap();
        match deserialized {
            QdrantVector::Dense(data) => {
                assert_eq!(data.len(), 4);
                assert!((data[0] - 0.1).abs() < 0.001);
            }
            _ => panic!("Expected Dense vector"),
        }
    }

    #[test]
    fn test_qdrant_vector_named_serialization() {
        let mut named = HashMap::new();
        named.insert("text".to_string(), vec![0.1, 0.2, 0.3]);
        named.insert("image".to_string(), vec![0.4, 0.5, 0.6]);

        let vector = QdrantVector::Named(named);
        let json = serde_json::to_string(&vector).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("image"));

        // Deserialize back
        let deserialized: QdrantVector = serde_json::from_str(&json).unwrap();
        match deserialized {
            QdrantVector::Named(data) => {
                assert_eq!(data.len(), 2);
                assert!(data.contains_key("text"));
                assert!(data.contains_key("image"));
            }
            _ => panic!("Expected Named vector"),
        }
    }

    #[test]
    fn test_qdrant_vector_named_single_deserialization() {
        // Test that a single named vector deserializes correctly
        let json = r#"{"default": [0.1, 0.2, 0.3, 0.4]}"#;
        let vector: QdrantVector = serde_json::from_str(json).unwrap();
        match vector {
            QdrantVector::Named(data) => {
                assert_eq!(data.len(), 1);
                assert!(data.contains_key("default"));
                assert_eq!(data.get("default").unwrap().len(), 4);
            }
            _ => panic!("Expected Named vector"),
        }
    }

    #[test]
    fn test_qdrant_point_struct_with_dense_vector() {
        let point = QdrantPointStruct {
            id: QdrantPointId::Uuid("test-id".to_string()),
            vector: QdrantVector::Dense(vec![0.1, 0.2, 0.3]),
            payload: None,
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("0.1"));

        let deserialized: QdrantPointStruct = serde_json::from_str(&json).unwrap();
        match deserialized.id {
            QdrantPointId::Uuid(id) => assert_eq!(id, "test-id"),
            _ => panic!("Expected UUID"),
        }
    }

    #[test]
    fn test_qdrant_point_struct_with_named_vector() {
        let mut named = HashMap::new();
        named.insert("embedding".to_string(), vec![0.1, 0.2, 0.3]);

        let mut payload = HashMap::new();
        payload.insert("text".to_string(), QdrantValue::String("hello".to_string()));

        let point = QdrantPointStruct {
            id: QdrantPointId::Numeric(42),
            vector: QdrantVector::Named(named),
            payload: Some(payload),
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("embedding"));
        assert!(json.contains("hello"));

        let deserialized: QdrantPointStruct = serde_json::from_str(&json).unwrap();
        match deserialized.id {
            QdrantPointId::Numeric(id) => assert_eq!(id, 42),
            _ => panic!("Expected Numeric ID"),
        }
        match deserialized.vector {
            QdrantVector::Named(data) => {
                assert!(data.contains_key("embedding"));
            }
            _ => panic!("Expected Named vector"),
        }
    }

    #[test]
    fn test_qdrant_upsert_request_with_named_vectors() {
        let mut named = HashMap::new();
        named.insert("dense".to_string(), vec![0.1, 0.2, 0.3, 0.4]);

        let request = QdrantUpsertPointsRequest {
            points: vec![
                QdrantPointStruct {
                    id: QdrantPointId::Uuid("point-1".to_string()),
                    vector: QdrantVector::Named(named.clone()),
                    payload: None,
                },
                QdrantPointStruct {
                    id: QdrantPointId::Numeric(2),
                    vector: QdrantVector::Dense(vec![0.5, 0.6, 0.7, 0.8]),
                    payload: None,
                },
            ],
            wait: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("point-1"));
        assert!(json.contains("dense"));

        let deserialized: QdrantUpsertPointsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.points.len(), 2);
        assert_eq!(deserialized.wait, Some(true));
    }

    #[test]
    fn test_qdrant_value_types() {
        // Test all value types
        let values = vec![
            (QdrantValue::String("test".to_string()), r#""test""#),
            (QdrantValue::Integer(42), "42"),
            (QdrantValue::Float(3.15), "3.15"),
            (QdrantValue::Boolean(true), "true"),
            (QdrantValue::Null, "null"),
        ];

        for (value, expected_substr) in values {
            let json = serde_json::to_string(&value).unwrap();
            assert!(
                json.contains(expected_substr),
                "Expected {} in {}",
                expected_substr,
                json
            );
        }

        // Test array
        let array = QdrantValue::Array(vec![QdrantValue::Integer(1), QdrantValue::Integer(2)]);
        let json = serde_json::to_string(&array).unwrap();
        assert!(json.contains("[1,2]"));

        // Test object
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), QdrantValue::String("value".to_string()));
        let object = QdrantValue::Object(obj);
        let json = serde_json::to_string(&object).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[test]
    fn test_qdrant_point_id_types() {
        // Numeric ID
        let numeric = QdrantPointId::Numeric(12345);
        let json = serde_json::to_string(&numeric).unwrap();
        assert_eq!(json, "12345");

        // UUID ID
        let uuid = QdrantPointId::Uuid("550e8400-e29b-41d4-a716-446655440000".to_string());
        let json = serde_json::to_string(&uuid).unwrap();
        assert!(json.contains("550e8400"));
    }
}
