//! Qdrant API Compatibility Tests
//!
//! Tests for Qdrant API compatibility features:
//! - Quantization Configuration API (Scalar, Product, Binary)
//! - Cluster Models
//! - Point ID Handling

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Common Types for Qdrant API Testing
// ============================================================================

/// Qdrant-style API response wrapper
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QdrantResponse<T> {
    pub result: T,
    pub status: String,
    pub time: f64,
}

/// Point structure for Qdrant API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct QdrantPoint {
    pub id: Value,
    pub vector: Option<Vec<f32>>,
    pub payload: Option<HashMap<String, Value>>,
}

/// Search result structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ScoredPoint {
    pub id: Value,
    pub score: f32,
    pub payload: Option<HashMap<String, Value>>,
}

// ============================================================================
// Unit Tests for Qdrant Quantization API Models
// ============================================================================

#[cfg(test)]
mod quantization_model_tests {
    use vectorizer::models::qdrant::{
        QdrantBinaryQuantization, QdrantBinaryQuantizationConfig, QdrantPQCompression,
        QdrantProductQuantization, QdrantProductQuantizationConfig, QdrantQuantizationConfig,
        QdrantScalarQuantization, QdrantScalarQuantizationConfig, QdrantScalarQuantizationType,
    };

    #[test]
    fn test_scalar_quantization_serialization() {
        let config = QdrantQuantizationConfig::Scalar(QdrantScalarQuantizationConfig {
            scalar: QdrantScalarQuantization {
                r#type: QdrantScalarQuantizationType::Int8,
                quantile: Some(0.99),
                always_ram: Some(true),
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("scalar"));
        assert!(json.contains("int8"));
        assert!(json.contains("0.99"));
    }

    #[test]
    fn test_scalar_quantization_deserialization() {
        let json = r#"{"scalar":{"type":"int8","quantile":0.95}}"#;
        let config: QdrantQuantizationConfig = serde_json::from_str(json).unwrap();

        match config {
            QdrantQuantizationConfig::Scalar(scalar_config) => {
                assert!(matches!(
                    scalar_config.scalar.r#type,
                    QdrantScalarQuantizationType::Int8
                ));
                assert_eq!(scalar_config.scalar.quantile, Some(0.95));
            }
            _ => panic!("Expected Scalar quantization config"),
        }
    }

    #[test]
    fn test_product_quantization_serialization() {
        let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
            product: QdrantProductQuantization {
                compression: Some(QdrantPQCompression::X16),
                always_ram: Some(false),
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("product"));
        assert!(json.contains("x16"));
    }

    #[test]
    fn test_product_quantization_deserialization() {
        let json = r#"{"product":{"compression":"x32","always_ram":true}}"#;
        let config: QdrantQuantizationConfig = serde_json::from_str(json).unwrap();

        match config {
            QdrantQuantizationConfig::Product(product_config) => {
                assert!(matches!(
                    product_config.product.compression,
                    Some(QdrantPQCompression::X32)
                ));
                assert_eq!(product_config.product.always_ram, Some(true));
            }
            _ => panic!("Expected Product quantization config"),
        }
    }

    #[test]
    fn test_binary_quantization_serialization() {
        let config = QdrantQuantizationConfig::Binary(QdrantBinaryQuantizationConfig {
            binary: QdrantBinaryQuantization {
                always_ram: Some(true),
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("binary"));
        assert!(json.contains("always_ram"));
    }

    #[test]
    fn test_binary_quantization_deserialization() {
        let json = r#"{"binary":{"always_ram":false}}"#;
        let config: QdrantQuantizationConfig = serde_json::from_str(json).unwrap();

        match config {
            QdrantQuantizationConfig::Binary(binary_config) => {
                assert_eq!(binary_config.binary.always_ram, Some(false));
            }
            _ => panic!("Expected Binary quantization config"),
        }
    }

    #[test]
    fn test_all_pq_compression_levels() {
        let compression_levels = vec![
            (QdrantPQCompression::X4, "x4"),
            (QdrantPQCompression::X8, "x8"),
            (QdrantPQCompression::X16, "x16"),
            (QdrantPQCompression::X32, "x32"),
            (QdrantPQCompression::X64, "x64"),
        ];

        for (compression, expected_str) in compression_levels {
            let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
                product: QdrantProductQuantization {
                    compression: Some(compression),
                    always_ram: None,
                },
            });

            let json = serde_json::to_string(&config).unwrap();
            assert!(
                json.contains(expected_str),
                "Expected {expected_str} in JSON: {json}"
            );
        }
    }

    #[test]
    fn test_quantization_config_none_values() {
        let config = QdrantQuantizationConfig::Scalar(QdrantScalarQuantizationConfig {
            scalar: QdrantScalarQuantization {
                r#type: QdrantScalarQuantizationType::Int8,
                quantile: None,
                always_ram: None,
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("int8"));
        // Optional fields with None should not appear in JSON with skip_serializing_if
    }

    #[test]
    fn test_pq_default_compression() {
        let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
            product: QdrantProductQuantization {
                compression: None,
                always_ram: None,
            },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("product"));
    }

    #[test]
    fn test_binary_quantization_minimal() {
        let config = QdrantQuantizationConfig::Binary(QdrantBinaryQuantizationConfig {
            binary: QdrantBinaryQuantization { always_ram: None },
        });

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("binary"));
    }
}

// ============================================================================
// Unit Tests for Cluster Models
// ============================================================================

#[cfg(test)]
mod cluster_model_tests {
    use std::collections::HashMap;
    use vectorizer::models::qdrant::cluster::{
        QdrantClusterStatus, QdrantPeerInfo, QdrantPeerState, QdrantRaftInfo,
    };

    #[test]
    fn test_cluster_status_serialization() {
        let mut peers = HashMap::new();
        peers.insert(
            "1".to_string(),
            QdrantPeerInfo {
                uri: "http://localhost:7777".to_string(),
                state: Some(QdrantPeerState::Active),
            },
        );

        let status = QdrantClusterStatus {
            status: "enabled".to_string(),
            peer_id: 1,
            peers,
            raft_info: Some(QdrantRaftInfo {
                term: 1,
                commit: 0,
                pending_operations: 0,
                leader: Some(1),
                role: Some("Leader".to_string()),
                is_voter: true,
            }),
            consensus_thread_status: None,
            message_send_failures: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("enabled"));
        assert!(json.contains("peer_id"));
        assert!(json.contains("Leader"));
    }

    #[test]
    fn test_peer_state_serialization() {
        let active = QdrantPeerState::Active;
        let json = serde_json::to_string(&active).unwrap();
        assert!(json.contains("Active") || json.contains("active"));
    }

    #[test]
    fn test_cluster_status_minimal() {
        let status = QdrantClusterStatus {
            status: "disabled".to_string(),
            peer_id: 0,
            peers: HashMap::new(),
            raft_info: None,
            consensus_thread_status: None,
            message_send_failures: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("disabled"));
    }

    #[test]
    fn test_raft_info_serialization() {
        let raft_info = QdrantRaftInfo {
            term: 5,
            commit: 100,
            pending_operations: 2,
            leader: Some(1),
            role: Some("Follower".to_string()),
            is_voter: false,
        };

        let json = serde_json::to_string(&raft_info).unwrap();
        assert!(json.contains("\"term\":5"));
        assert!(json.contains("\"commit\":100"));
        assert!(json.contains("Follower"));
    }
}

// ============================================================================
// Unit Tests for Point ID Handling
// ============================================================================

#[cfg(test)]
mod point_id_tests {
    use serde_json::json;
    use vectorizer::models::qdrant::point::QdrantPointId;

    #[test]
    fn test_numeric_point_id() {
        let id: QdrantPointId = serde_json::from_value(json!(123)).unwrap();
        match id {
            QdrantPointId::Numeric(n) => assert_eq!(n, 123),
            QdrantPointId::Uuid(_) => panic!("Expected numeric ID"),
        }
    }

    #[test]
    fn test_string_point_id() {
        let id: QdrantPointId = serde_json::from_value(json!("abc-123")).unwrap();
        match id {
            QdrantPointId::Uuid(s) => assert_eq!(s, "abc-123"),
            QdrantPointId::Numeric(_) => panic!("Expected string ID"),
        }
    }

    #[test]
    fn test_point_id_serialization() {
        let num_id = QdrantPointId::Numeric(456);
        let json = serde_json::to_value(&num_id).unwrap();
        assert_eq!(json, json!(456));

        let str_id = QdrantPointId::Uuid("def-789".to_string());
        let json = serde_json::to_value(&str_id).unwrap();
        assert_eq!(json, json!("def-789"));
    }

    #[test]
    fn test_uuid_point_id() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id: QdrantPointId = serde_json::from_value(json!(uuid)).unwrap();
        match id {
            QdrantPointId::Uuid(s) => assert_eq!(s, uuid),
            QdrantPointId::Numeric(_) => panic!("Expected UUID string ID"),
        }
    }

    #[test]
    fn test_large_numeric_point_id() {
        let id: QdrantPointId = serde_json::from_value(json!(9999999999i64)).unwrap();
        match id {
            QdrantPointId::Numeric(n) => assert_eq!(n, 9999999999),
            QdrantPointId::Uuid(_) => panic!("Expected large numeric ID"),
        }
    }
}

// ============================================================================
// Unit Tests for Collection Distance Metrics
// ============================================================================

#[cfg(test)]
mod distance_metric_tests {
    use vectorizer::models::qdrant::collection::QdrantDistance;

