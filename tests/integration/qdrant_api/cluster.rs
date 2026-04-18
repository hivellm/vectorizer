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
mod cluster_api_tests {
    use std::collections::HashMap;

    use serde_json::json;
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
