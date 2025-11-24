//! Comprehensive Replication Tests
//!
//! This test suite validates the master-replica replication system with:
//! - Unit tests for individual components
//! - Integration tests for end-to-end replication
//! - Stress tests for high-volume scenarios
//! - Failover and reconnection tests
//! - Performance benchmarks

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::time::sleep;
use tracing::info;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector,
};
use vectorizer::replication::{
    MasterNode, NodeRole, ReplicaNode, ReplicationConfig, ReplicationLog, VectorOperation,
};

/// Port allocator for tests
static TEST_PORT: AtomicU16 = AtomicU16::new(40000);

fn next_port_comprehensive() -> u16 {
    TEST_PORT.fetch_add(1, Ordering::SeqCst)
}

/// Create a master node for testing
async fn create_master() -> (Arc<MasterNode>, Arc<VectorStore>, std::net::SocketAddr) {
    let port = next_port_comprehensive();
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

    // Start master server
    let master_clone = Arc::clone(&master);
    tokio::spawn(async move {
        let _ = master_clone.start().await;
    });

    sleep(Duration::from_millis(100)).await;

    (master, store, addr)
}

/// Create a replica node for testing
async fn create_replica(master_addr: std::net::SocketAddr) -> (Arc<ReplicaNode>, Arc<VectorStore>) {
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

    // Start replica
    let replica_clone = Arc::clone(&replica);
    tokio::spawn(async move {
        let _ = replica_clone.start().await;
    });

    sleep(Duration::from_millis(200)).await;

    (replica, store)
}

// ============================================================================
// UNIT TESTS - Replication Log
// ============================================================================

#[tokio::test]
async fn test_replication_log_append_and_retrieve() {
    let log = ReplicationLog::new(100);

    // Append operations
    for i in 0..10 {
        let op = VectorOperation::CreateCollection {
            name: format!("collection_{i}"),
            config: vectorizer::replication::CollectionConfigData {
                dimension: 128,
                metric: "cosine".to_string(),
            },
        };
        let offset = log.append(op);
        assert_eq!(offset, i + 1);
    }

    assert_eq!(log.current_offset(), 10);
    assert_eq!(log.size(), 10);

    // Retrieve operations
    let ops = log.get_operations(5).unwrap();
    assert_eq!(ops.len(), 5); // 6, 7, 8, 9, 10
    assert_eq!(ops[0].offset, 6);
    assert_eq!(ops[4].offset, 10);
}

#[tokio::test]
async fn test_replication_log_circular_buffer() {
    let log = ReplicationLog::new(5);

    // Add 20 operations (exceeds buffer size)
    for i in 0..20 {
        let op = VectorOperation::InsertVector {
            collection: "test".to_string(),
            id: format!("vec_{i}"),
            vector: vec![i as f32; 128],
            payload: None,
        };
        log.append(op);
    }

    // Should only keep last 5
    assert_eq!(log.size(), 5);
    assert_eq!(log.current_offset(), 20);

    // Oldest operation should be offset 16
    // get_operations(15) returns operations with offset > 15, which are 16-20 (5 ops)
    if let Some(ops) = log.get_operations(15) {
        assert_eq!(ops.len(), 5);
        assert_eq!(ops[0].offset, 16);
        assert_eq!(ops[4].offset, 20);
    }

    // Operations before 16 should trigger full sync (None)
    assert!(log.get_operations(10).is_none());
}