    #[test]
    fn test_cosine_distance_serialization() {
        let distance = QdrantDistance::Cosine;
        let json = serde_json::to_string(&distance).unwrap();
        assert_eq!(json, r#""Cosine""#);
    }

    #[test]
    fn test_euclid_distance_serialization() {
        let distance = QdrantDistance::Euclid;
        let json = serde_json::to_string(&distance).unwrap();
        assert_eq!(json, r#""Euclid""#);
    }

    #[test]
    fn test_dot_distance_serialization() {
        let distance = QdrantDistance::Dot;
        let json = serde_json::to_string(&distance).unwrap();
        assert_eq!(json, r#""Dot""#);
    }

    #[test]
    fn test_distance_deserialization() {
        let cosine: QdrantDistance = serde_json::from_str(r#""Cosine""#).unwrap();
        assert!(matches!(cosine, QdrantDistance::Cosine));

        let euclid: QdrantDistance = serde_json::from_str(r#""Euclid""#).unwrap();
        assert!(matches!(euclid, QdrantDistance::Euclid));

        let dot: QdrantDistance = serde_json::from_str(r#""Dot""#).unwrap();
        assert!(matches!(dot, QdrantDistance::Dot));
    }
}

// ============================================================================
// Unit Tests for Collection Status
// ============================================================================

#[cfg(test)]
mod collection_status_tests {
    use vectorizer::models::qdrant::collection::QdrantCollectionStatus;

    #[test]
    fn test_green_status_serialization() {
        let status = QdrantCollectionStatus::Green;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""green""#);
    }

    #[test]
    fn test_yellow_status_serialization() {
        let status = QdrantCollectionStatus::Yellow;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""yellow""#);
    }

    #[test]
    fn test_red_status_serialization() {
        let status = QdrantCollectionStatus::Red;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""red""#);
    }

    #[test]
    fn test_status_deserialization() {
        let green: QdrantCollectionStatus = serde_json::from_str(r#""green""#).unwrap();
        assert!(matches!(green, QdrantCollectionStatus::Green));

        let yellow: QdrantCollectionStatus = serde_json::from_str(r#""yellow""#).unwrap();
        assert!(matches!(yellow, QdrantCollectionStatus::Yellow));

        let red: QdrantCollectionStatus = serde_json::from_str(r#""red""#).unwrap();
        assert!(matches!(red, QdrantCollectionStatus::Red));
    }
}

// ============================================================================
// Unit Tests for Optimizer Status
// ============================================================================

#[cfg(test)]
mod optimizer_status_tests {
    use vectorizer::models::qdrant::collection::QdrantOptimizerStatus;

    #[test]
    fn test_optimizer_ok_status() {
        let status = QdrantOptimizerStatus {
            ok: true,
            error: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"ok\":true"));
    }

    #[test]
    fn test_optimizer_error_status() {
        let status = QdrantOptimizerStatus {
            ok: false,
            error: Some("Optimization failed: disk full".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("disk full"));
    }
}

// ============================================================================
// Unit Tests for Snapshot Models
// ============================================================================

#[cfg(test)]
mod snapshot_model_tests {
    use vectorizer::models::qdrant::snapshot::{
        QdrantCreateSnapshotRequest, QdrantListSnapshotsResponse, QdrantRecoverSnapshotRequest,
        QdrantSnapshotDescription,
    };

    #[test]
    fn test_snapshot_description_serialization() {
        let snapshot = QdrantSnapshotDescription {
            name: "snapshot_20240101_120000".to_string(),
            creation_time: Some("2024-01-01T12:00:00Z".to_string()),
            size: 1024 * 1024, // 1MB
            checksum: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("snapshot_20240101_120000"));
        assert!(json.contains("2024-01-01T12:00:00Z"));
        assert!(json.contains("1048576"));
    }

    #[test]
    fn test_snapshot_description_without_optional() {
        let snapshot = QdrantSnapshotDescription {
            name: "minimal_snapshot".to_string(),
            creation_time: None,
            size: 512,
            checksum: None,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("minimal_snapshot"));
        assert!(json.contains("512"));
    }

    #[test]
    fn test_list_snapshots_response() {
        let response = QdrantListSnapshotsResponse {
            result: vec![
                QdrantSnapshotDescription {
                    name: "snap1".to_string(),
                    creation_time: None,
                    size: 100,
                    checksum: None,
                },
                QdrantSnapshotDescription {
                    name: "snap2".to_string(),
                    creation_time: None,
                    size: 200,
                    checksum: None,
                },
            ],
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("snap1"));
        assert!(json.contains("snap2"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_create_snapshot_request() {
        let request = QdrantCreateSnapshotRequest { wait: Some(true) };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"wait\":true"));
    }

    #[test]
    fn test_recover_snapshot_request() {
        let request = QdrantRecoverSnapshotRequest {
            location: "s3://bucket/snapshot.tar".to_string(),
            priority: Some("high".to_string()),
            checksum: Some("sha256:abc123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("s3://bucket/snapshot.tar"));
        assert!(json.contains("high"));
        assert!(json.contains("sha256:abc123"));
    }
}

// ============================================================================
// Unit Tests for Sharding Models
// ============================================================================

#[cfg(test)]
mod sharding_model_tests {
    use vectorizer::models::qdrant::sharding::{
        QdrantCreateShardKeyRequest, QdrantDeleteShardKeyRequest, QdrantLocalShardInfo,
        QdrantRemoteShardInfo, QdrantShardKeyValue, QdrantShardState,
    };

    #[test]
    fn test_shard_key_string() {
        let key = QdrantShardKeyValue::String("tenant_1".to_string());
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, "\"tenant_1\"");
    }

    #[test]
    fn test_shard_key_integer() {
        let key = QdrantShardKeyValue::Integer(42);
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_shard_key_deserialization() {
        let string_key: QdrantShardKeyValue = serde_json::from_str("\"region_us\"").unwrap();
        match string_key {
            QdrantShardKeyValue::String(s) => assert_eq!(s, "region_us"),
            QdrantShardKeyValue::Integer(_) => panic!("Expected string shard key"),
        }

        let int_key: QdrantShardKeyValue = serde_json::from_str("123").unwrap();
        match int_key {
            QdrantShardKeyValue::Integer(i) => assert_eq!(i, 123),
            QdrantShardKeyValue::String(_) => panic!("Expected integer shard key"),
        }
    }

    #[test]
    fn test_create_shard_key_request() {
        let request = QdrantCreateShardKeyRequest {
            shard_key: QdrantShardKeyValue::String("new_tenant".to_string()),
            shards_number: Some(4),
            replication_factor: Some(2),
            placement: Some(vec![1, 2, 3]),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("new_tenant"));
        assert!(json.contains("\"shards_number\":4"));
        assert!(json.contains("\"replication_factor\":2"));
    }

    #[test]
    fn test_delete_shard_key_request() {
        let request = QdrantDeleteShardKeyRequest {
            shard_key: QdrantShardKeyValue::Integer(99),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("99"));
    }

    #[test]
    fn test_shard_state_serialization() {
        let states = vec![
            (QdrantShardState::Active, "Active"),
            (QdrantShardState::Dead, "Dead"),
            (QdrantShardState::Initializing, "Initializing"),
            (QdrantShardState::Recovering, "Recovering"),
            (QdrantShardState::Partial, "Partial"),
        ];

        for (state, expected) in states {
            let json = serde_json::to_string(&state).unwrap();
            assert!(json.contains(expected), "Expected {expected} in {json}");
        }
    }

    #[test]
    fn test_local_shard_info() {
        let info = QdrantLocalShardInfo {
            shard_id: 1,
            points_count: 10000,
            state: QdrantShardState::Active,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"shard_id\":1"));
        assert!(json.contains("\"points_count\":10000"));
        assert!(json.contains("Active"));
    }

    #[test]
    fn test_remote_shard_info() {
        let info = QdrantRemoteShardInfo {
            shard_id: 2,
            peer_id: 100,
            state: QdrantShardState::Recovering,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"shard_id\":2"));
        assert!(json.contains("\"peer_id\":100"));
        assert!(json.contains("Recovering"));
    }
}

// ============================================================================
// Unit Tests for Search/Query Models
// ============================================================================

#[cfg(test)]
mod search_model_tests {
    use std::collections::HashMap;
    use vectorizer::models::qdrant::point::QdrantPointId;
    use vectorizer::models::qdrant::search::{
        QdrantDirection, QdrantFusionMethod, QdrantPrefetch, QdrantQuery, QdrantRecommendStrategy,
        QdrantScoredPoint, QdrantSearchParams, QdrantWithPayload, QdrantWithVector,
    };

    #[test]
    fn test_recommend_strategy_serialization() {
        let avg = QdrantRecommendStrategy::AverageVector;
        let best = QdrantRecommendStrategy::BestScore;

        let avg_json = serde_json::to_string(&avg).unwrap();
        let best_json = serde_json::to_string(&best).unwrap();

        assert!(avg_json.contains("average_vector"));
        assert!(best_json.contains("best_score"));
    }

    #[test]
    fn test_direction_serialization() {
        let asc = QdrantDirection::Asc;
        let desc = QdrantDirection::Desc;

        let asc_json = serde_json::to_string(&asc).unwrap();
        let desc_json = serde_json::to_string(&desc).unwrap();

        assert!(asc_json.contains("asc"));
        assert!(desc_json.contains("desc"));
    }

    #[test]
    fn test_fusion_method_serialization() {
        let rrf = QdrantFusionMethod::Rrf;
        let dbsf = QdrantFusionMethod::Dbsf;

        let rrf_json = serde_json::to_string(&rrf).unwrap();
        let dbsf_json = serde_json::to_string(&dbsf).unwrap();

        assert!(rrf_json.contains("rrf"));
        assert!(dbsf_json.contains("dbsf"));
    }

    #[test]
    fn test_with_payload_bool() {
        let with = QdrantWithPayload::Bool(true);
        let json = serde_json::to_string(&with).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn test_with_payload_include() {
        let include = QdrantWithPayload::Include(vec!["field1".to_string(), "field2".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("field1"));
        assert!(json.contains("field2"));
    }

    #[test]
    fn test_with_vector_bool() {
        let with = QdrantWithVector::Bool(false);
        let json = serde_json::to_string(&with).unwrap();
        assert_eq!(json, "false");
    }

    #[test]
    fn test_with_vector_include() {
        let include = QdrantWithVector::Include(vec!["dense".to_string(), "sparse".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("dense"));
        assert!(json.contains("sparse"));
    }

    #[test]
    fn test_search_params() {
        let params = QdrantSearchParams {
            hnsw_ef: Some(128),
            exact: Some(false),
            quantization: None,
            indexed_only: Some(true),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"hnsw_ef\":128"));
        assert!(json.contains("\"exact\":false"));
        assert!(json.contains("\"indexed_only\":true"));
    }

    #[test]
    fn test_scored_point() {
        let point = QdrantScoredPoint {
            id: QdrantPointId::Numeric(123),
            vector: None,
            payload: Some(HashMap::new()),
            score: 0.95,
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_query_vector() {
        let query = QdrantQuery::Vector(vec![0.1, 0.2, 0.3]);
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("0.1"));
        assert!(json.contains("0.2"));
        assert!(json.contains("0.3"));
    }

    #[test]
    fn test_query_point_id() {
        let query = QdrantQuery::PointId(QdrantPointId::Uuid("abc-123".to_string()));
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("abc-123"));
    }

    #[test]
    fn test_prefetch_basic() {
        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.5, 0.5])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: None,
            score_threshold: Some(0.5),
            params: None,
        };

        let json = serde_json::to_string(&prefetch).unwrap();
        assert!(json.contains("0.5"));
        assert!(json.contains("\"limit\":100"));
        assert!(json.contains("dense"));
    }

    #[test]
    fn test_prefetch_nested() {
        // Test nested prefetch (prefetch within prefetch)
        let inner_prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3])),
            filter: None,
            limit: Some(50),
            using: Some("sparse".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let outer_prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.4, 0.5, 0.6])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: Some(vec![inner_prefetch]),
            score_threshold: Some(0.7),
            params: None,
        };

        let json = serde_json::to_string(&outer_prefetch).unwrap();
        assert!(json.contains("prefetch"));
        assert!(json.contains("sparse"));
        assert!(json.contains("dense"));
        assert!(json.contains("\"limit\":50"));
        assert!(json.contains("\"limit\":100"));
    }

    #[test]
    fn test_prefetch_with_params() {
        let params = QdrantSearchParams {
            hnsw_ef: Some(128),
            exact: Some(false),
            quantization: None,
            indexed_only: Some(true),
        };

        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            filter: None,
            limit: Some(200),
            using: None,
            prefetch: None,
            score_threshold: None,
            params: Some(params),
        };

        let json = serde_json::to_string(&prefetch).unwrap();
        assert!(json.contains("hnsw_ef"));
        assert!(json.contains("128"));
        assert!(json.contains("indexed_only"));
    }

    #[test]
    fn test_prefetch_multiple_stages() {
        // Test multiple prefetch stages (array of prefetch)
        let prefetch1 = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let prefetch2 = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.3, 0.4])),
            filter: None,
            limit: Some(50),
            using: Some("sparse".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let prefetch_array = vec![prefetch1, prefetch2];
        let json = serde_json::to_string(&prefetch_array).unwrap();

        assert!(json.contains("dense"));
        assert!(json.contains("sparse"));
        // Should be an array
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }
}

// ============================================================================
// Unit Tests for Query API Models
// ============================================================================

#[cfg(test)]
mod query_api_model_tests {
    use vectorizer::models::qdrant::point::QdrantPointId;
    use vectorizer::models::qdrant::search::{
        QdrantBatchQueryRequest, QdrantPrefetch, QdrantQuery, QdrantQueryGroupsRequest,
        QdrantQueryRequest, QdrantWithPayload, QdrantWithVector,
    };

