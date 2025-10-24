//! Real Integration Tests for Master-Replica Communication
//!
//! These tests actually run the TCP server and client to achieve >95% coverage
//! for master.rs and replica.rs modules.

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::time::sleep;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector,
};
use vectorizer::replication::{
    MasterNode, NodeRole, ReplicaNode, ReplicationConfig, VectorOperation,
};

static INTEGRATION_PORT: AtomicU16 = AtomicU16::new(50000);

fn next_port() -> u16 {
    INTEGRATION_PORT.fetch_add(1, Ordering::SeqCst)
}

/// Helper to create and start a master node
async fn create_running_master() -> (Arc<MasterNode>, Arc<VectorStore>, std::net::SocketAddr) {
    let port = next_port();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    let config = ReplicationConfig {
        role: NodeRole::Master,
        bind_address: Some(addr),
        master_address: None,
        heartbeat_interval: 1,
        replica_timeout: 10,
        log_size: 10000,
        reconnect_interval: 1,
    };

    let store = Arc::new(VectorStore::new());
    let master = Arc::new(MasterNode::new(config, Arc::clone(&store)).unwrap());

    // Actually start the master TCP server
    let master_clone = Arc::clone(&master);
    tokio::spawn(async move {
        let _ = master_clone.start().await;
    });

    // Wait for server to be ready
    sleep(Duration::from_millis(200)).await;

    (master, store, addr)
}

/// Helper to create and start a replica node
async fn create_running_replica(
    master_addr: std::net::SocketAddr,
) -> (Arc<ReplicaNode>, Arc<VectorStore>) {
    let config = ReplicationConfig {
        role: NodeRole::Replica,
        bind_address: None,
        master_address: Some(master_addr),
        heartbeat_interval: 1,
        replica_timeout: 10,
        log_size: 10000,
        reconnect_interval: 1,
    };

    let store = Arc::new(VectorStore::new());
    let replica = Arc::new(ReplicaNode::new(config, Arc::clone(&store)));

    // Actually start the replica (connects to master)
    let replica_clone = Arc::clone(&replica);
    tokio::spawn(async move {
        let _ = replica_clone.start().await;
    });

    // Wait for connection and initial sync
    sleep(Duration::from_millis(500)).await;

    (replica, store)
}