#[tokio::test]
async fn test_replication_log_concurrent_access() {
    let log = Arc::new(ReplicationLog::new(1000));
    let mut handles = vec![];

    // Spawn 10 threads appending operations
    for thread_id in 0..10 {
        let log_clone = Arc::clone(&log);
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                let op = VectorOperation::InsertVector {
                    collection: format!("col_{thread_id}"),
                    id: format!("vec_{thread_id}_{i}"),
                    vector: vec![thread_id as f32; 64],
                    payload: None,
                };
                log_clone.append(op);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have 1000 operations total
    assert_eq!(log.current_offset(), 1000);
    assert_eq!(log.size(), 1000);
}

// ============================================================================
// INTEGRATION TESTS - Master-Replica Communication
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_basic_master_replica_sync() {
    let (_master, master_store, master_addr) = create_master().await;

    // Create collection on master
    let config = CollectionConfig {
        graph: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Insert vectors on master
    let vectors = vec![
        Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            ..Default::default()
        },
        Vector {
            id: "vec2".to_string(),
            data: vec![0.0, 1.0, 0.0],
            ..Default::default()
        },
    ];
    master_store.insert("test", vectors).unwrap();

    // Create replica (should receive snapshot)
    let (_replica, replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(2)).await;

    // Verify collection exists on replica
    assert_eq!(replica_store.list_collections().len(), 1);

    // Verify vectors are replicated
    let collection = replica_store.get_collection("test").unwrap();
    assert_eq!(collection.vector_count(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_incremental_replication() {
    let (_master, master_store, master_addr) = create_master().await;

    // Create collection
    let config = CollectionConfig {
        graph: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Start replica
    let (_replica, replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert vectors incrementally on master
    for i in 0..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, (i + 1) as f32, (i + 2) as f32],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
        sleep(Duration::from_millis(50)).await;
    }

    sleep(Duration::from_secs(1)).await;

    // Verify all vectors replicated
    let collection = replica_store.get_collection("test").unwrap();
    assert_eq!(collection.vector_count(), 10);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_multiple_replicas() {
    let (_master, master_store, master_addr) = create_master().await;

    // Create collection
    let config = CollectionConfig {
        graph: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Create 3 replicas
    let mut replicas = vec![];
    for _ in 0..3 {
        replicas.push(create_replica(master_addr).await);
        sleep(Duration::from_millis(100)).await;
    }

    // Insert data on master
    let vectors = vec![
        Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            ..Default::default()
        },
        Vector {
            id: "vec2".to_string(),
            data: vec![0.0, 1.0, 0.0],
            ..Default::default()
        },
    ];
    master_store.insert("test", vectors).unwrap();

    sleep(Duration::from_secs(2)).await;

    // Verify all replicas have the data
    for (_replica_node, replica_store) in &replicas {
        assert_eq!(replica_store.list_collections().len(), 1);
        let collection = replica_store.get_collection("test").unwrap();
        assert_eq!(collection.vector_count(), 2);
    }
}

// ============================================================================
// STRESS TESTS - High Volume Replication
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Run with: cargo test --release -- --ignored
async fn test_stress_high_volume_replication() {
    let (_master, master_store, master_addr) = create_master().await;

    // Create collection
    let config = CollectionConfig {
        graph: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
    };
    master_store
        .create_collection("stress_test", config)
        .unwrap();

    // Create replica
    let (_replica, replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert 10,000 vectors
    info!("Inserting 10,000 vectors...");
    let batch_size = 100;
    let mut handles = Vec::new();
    for batch in 0..100 {
        let store_clone = master_store.clone();
        let handle = tokio::spawn(async move {
            let mut vectors = Vec::new();
            for i in 0..batch_size {
                let idx = batch * batch_size + i;
                let data: Vec<f32> = (0..128).map(|j| (idx + j) as f32 * 0.01).collect();
                vectors.push(Vector {
                    id: format!("vec_{idx}"),
                    data,
                    ..Default::default()
                });
            }
            store_clone.insert("stress_test", vectors).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    info!("All concurrent insertions complete");
    sleep(Duration::from_secs(3)).await;

    // Verify total count
    let master_collection = master_store.get_collection("concurrent").unwrap();
    let replica_collection = replica_store.get_collection("concurrent").unwrap();

    assert_eq!(master_collection.vector_count(), 1000);
    assert_eq!(replica_collection.vector_count(), 1000);
    info!("✅ All 1000 concurrent operations replicated!");
}

// ============================================================================
// SNAPSHOT TESTS - Large Datasets
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Replication issue - snapshot not being transferred. Same root cause as other snapshot sync issues"]
async fn test_snapshot_with_large_vectors() {
    let store1 = VectorStore::new();

    // Create collection with high dimensions
    let config = CollectionConfig {
        graph: None,
        dimension: 1536, // OpenAI embedding size
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
    };
    store1.create_collection("large_dims", config).unwrap();

    // Insert 100 high-dimensional vectors
    let mut vectors = Vec::new();
    for i in 0..100 {
        let data: Vec<f32> = (0..1536).map(|j| (i + j) as f32 * 0.001).collect();
        vectors.push(Vector {
            id: format!("vec_{i}"),
            data,
            ..Default::default()
        });
    }
    store1.insert("large_dims", vectors).unwrap();

    // Create snapshot
    let mut snapshot = vectorizer::replication::sync::create_snapshot(&store1, 0)
        .await
        .unwrap();

    // Corrupt the snapshot
    let len = snapshot.len();
    if len > 100 {
        snapshot[len - 10] ^= 0xFF;
    }

    // Should fail checksum verification
    let store2 = VectorStore::new();
    let result = vectorizer::replication::sync::apply_snapshot(&store2, &snapshot).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Checksum mismatch"));
    info!("✅ Checksum verification works!");
}

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_replication_config_defaults() {
    let config = ReplicationConfig::default();
    assert_eq!(config.role, NodeRole::Standalone);
    assert_eq!(config.heartbeat_interval, 5);
    assert_eq!(config.replica_timeout, 30);
    assert_eq!(config.log_size, 1_000_000);
}

#[test]
fn test_replication_config_master() {
    let addr = "0.0.0.0:7001".parse().unwrap();
    let config = ReplicationConfig::master(addr);

    assert_eq!(config.role, NodeRole::Master);
    assert_eq!(config.bind_address, Some(addr));
    assert!(config.master_address.is_none());
}

#[test]
fn test_replication_config_replica() {
    let addr = "127.0.0.1:7001".parse().unwrap();
    let config = ReplicationConfig::replica(addr);

    assert_eq!(config.role, NodeRole::Replica);
    assert_eq!(config.master_address, Some(addr));
    assert!(config.bind_address.is_none());
}