    #[test]
    fn test_query_request_simple_vector() {
        let request = QdrantQueryRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3, 0.4])),
            prefetch: None,
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: Some(QdrantWithVector::Bool(false)),
            score_threshold: Some(0.5),
            using: None,
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("0.1"));
        assert!(json.contains("\"limit\":10"));
        assert!(json.contains("\"score_threshold\":0.5"));
    }

    #[test]
    fn test_query_request_with_point_id() {
        let request = QdrantQueryRequest {
            query: Some(QdrantQuery::PointId(QdrantPointId::Numeric(123))),
            prefetch: None,
            filter: None,
            limit: Some(5),
            offset: None,
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: Some("default".to_string()),
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("\"using\":\"default\""));
    }

    #[test]
    fn test_query_request_with_prefetch() {
        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.5, 0.5])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let request = QdrantQueryRequest {
            query: None, // No query when using prefetch with fusion
            prefetch: Some(vec![prefetch]),
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("prefetch"));
        assert!(json.contains("dense"));
    }

    #[test]
    fn test_batch_query_request() {
        let query1 = QdrantQueryRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            prefetch: None,
            filter: None,
            limit: Some(5),
            offset: None,
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            params: None,
        };

        let query2 = QdrantQueryRequest {
            query: Some(QdrantQuery::Vector(vec![0.3, 0.4])),
            prefetch: None,
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            params: None,
        };

        let batch = QdrantBatchQueryRequest {
            searches: vec![query1, query2],
        };

        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains("searches"));
        assert!(json.contains("0.1"));
        assert!(json.contains("0.3"));
    }

    #[test]
    fn test_query_groups_request() {
        let request = QdrantQueryGroupsRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3])),
            prefetch: None,
            filter: None,
            group_by: "category".to_string(),
            group_size: Some(3),
            limit: Some(10),
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            with_lookup: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"group_by\":\"category\""));
        assert!(json.contains("\"group_size\":3"));
        assert!(json.contains("\"limit\":10"));
    }

    #[test]
    fn test_with_payload_selector() {
        let include =
            QdrantWithPayload::Include(vec!["name".to_string(), "description".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("name"));
        assert!(json.contains("description"));
    }

    #[test]
    fn test_with_vector_selector() {
        let include = QdrantWithVector::Include(vec!["dense".to_string(), "sparse".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("dense"));
        assert!(json.contains("sparse"));
    }
}

// ============================================================================
// Unit Tests for Quantization API Configuration
// ============================================================================

#[cfg(test)]
mod quantization_api_tests {
    use serde_json::json;
    use vectorizer::models::qdrant::search::QdrantQuantizationSearchParams;
    use vectorizer::models::qdrant::{
        QdrantBinaryQuantization, QdrantBinaryQuantizationConfig, QdrantPQCompression,
        QdrantProductQuantization, QdrantProductQuantizationConfig, QdrantQuantizationConfig,
        QdrantScalarQuantization, QdrantScalarQuantizationConfig, QdrantScalarQuantizationType,
    };

    #[test]
    fn test_quantization_config_in_collection_request() {
        // Simulate the request body for creating a collection with quantization
        let scalar_config = QdrantQuantizationConfig::Scalar(QdrantScalarQuantizationConfig {
            scalar: QdrantScalarQuantization {
                r#type: QdrantScalarQuantizationType::Int8,
                quantile: Some(0.99),
                always_ram: Some(true),
            },
        });

        let request_body = json!({
            "vectors": {
                "size": 128,
                "distance": "Cosine"
            },
            "quantization_config": scalar_config
        });

        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("quantization_config"));
        assert!(json_str.contains("scalar"));
        assert!(json_str.contains("int8"));
    }

    #[test]
    fn test_product_quantization_in_request() {
        let pq_config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
            product: QdrantProductQuantization {
                compression: Some(QdrantPQCompression::X16),
                always_ram: Some(false),
            },
        });

        let request_body = json!({
            "vectors": {
                "size": 256,
                "distance": "Euclid"
            },
            "quantization_config": pq_config
        });

        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("product"));
        assert!(json_str.contains("x16"));
    }

    #[test]
    fn test_binary_quantization_in_request() {
        let binary_config = QdrantQuantizationConfig::Binary(QdrantBinaryQuantizationConfig {
            binary: QdrantBinaryQuantization {
                always_ram: Some(true),
            },
        });

        let request_body = json!({
            "vectors": {
                "size": 1024,
                "distance": "Dot"
            },
            "quantization_config": binary_config
        });

        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("binary"));
        assert!(json_str.contains("always_ram"));
    }

    #[test]
    fn test_quantization_search_params() {
        let params = QdrantQuantizationSearchParams {
            ignore: Some(false),
            rescore: Some(true),
            oversampling: Some(2.0),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"ignore\":false"));
        assert!(json.contains("\"rescore\":true"));
        assert!(json.contains("\"oversampling\":2"));
    }

    #[test]
    fn test_quantization_search_params_minimal() {
        let params = QdrantQuantizationSearchParams {
            ignore: None,
            rescore: Some(true),
            oversampling: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("rescore"));
    }

    #[test]
    fn test_all_compression_levels_round_trip() {
        let compressions = vec![
            QdrantPQCompression::X4,
            QdrantPQCompression::X8,
            QdrantPQCompression::X16,
            QdrantPQCompression::X32,
            QdrantPQCompression::X64,
        ];

        for compression in compressions {
            let config = QdrantQuantizationConfig::Product(QdrantProductQuantizationConfig {
                product: QdrantProductQuantization {
                    compression: Some(compression.clone()),
                    always_ram: None,
                },
            });

            // Serialize
            let json = serde_json::to_string(&config).unwrap();

            // Deserialize back
            let deserialized: QdrantQuantizationConfig = serde_json::from_str(&json).unwrap();

            match deserialized {
                QdrantQuantizationConfig::Product(p) => {
                    assert!(p.product.compression.is_some());
                }
                _ => panic!("Expected Product quantization"),
            }
        }
    }
}

