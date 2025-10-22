//! Tests for replication module

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::db::VectorStore;
    use crate::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};
    use std::sync::Arc;

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

        // Can get operations from offset 5
        let ops = log.get_operations(5).unwrap();
        assert_eq!(ops.len(), 5);
        assert_eq!(ops[0].offset, 6);
    }

    #[tokio::test]
    async fn test_snapshot_creation_and_application() {
        let store1 = VectorStore::new();

        // Create collection
        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        store1.create_collection("test", config).unwrap();

        // Insert vectors
        store1
            .insert_vector("test", "vec1", vec![1.0, 0.0, 0.0], None)
            .unwrap();
        store1
            .insert_vector("test", "vec2", vec![0.0, 1.0, 0.0], None)
            .unwrap();

        // Create snapshot
        let snapshot = sync::create_snapshot(&store1, 100).await.unwrap();
        assert!(!snapshot.is_empty());

        // Apply to new store
        let store2 = VectorStore::new();
        let offset = sync::apply_snapshot(&store2, &snapshot).await.unwrap();

        assert_eq!(offset, 100);

        // Verify data
        assert_eq!(store2.list_collections().len(), 1);
        let info = store2.get_collection_info("test").unwrap().unwrap();
        assert_eq!(info.vector_count, 2);
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
}

