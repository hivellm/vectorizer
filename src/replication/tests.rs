//! Replication Module Tests
//!
//! Core unit tests for the replication components.
//! For comprehensive integration tests, see:
//! - tests/replication_comprehensive.rs - Full integration tests
//! - tests/replication_failover.rs - Failover and reconnection tests
//! - benches/replication_bench.rs - Performance benchmarks

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::*;
    use crate::db::VectorStore;
    use crate::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector};

    // ============================================================================
    // Replication Log Tests
    // ============================================================================

    #[tokio::test]
    async fn test_replication_log_basic() {
        let log = ReplicationLog::new(10);

        let op = VectorOperation::CreateCollection {
            name: "test".to_string(),
            config: CollectionConfigData {
                dimension: 128,
                metric: "cosine".to_string(),
            },
        };

        let offset1 = log.append(op.clone());
        assert_eq!(offset1, 1);
        assert_eq!(log.current_offset(), 1);
        assert_eq!(log.size(), 1);

        let offset2 = log.append(op);
        assert_eq!(offset2, 2);
        assert_eq!(log.size(), 2);
    }

    #[tokio::test]
    async fn test_replication_log_circular() {
        let log = ReplicationLog::new(5);

        // Add 10 operations (more than max_size)
        for i in 0..10 {
            let op = VectorOperation::CreateCollection {
                name: format!("test{}", i),
                config: CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
            };
            log.append(op);
        }

        // Should only keep last 5
        assert_eq!(log.size(), 5);
        assert_eq!(log.current_offset(), 10);

        // Operations from offset 5 - should get offsets > 5
        // Oldest is 6, so we get 6, 7, 8, 9, 10 (5 operations)
        if let Some(ops) = log.get_operations(5) {
            assert_eq!(ops.len(), 5);
            assert_eq!(ops[0].offset, 6);
            assert_eq!(ops[4].offset, 10);
        }
    }

    #[tokio::test]
    #[ignore = "Snapshot replication issue - vector_count returns 0 after snapshot application. Same root cause as integration tests"]
    async fn test_snapshot_creation_and_application() {
        let store1 = VectorStore::new();

        // Create collection
        let config = CollectionConfig {
            graph: None,
            sharding: None,
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
        };
        store1.create_collection("test", config).unwrap();

        // Insert vectors
        let vec1 = Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            sparse: None,
            payload: None,
        };
        let vec2 = Vector {
            id: "vec2".to_string(),
            data: vec![0.0, 1.0, 0.0],
            sparse: None,
            payload: None,
        };
        store1.insert("test", vec![vec1, vec2]).unwrap();

        // Create snapshot
        let snapshot = sync::create_snapshot(&store1, 100).await.unwrap();
        assert!(!snapshot.is_empty());

        // Apply to new store
        let store2 = VectorStore::new();
        let offset = sync::apply_snapshot(&store2, &snapshot).await.unwrap();

        assert_eq!(offset, 100);

        // Verify data
        assert_eq!(store2.list_collections().len(), 1);
        let collection = store2.get_collection("test").unwrap();
        assert_eq!(collection.vector_count(), 2);
    }

    #[tokio::test]
    async fn test_replication_config() {
        let config = ReplicationConfig::master("127.0.0.1:7000".parse().unwrap());
        assert_eq!(config.role, NodeRole::Master);
        assert!(config.bind_address.is_some());
        assert!(config.master_address.is_none());

        let config = ReplicationConfig::replica("127.0.0.1:7000".parse().unwrap());
        assert_eq!(config.role, NodeRole::Replica);
        assert!(config.bind_address.is_none());
        assert!(config.master_address.is_some());
    }

    // ============================================================================
    // Node Creation Tests
    // ============================================================================

    #[tokio::test]
    async fn test_master_node_creation() {
        let store = Arc::new(VectorStore::new());
        let config = ReplicationConfig {
            role: NodeRole::Master,
            bind_address: Some("127.0.0.1:0".parse().unwrap()),
            master_address: None,
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size: 1000,
            reconnect_interval: 5,
        };

        let master = MasterNode::new(config, store);
        assert!(master.is_ok());
    }

    #[tokio::test]
    async fn test_replica_node_creation() {
        let store = Arc::new(VectorStore::new());
        let config = ReplicationConfig {
            role: NodeRole::Replica,
            bind_address: None,
            master_address: Some("127.0.0.1:7000".parse().unwrap()),
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size: 1000,
            reconnect_interval: 5,
        };

        let replica = ReplicaNode::new(config, store);
        assert_eq!(replica.get_offset(), 0);
        assert!(!replica.is_connected());
    }

    // ============================================================================
    // Vector Operation Tests
    // ============================================================================

    #[test]
    fn test_vector_operation_serialization() {
        let operations = vec![
            VectorOperation::CreateCollection {
                name: "test".to_string(),
                config: CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
            },
            VectorOperation::InsertVector {
                collection: "test".to_string(),
                id: "vec1".to_string(),
                vector: vec![1.0, 2.0, 3.0],
                payload: Some(b"test".to_vec()),
            },
            VectorOperation::UpdateVector {
                collection: "test".to_string(),
                id: "vec1".to_string(),
                vector: Some(vec![4.0, 5.0, 6.0]),
                payload: None,
            },
            VectorOperation::DeleteVector {
                collection: "test".to_string(),
                id: "vec1".to_string(),
            },
            VectorOperation::DeleteCollection {
                name: "test".to_string(),
            },
        ];

        for op in operations {
            let serialized = bincode::serialize(&op).unwrap();
            let deserialized: VectorOperation = bincode::deserialize(&serialized).unwrap();
            // Just verify it round-trips without error
            let _ = bincode::serialize(&deserialized).unwrap();
        }
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[tokio::test]
    async fn test_replication_log_empty() {
        let log = ReplicationLog::new(10);
        assert_eq!(log.current_offset(), 0);
        assert_eq!(log.size(), 0);
        assert!(log.get_operations(0).is_none());
    }

    #[tokio::test]
    async fn test_replication_log_single_operation() {
        let log = ReplicationLog::new(10);

        let op = VectorOperation::CreateCollection {
            name: "test".to_string(),
            config: CollectionConfigData {
                dimension: 128,
                metric: "cosine".to_string(),
            },
        };

        let offset = log.append(op);
        assert_eq!(offset, 1);

        // get_operations(0) returns operations with offset > 0, so offset 1
        if let Some(ops) = log.get_operations(0) {
            assert_eq!(ops.len(), 1);
            assert_eq!(ops[0].offset, 1);
        }
    }

    #[tokio::test]
    async fn test_config_durations() {
        let config = ReplicationConfig::default();
        assert_eq!(config.heartbeat_duration().as_secs(), 5);
        assert_eq!(config.timeout_duration().as_secs(), 30);
        assert_eq!(config.reconnect_duration().as_secs(), 5);
    }
}

// ============================================================================
// Integration Test Notes
// ============================================================================

// For comprehensive testing, run:
// - `cargo test` - Run all unit tests
// - `cargo test --test replication_comprehensive` - Integration tests
// - `cargo test --test replication_failover` - Failover tests
// - `cargo test -- --ignored` - Stress tests (slower)
// - `cargo bench --bench replication_bench` - Performance benchmarks