// ============================================================================
// Unit Tests for Cluster API Models
// ============================================================================

#[cfg(test)]
mod cluster_api_tests {
    use serde_json::json;
    use std::collections::HashMap;
    use vectorizer::models::qdrant::cluster::{
        QdrantClusterRecoverResponse, QdrantClusterStatus, QdrantClusterStatusResponse,
        QdrantGetMetadataKeyResponse, QdrantListMetadataKeysResponse, QdrantPeerInfo,
        QdrantPeerState, QdrantRaftInfo, QdrantRemovePeerResponse, QdrantUpdateMetadataKeyRequest,
        QdrantUpdateMetadataKeyResponse,
    };

    #[test]
    fn test_cluster_status_response_structure() {
        let mut peers = HashMap::new();
        peers.insert(
            "1".to_string(),
            QdrantPeerInfo {
                uri: "http://node1:6333".to_string(),
                state: Some(QdrantPeerState::Active),
            },
        );
        peers.insert(
            "2".to_string(),
            QdrantPeerInfo {
                uri: "http://node2:6333".to_string(),
                state: Some(QdrantPeerState::Dead),
            },
        );

        let status = QdrantClusterStatus {
            status: "enabled".to_string(),
            peer_id: 1,
            peers,
            raft_info: Some(QdrantRaftInfo {
                term: 5,
                commit: 100,
                pending_operations: 0,
                leader: Some(1),
                role: Some("Leader".to_string()),
                is_voter: true,
            }),
            consensus_thread_status: None,
            message_send_failures: None,
        };

        let response = QdrantClusterStatusResponse {
            result: status,
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("peer_id"));
        assert!(json.contains("node1"));
        assert!(json.contains("node2"));
        assert!(json.contains("Leader"));
    }