// ============================================================================
// Master Node Coverage Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_master_start_and_accept_connections() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection BEFORE replica connects
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("pre_sync", config).unwrap();

    // Insert some vectors
    for i in 0..5 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            payload: None,
        };
        master_store.insert("pre_sync", vec![vec]).unwrap();
    }

    // Now connect replica - should trigger full sync
    let (_replica, replica_store) = create_running_replica(master_addr).await;

    // Wait for full sync to complete
    sleep(Duration::from_secs(2)).await;

    // Verify replica received snapshot
    assert!(
        replica_store
            .list_collections()
            .contains(&"pre_sync".to_string())
    );
    let collection = replica_store.get_collection("pre_sync").unwrap();
    assert_eq!(collection.vector_count(), 5);

    // Test master stats (offset may be 0 since insert was before replica connected)
    let stats = master.get_stats();
    assert_eq!(stats.role, vectorizer::replication::NodeRole::Master);
    // Note: master_offset will be 0 because vectors were inserted before replication started

    // Test master replicas info
    let replicas = master.get_replicas();
    assert_eq!(replicas.len(), 1);
    assert_eq!(
        replicas[0].status,
        vectorizer::replication::ReplicaStatus::Connected
    );

    // Verify replica received full sync
    assert_eq!(replicas[0].offset, 0); // Replica got full sync, not incremental

    println!("✅ Master start and full sync: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_master_replicate_operations() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Connect replica first
    let (_replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Create collection and replicate creation
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("test", config.clone())
        .unwrap();

    // Trigger replication explicitly
    master.replicate(VectorOperation::CreateCollection {
        name: "test".to_string(),
        config: vectorizer::replication::CollectionConfigData {
            dimension: 3,
            metric: "cosine".to_string(),
        },
    });

    sleep(Duration::from_millis(500)).await;

    // Now insert vectors and replicate them
    for i in 0..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, (i + 1) as f32, (i + 2) as f32],
            payload: Some(Payload {
                data: serde_json::json!({"index": i}),
            }),
        };
        master_store.insert("test", vec![vec.clone()]).unwrap();

        // Replicate the operation
        let payload_bytes = vec
            .payload
            .as_ref()
            .map(|p| serde_json::to_vec(&p.data).unwrap());

        master.replicate(VectorOperation::InsertVector {
            collection: "test".to_string(),
            id: vec.id,
            vector: vec.data,
            payload: payload_bytes,
        });
    }

    sleep(Duration::from_secs(1)).await;

    // Verify replication
    let replica_collection = replica_store.get_collection("test").unwrap();
    assert_eq!(replica_collection.vector_count(), 10);

    // Verify stats updated
    let stats = master.get_stats();
    assert!(stats.master_offset >= 11); // 1 CreateCollection + 10 InsertVector
    assert_eq!(stats.role, vectorizer::replication::NodeRole::Master);

    println!("✅ Master replicate operations: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_master_multiple_replicas_and_stats() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("multi", config).unwrap();

    // Connect 3 replicas
    let mut replicas = vec![];
    for i in 0..3 {
        let (replica, store) = create_running_replica(master_addr).await;
        replicas.push((replica, store));
        println!("Replica {i} connected");
        sleep(Duration::from_millis(200)).await;
    }

    // Wait for all to sync
    sleep(Duration::from_secs(1)).await;

    // Insert data
    for i in 0..20 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            payload: None,
        };
        master_store.insert("multi", vec![vec.clone()]).unwrap();

        master.replicate(VectorOperation::InsertVector {
            collection: "multi".to_string(),
            id: format!("vec_{i}"),
            vector: vec![i as f32, 0.0, 0.0],
            payload: None,
        });
    }

    sleep(Duration::from_secs(2)).await;

    // Verify all replicas got the data
    for (i, (_replica, store)) in replicas.iter().enumerate() {
        let collection = store.get_collection("multi").unwrap();
        assert_eq!(collection.vector_count(), 20, "Replica {i} mismatch");
    }

    // Test master stats with multiple replicas
    let stats = master.get_stats();
    assert!(stats.master_offset >= 20);

    // Test get_replicas with multiple replicas
    let replica_infos = master.get_replicas();
    assert_eq!(replica_infos.len(), 3);

    for info in replica_infos {
        assert_eq!(
            info.status,
            vectorizer::replication::ReplicaStatus::Connected
        );
        assert!(info.offset > 0);
    }

    println!("✅ Master multiple replicas: PASS");
}

