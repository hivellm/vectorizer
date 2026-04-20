#![allow(clippy::unwrap_used, clippy::expect_used)]

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