    #[test]
    fn test_cluster_recover_response() {
        let response = QdrantClusterRecoverResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.05,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_remove_peer_response() {
        let response = QdrantRemovePeerResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.01,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
    }

    #[test]
    fn test_list_metadata_keys_response() {
        let response = QdrantListMetadataKeysResponse {
            result: vec!["key1".to_string(), "key2".to_string(), "config".to_string()],
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("key1"));
        assert!(json.contains("key2"));
        assert!(json.contains("config"));
    }

    #[test]
    fn test_get_metadata_key_response() {
        let response = QdrantGetMetadataKeyResponse {
            result: json!({"setting": "value", "count": 42}),
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("setting"));
        assert!(json.contains("value"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_update_metadata_key_request() {
        let request = QdrantUpdateMetadataKeyRequest {
            value: json!({"new_setting": true, "limit": 100}),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("new_setting"));
        assert!(json.contains("true"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_update_metadata_key_response() {
        let response = QdrantUpdateMetadataKeyResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.002,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
    }

    #[test]
    fn test_all_peer_states() {
        let states = vec![
            (QdrantPeerState::Active, "Active"),
            (QdrantPeerState::Dead, "Dead"),
            (QdrantPeerState::Restarting, "Restarting"),
        ];

        for (state, expected) in states {
            let json = serde_json::to_string(&state).unwrap();
            assert!(json.contains(expected), "Expected {expected} in {json}");
        }
    }
}

// ============================================================================
// Unit Tests for Snapshot API Models
// ============================================================================

#[cfg(test)]
mod snapshot_api_tests {
    use vectorizer::models::qdrant::snapshot::{
        QdrantCreateSnapshotResponse, QdrantDeleteSnapshotResponse, QdrantListSnapshotsResponse,
        QdrantRecoverSnapshotRequest, QdrantRecoverSnapshotResponse, QdrantSnapshotDescription,
    };

    #[test]
    fn test_create_snapshot_response() {
        let response = QdrantCreateSnapshotResponse {
            result: QdrantSnapshotDescription {
                name: "snapshot_2024_01_15_120000".to_string(),
                creation_time: Some("2024-01-15T12:00:00Z".to_string()),
                size: 1024 * 1024 * 100, // 100MB
                checksum: Some("sha256:abc123def456".to_string()),
            },
            status: "ok".to_string(),
            time: 5.25,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("snapshot_2024_01_15_120000"));
        assert!(json.contains("2024-01-15T12:00:00Z"));
        assert!(json.contains("sha256:abc123def456"));
    }

    #[test]
    fn test_delete_snapshot_response() {
        let response = QdrantDeleteSnapshotResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
    }

    #[test]
    fn test_recover_snapshot_request_s3() {
        let request = QdrantRecoverSnapshotRequest {
            location: "s3://my-bucket/backups/collection_snapshot.tar".to_string(),
            priority: Some("snapshot".to_string()),
            checksum: Some("sha256:xyz789".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("s3://my-bucket/backups/collection_snapshot.tar"));
        assert!(json.contains("snapshot"));
        assert!(json.contains("sha256:xyz789"));
    }

    #[test]
    fn test_recover_snapshot_request_local() {
        let request = QdrantRecoverSnapshotRequest {
            location: "file:///snapshots/backup.tar".to_string(),
            priority: None,
            checksum: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("file:///snapshots/backup.tar"));
    }

    #[test]
    fn test_recover_snapshot_response() {
        let response = QdrantRecoverSnapshotResponse {
            result: true,
            status: "ok".to_string(),
            time: 30.5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
        assert!(json.contains("30.5"));
    }

    #[test]
    fn test_list_snapshots_empty() {
        let response = QdrantListSnapshotsResponse {
            result: vec![],
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":[]"));
    }

    #[test]
    fn test_list_snapshots_multiple() {
        let response = QdrantListSnapshotsResponse {
            result: vec![
                QdrantSnapshotDescription {
                    name: "snap1".to_string(),
                    creation_time: Some("2024-01-01T00:00:00Z".to_string()),
                    size: 1000,
                    checksum: None,
                },
                QdrantSnapshotDescription {
                    name: "snap2".to_string(),
                    creation_time: Some("2024-01-02T00:00:00Z".to_string()),
                    size: 2000,
                    checksum: Some("checksum2".to_string()),
                },
            ],
            status: "ok".to_string(),
            time: 0.01,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("snap1"));
        assert!(json.contains("snap2"));
        assert!(json.contains("checksum2"));
    }
}

// ============================================================================
// Unit Tests for Sharding API Models
// ============================================================================

#[cfg(test)]
mod sharding_api_tests {
    use vectorizer::models::qdrant::sharding::{
        QdrantCreateShardKeyRequest, QdrantCreateShardKeyResponse, QdrantDeleteShardKeyResponse,
        QdrantListShardKeysResponse, QdrantShardKeyInfo, QdrantShardKeyValue,
        QdrantShardKeysResult,
    };

    #[test]
    fn test_create_shard_key_response() {
        let response = QdrantCreateShardKeyResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
    }

    #[test]
    fn test_delete_shard_key_response() {
        let response = QdrantDeleteShardKeyResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.3,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
    }

    #[test]
    fn test_create_shard_key_with_placement() {
        let request = QdrantCreateShardKeyRequest {
            shard_key: QdrantShardKeyValue::String("tenant_premium".to_string()),
            shards_number: Some(8),
            replication_factor: Some(3),
            placement: Some(vec![1, 2, 3, 4]),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("tenant_premium"));
        assert!(json.contains("\"shards_number\":8"));
        assert!(json.contains("\"replication_factor\":3"));
        assert!(json.contains("[1,2,3,4]"));
    }

    #[test]
    fn test_create_shard_key_minimal() {
        let request = QdrantCreateShardKeyRequest {
            shard_key: QdrantShardKeyValue::Integer(100),
            shards_number: None,
            replication_factor: None,
            placement: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("100"));
    }

    #[test]
    fn test_list_shard_keys_response() {
        let response = QdrantListShardKeysResponse {
            result: QdrantShardKeysResult {
                keys: vec![
                    QdrantShardKeyInfo {
                        shard_key: QdrantShardKeyValue::String("tenant_a".to_string()),
                        shards_number: 4,
                        replication_factor: 1,
                        local_shards: vec![],
                        remote_shards: vec![],
                    },
                    QdrantShardKeyInfo {
                        shard_key: QdrantShardKeyValue::Integer(42),
                        shards_number: 2,
                        replication_factor: 1,
                        local_shards: vec![],
                        remote_shards: vec![],
                    },
                ],
            },
            status: "ok".to_string(),
            time: 0.01,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("tenant_a"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_list_shard_keys_empty() {
        let response = QdrantListShardKeysResponse {
            result: QdrantShardKeysResult { keys: vec![] },
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("keys"));
    }
}