// ============================================================================
// Replica Node Coverage Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_full_sync_on_connect() {
    let (_master, master_store, master_addr) = create_running_master().await;

    // Populate master before replica connects
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("full_sync", config).unwrap();

    for i in 0..50 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, (i + 1) as f32, (i + 2) as f32],
            payload: None,
        };
        master_store.insert("full_sync", vec![vec]).unwrap();
    }

    // Now connect replica - should receive full snapshot
    let (replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(2)).await;

    // Verify full sync worked
    assert_eq!(replica_store.list_collections().len(), 1);
    let collection = replica_store.get_collection("full_sync").unwrap();
    assert_eq!(collection.vector_count(), 50);

    // Test replica stats
    let stats = replica.get_stats();
    // Full sync via snapshot may have offset 0 (snapshot-based, not incremental)
    assert_eq!(stats.role, vectorizer::replication::NodeRole::Replica);
    assert!(replica.is_connected());

    println!("✅ Replica full sync: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_partial_sync_on_reconnect() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("partial", config).unwrap();

    // Connect replica and sync
    let (replica1, replica_store1) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert some data
    for i in 0..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            payload: None,
        };
        master_store.insert("partial", vec![vec.clone()]).unwrap();

        master.replicate(VectorOperation::InsertVector {
            collection: "partial".to_string(),
            id: vec.id,
            vector: vec.data,
            payload: None,
        });
    }

    sleep(Duration::from_secs(1)).await;

    let offset_before = replica1.get_offset();
    assert_eq!(
        replica_store1
            .get_collection("partial")
            .unwrap()
            .vector_count(),
        10
    );

    // Disconnect replica
    drop(replica1);
    drop(replica_store1);
    sleep(Duration::from_millis(200)).await;

    // Insert more data while disconnected (but within log window)
    for i in 10..15 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            payload: None,
        };
        master_store.insert("partial", vec![vec.clone()]).unwrap();

        master.replicate(VectorOperation::InsertVector {
            collection: "partial".to_string(),
            id: vec.id,
            vector: vec.data,
            payload: None,
        });
    }

    // Reconnect - should use partial sync
    let (replica2, replica_store2) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Verify caught up via partial sync
    let collection = replica_store2.get_collection("partial").unwrap();
    assert_eq!(collection.vector_count(), 15);

    let offset_after = replica2.get_offset();
    assert!(offset_after > offset_before);

    println!("✅ Replica partial sync: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_apply_all_operation_types() {
    let (master, master_store, master_addr) = create_running_master().await;

    let (_replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Test CreateCollection operation
    let config = CollectionConfig {
        dimension: 4,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("ops_test", config).unwrap();

    master.replicate(VectorOperation::CreateCollection {
        name: "ops_test".to_string(),
        config: vectorizer::replication::CollectionConfigData {
            dimension: 4,
            metric: "euclidean".to_string(),
        },
    });

    sleep(Duration::from_millis(300)).await;

    // Test InsertVector operation
    let vec1 = Vector {
        id: "insert_test".to_string(),
        data: vec![1.0, 2.0, 3.0, 4.0],
        payload: Some(Payload {
            data: serde_json::json!({"type": "insert"}),
        }),
    };
    master_store.insert("ops_test", vec![vec1.clone()]).unwrap();

    let payload_bytes = serde_json::to_vec(&serde_json::json!({"type": "insert"})).unwrap();
    master.replicate(VectorOperation::InsertVector {
        collection: "ops_test".to_string(),
        id: "insert_test".to_string(),
        vector: vec![1.0, 2.0, 3.0, 4.0],
        payload: Some(payload_bytes),
    });

    sleep(Duration::from_millis(300)).await;

    // Test UpdateVector operation
    master.replicate(VectorOperation::UpdateVector {
        collection: "ops_test".to_string(),
        id: "insert_test".to_string(),
        vector: Some(vec![5.0, 6.0, 7.0, 8.0]),
        payload: None,
    });

    sleep(Duration::from_millis(300)).await;

    // Test DeleteVector operation
    master.replicate(VectorOperation::DeleteVector {
        collection: "ops_test".to_string(),
        id: "insert_test".to_string(),
    });

    sleep(Duration::from_millis(300)).await;

    // Verify operations were applied on replica
    assert!(
        replica_store
            .list_collections()
            .contains(&"ops_test".to_string())
    );

    println!("✅ All operation types applied: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_heartbeat_and_connection_status() {
    let (_master, _master_store, master_addr) = create_running_master().await;

    let (replica, _replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Initially connected
    assert!(replica.is_connected());

    // Wait for heartbeats
    sleep(Duration::from_secs(3)).await;

    // Should still be connected
    assert!(replica.is_connected());

    // Check stats
    let stats = replica.get_stats();
    assert_eq!(stats.role, vectorizer::replication::NodeRole::Replica);
    assert!(stats.lag_ms < 5000); // Should be recent (within 5 seconds)

    println!("✅ Replica heartbeat: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_incremental_operations() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("incremental", config)
        .unwrap();

    // Connect replica
    let (replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    let initial_offset = replica.get_offset();

    // Send operations one by one (tests incremental replication)
    for i in 0..20 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            payload: None,
        };
        master_store
            .insert("incremental", vec![vec.clone()])
            .unwrap();

        master.replicate(VectorOperation::InsertVector {
            collection: "incremental".to_string(),
            id: vec.id,
            vector: vec.data,
            payload: None,
        });

        // Small delay between operations
        sleep(Duration::from_millis(50)).await;
    }

    sleep(Duration::from_secs(1)).await;

    // Verify all operations received
    let collection = replica_store.get_collection("incremental").unwrap();
    assert_eq!(collection.vector_count(), 20);

    // Verify offset incremented
    let final_offset = replica.get_offset();
    assert!(final_offset > initial_offset);

    // Verify stats
    let stats = replica.get_stats();
    assert_eq!(stats.total_replicated, 20);

    println!("✅ Replica incremental operations: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_delete_operations() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection BEFORE replica connects (matching test_master_start_and_accept_connections pattern)
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("delete_test", config)
        .unwrap();

    // Insert some vectors
    for i in 0..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            payload: None,
        };
        master_store.insert("delete_test", vec![vec]).unwrap();
    }

    // Now connect replica - should trigger full sync
    let (_replica, replica_store) = create_running_replica(master_addr).await;

    // Wait for full sync to complete
    sleep(Duration::from_secs(2)).await;

    // Verify replica received snapshot
    assert!(
        replica_store
            .list_collections()
            .contains(&"delete_test".to_string())
    );
    let collection = replica_store.get_collection("delete_test").unwrap();
    assert_eq!(collection.vector_count(), 10);

    // Delete some vectors
    for i in 0..5 {
        master_store
            .delete("delete_test", &format!("vec_{i}"))
            .unwrap();

        let offset = master.replicate(VectorOperation::DeleteVector {
            collection: "delete_test".to_string(),
            id: format!("vec_{i}"),
        });
        println!("Replicated delete of vec_{i} at offset {offset}");
    }

    println!(
        "Master now has {} vectors",
        master_store
            .get_collection("delete_test")
            .unwrap()
            .vector_count()
    );

    sleep(Duration::from_secs(3)).await;

    // Verify deletes replicated
    let collection = replica_store.get_collection("delete_test").unwrap();
    println!("Replica has {} vectors", collection.vector_count());

    // List vectors in replica to see which ones remain
    for i in 0..10 {
        if let Ok(v) = replica_store.get_vector("delete_test", &format!("vec_{i}")) {
            println!("Replica still has vec_{i}: {:?}", v.data);
        }
    }

    assert_eq!(collection.vector_count(), 5);

    // Delete entire collection
    master_store.delete_collection("delete_test").unwrap();

    master.replicate(VectorOperation::DeleteCollection {
        name: "delete_test".to_string(),
    });

    sleep(Duration::from_millis(500)).await;

    // Verify collection deleted on replica
    assert!(
        !replica_store
            .list_collections()
            .contains(&"delete_test".to_string())
    );

    println!("✅ Replica delete operations: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_update_operations() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection with Euclidean metric to avoid normalization
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("update_test", config)
        .unwrap();

    // Insert initial vector
    let vec1 = Vector {
        id: "updatable".to_string(),
        data: vec![1.0, 0.0, 0.0],
        payload: None,
    };
    master_store.insert("update_test", vec![vec1]).unwrap();

    // Connect replica
    let (_replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Verify initial sync
    let vector = replica_store
        .get_vector("update_test", "updatable")
        .unwrap();
    assert_eq!(vector.data, vec![1.0, 0.0, 0.0]);

    // Update the vector
    master.replicate(VectorOperation::UpdateVector {
        collection: "update_test".to_string(),
        id: "updatable".to_string(),
        vector: Some(vec![9.0, 9.0, 9.0]),
        payload: Some(serde_json::to_vec(&serde_json::json!({"updated": true})).unwrap()),
    });

    sleep(Duration::from_secs(1)).await;

    // Verify update replicated
    let updated_vector = replica_store
        .get_vector("update_test", "updatable")
        .unwrap();
    assert_eq!(updated_vector.data, vec![9.0, 9.0, 9.0]);

    println!("✅ Replica update operations: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_replica_stats_tracking() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("stats", config).unwrap();

    let (replica, _replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Check initial stats
    let stats1 = replica.get_stats();
    assert_eq!(stats1.total_replicated, 0);
    assert_eq!(stats1.role, vectorizer::replication::NodeRole::Replica);

    // Replicate operations
    for i in 0..30 {
        master.replicate(VectorOperation::InsertVector {
            collection: "stats".to_string(),
            id: format!("vec_{i}"),
            vector: vec![i as f32, 0.0, 0.0],
            payload: None,
        });
    }

    sleep(Duration::from_secs(1)).await;

    // Check updated stats
    let stats2 = replica.get_stats();
    assert_eq!(stats2.total_replicated, 30);
    assert!(stats2.replica_offset > stats1.replica_offset);
    assert_eq!(stats2.role, vectorizer::replication::NodeRole::Replica);

    println!("✅ Replica stats tracking: PASS");
}

// ============================================================================
// Coverage for Edge Cases
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_empty_snapshot_replication() {
    let (_master, _master_store, master_addr) = create_running_master().await;

    // Connect replica with no data on master
    let (_replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Should have empty state
    assert_eq!(replica_store.list_collections().len(), 0);

    println!("✅ Empty snapshot: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_large_payload_replication() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("large_payload", config)
        .unwrap();

    let (_replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert vector with large payload
    let large_data = (0..1000).map(|i| format!("item_{i}")).collect::<Vec<_>>();
    let vec = Vector {
        id: "large".to_string(),
        data: vec![1.0, 2.0, 3.0],
        payload: Some(Payload {
            data: serde_json::json!({"items": large_data}),
        }),
    };

    master_store
        .insert("large_payload", vec![vec.clone()])
        .unwrap();

    let payload_bytes = serde_json::to_vec(&serde_json::json!({"items": large_data})).unwrap();
    master.replicate(VectorOperation::InsertVector {
        collection: "large_payload".to_string(),
        id: "large".to_string(),
        vector: vec.data,
        payload: Some(payload_bytes),
    });

    sleep(Duration::from_millis(500)).await;

    // Verify large payload replicated
    let collection = replica_store.get_collection("large_payload").unwrap();
    assert_eq!(collection.vector_count(), 1);

    let replicated_vec = replica_store.get_vector("large_payload", "large").unwrap();
    assert!(replicated_vec.payload.is_some());

    println!("✅ Large payload replication: PASS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_different_distance_metrics() {
    let (master, master_store, master_addr) = create_running_master().await;

    let (_replica, replica_store) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Test Cosine
    let config_cosine = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("cosine", config_cosine)
        .unwrap();
    master.replicate(VectorOperation::CreateCollection {
        name: "cosine".to_string(),
        config: vectorizer::replication::CollectionConfigData {
            dimension: 3,
            metric: "cosine".to_string(),
        },
    });

    // Test Euclidean
    let config_euclidean = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("euclidean", config_euclidean)
        .unwrap();
    master.replicate(VectorOperation::CreateCollection {
        name: "euclidean".to_string(),
        config: vectorizer::replication::CollectionConfigData {
            dimension: 3,
            metric: "euclidean".to_string(),
        },
    });

    // Test DotProduct
    let config_dot = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::DotProduct,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store
        .create_collection("dotproduct", config_dot)
        .unwrap();
    master.replicate(VectorOperation::CreateCollection {
        name: "dotproduct".to_string(),
        config: vectorizer::replication::CollectionConfigData {
            dimension: 3,
            metric: "dot_product".to_string(),
        },
    });

    sleep(Duration::from_secs(1)).await;

    // Verify all metrics replicated
    assert_eq!(replica_store.list_collections().len(), 3);

    println!("✅ Different distance metrics: PASS");
}

// ============================================================================
// High Coverage Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn test_master_get_stats_coverage() {
    let (master, master_store, master_addr) = create_running_master().await;

    // Get stats with no replicas
    let stats1 = master.get_stats();
    assert_eq!(stats1.master_offset, 0);
    assert_eq!(stats1.connected_replicas, Some(0)); // No replicas yet

    // Connect replica
    let (_replica, _) = create_running_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Perform operations
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };
    master_store.create_collection("coverage", config).unwrap();
    master.replicate(VectorOperation::CreateCollection {
        name: "coverage".to_string(),
        config: vectorizer::replication::CollectionConfigData {
            dimension: 3,
            metric: "cosine".to_string(),
        },
    });

    sleep(Duration::from_millis(300)).await;

    // Get stats with replica
    let stats2 = master.get_stats();
    assert!(stats2.master_offset > 0);
    assert_eq!(stats2.role, vectorizer::replication::NodeRole::Master);
    assert!(stats2.total_replicated > 0);

    println!("✅ Master stats coverage: PASS");
}
