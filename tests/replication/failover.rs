//! Replication Failover and Reconnection Tests
//!
//! Tests for:
//! - Replica reconnection after disconnect
//! - Partial sync after reconnection
//! - Full sync when offset is too old
//! - Multiple replica recovery
//! - Data consistency after failover

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::time::sleep;
use tracing::info;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, StorageType, Vector,
};
use vectorizer::replication::{MasterNode, NodeRole, ReplicaNode, ReplicationConfig};

static FAILOVER_PORT: AtomicU16 = AtomicU16::new(45000);

fn next_port_failover() -> u16 {
    FAILOVER_PORT.fetch_add(1, Ordering::SeqCst)
}

async fn create_master() -> (Arc<MasterNode>, Arc<VectorStore>, std::net::SocketAddr) {
    let port = next_port_failover();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    let config = ReplicationConfig {
        role: NodeRole::Master,
        bind_address: Some(addr),
        master_address: None,
        heartbeat_interval: 1,
        replica_timeout: 5,
        log_size: 1000,
        reconnect_interval: 1,
    };

    let store = Arc::new(VectorStore::new());
    let master = Arc::new(MasterNode::new(config, Arc::clone(&store)).unwrap());

    let master_clone = Arc::clone(&master);
    tokio::spawn(async move {
        let _ = master_clone.start().await;
    });

    sleep(Duration::from_millis(100)).await;
    (master, store, addr)
}

async fn create_replica(master_addr: std::net::SocketAddr) -> (Arc<ReplicaNode>, Arc<VectorStore>) {
    let config = ReplicationConfig {
        role: NodeRole::Replica,
        bind_address: None,
        master_address: Some(master_addr),
        heartbeat_interval: 1,
        replica_timeout: 5,
        log_size: 1000,
        reconnect_interval: 1,
    };

    let store = Arc::new(VectorStore::new());
    let replica = Arc::new(ReplicaNode::new(config, Arc::clone(&store)));

    let replica_clone = Arc::clone(&replica);
    tokio::spawn(async move {
        let _ = replica_clone.start().await;
    });

    sleep(Duration::from_millis(200)).await;
    (replica, store)
}

// ============================================================================
// Reconnection Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_replica_reconnect_after_disconnect() {
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
        storage_type: Some(StorageType::Memory),
        sharding: None,
        encryption: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Start replica
    let (replica, replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert initial data
    let vec1 = Vector {
        id: "vec1".to_string(),
        data: vec![1.0, 0.0, 0.0],
        ..Default::default()
    };
    master_store.insert("test", vec![vec1]).unwrap();
    sleep(Duration::from_millis(500)).await;

    // Verify replication
    let collection = replica_store.get_collection("test").unwrap();
    assert_eq!(collection.vector_count(), 1);

    // Simulate disconnect and reconnect
    // Note: In a real scenario, we would drop the replica and create a new one
    drop(replica);
    info!("Replica disconnected");

    sleep(Duration::from_secs(2)).await;

    // Insert more data while replica is disconnected
    let vec2 = Vector {
        id: "vec2".to_string(),
        data: vec![0.0, 1.0, 0.0],
        ..Default::default()
    };
    master_store.insert("test", vec![vec2]).unwrap();

    // Recreate replica (simulates reconnection)
    let (_new_replica, new_replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(2)).await;

    // Verify replica caught up
    let new_collection = new_replica_store.get_collection("test").unwrap();
    assert_eq!(new_collection.vector_count(), 2);
    info!("✅ Replica successfully reconnected and caught up!");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_partial_sync_after_brief_disconnect() {
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
        storage_type: Some(StorageType::Memory),
        sharding: None,
        encryption: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Start replica and sync
    let (replica, replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert 10 vectors
    for i in 0..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    sleep(Duration::from_millis(500)).await;

    // Verify sync
    let collection = replica_store.get_collection("test").unwrap();
    assert_eq!(collection.vector_count(), 10);

    // Brief disconnect
    drop(replica);
    sleep(Duration::from_millis(100)).await;

    // Insert a few more while disconnected
    for i in 10..15 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    // Reconnect
    let (_new_replica, new_replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Should use partial sync (offset still in log)
    let new_collection = new_replica_store.get_collection("test").unwrap();
    assert_eq!(new_collection.vector_count(), 15);
    info!("✅ Partial sync successful!");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_full_sync_when_offset_too_old() {
    // Create master with small log size
    let port = next_port_failover();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    let config = ReplicationConfig {
        role: NodeRole::Master,
        bind_address: Some(addr),
        master_address: None,
        heartbeat_interval: 1,
        replica_timeout: 5,
        log_size: 5, // Very small log to force full sync
        reconnect_interval: 1,
    };

    let master_store = Arc::new(VectorStore::new());
    let master = Arc::new(MasterNode::new(config, Arc::clone(&master_store)).unwrap());

    let master_clone = Arc::clone(&master);
    tokio::spawn(async move {
        let _ = master_clone.start().await;
    });
    sleep(Duration::from_millis(100)).await;

    // Create collection
    let col_config = CollectionConfig {
        graph: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        encryption: None,
    };
    master_store.create_collection("test", col_config).unwrap();

    // Start replica
    let (replica, _replica_store) = create_replica(addr).await;
    sleep(Duration::from_secs(1)).await;

    // Insert 3 vectors
    for i in 0..3 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }
    sleep(Duration::from_millis(500)).await;

    // Disconnect replica
    drop(replica);
    sleep(Duration::from_millis(100)).await;

    // Insert many more vectors (exceed log size)
    for i in 3..20 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    // Reconnect - should trigger full sync
    let (_new_replica, new_replica_store) = create_replica(addr).await;
    sleep(Duration::from_secs(2)).await;

    // Verify all data synced via snapshot
    let collection = new_replica_store.get_collection("test").unwrap();
    assert_eq!(collection.vector_count(), 20);
    info!("✅ Full sync triggered successfully!");
}

// ============================================================================
// Multiple Replica Recovery
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_multiple_replicas_recovery() {
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
        storage_type: Some(StorageType::Memory),
        sharding: None,
        encryption: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Create 3 replicas
    let mut replicas = vec![];
    for i in 0..3 {
        let (replica, store) = create_replica(master_addr).await;
        replicas.push((replica, store));
        info!("Replica {i} created");
        sleep(Duration::from_millis(100)).await;
    }

    sleep(Duration::from_secs(1)).await;

    // Insert data
    for i in 0..5 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    sleep(Duration::from_secs(1)).await;

    // Verify all replicas synced
    for (i, (_replica, store)) in replicas.iter().enumerate() {
        let collection = store.get_collection("test").unwrap();
        assert_eq!(collection.vector_count(), 5);
        info!("Replica {i} verified: 5 vectors");
    }

    // Disconnect all replicas
    for (replica, _) in replicas {
        drop(replica);
    }
    info!("All replicas disconnected");

    sleep(Duration::from_millis(200)).await;

    // Insert more data
    for i in 5..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, 0.0, 0.0],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    // Recreate replicas
    let mut new_replicas = vec![];
    for i in 0..3 {
        let (_replica, store) = create_replica(master_addr).await;
        new_replicas.push(store);
        info!("New replica {i} created");
        sleep(Duration::from_millis(100)).await;
    }

    sleep(Duration::from_secs(2)).await;

    // Verify all replicas caught up
    for (i, store) in new_replicas.iter().enumerate() {
        let collection = store.get_collection("test").unwrap();
        assert_eq!(collection.vector_count(), 10);
        info!("New replica {i} caught up: 10 vectors");
    }

    info!("✅ All replicas recovered successfully!");
}

// ============================================================================
// Data Consistency Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Requires TCP connection
async fn test_data_consistency_after_multiple_disconnects() {
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
        storage_type: Some(StorageType::Memory),
        sharding: None,
        encryption: None,
    };
    master_store.create_collection("test", config).unwrap();

    // Initial sync
    let (replica, _replica_store) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Phase 1: Insert and sync
    for i in 0..5 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, (i + 1) as f32, (i + 2) as f32],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }
    sleep(Duration::from_millis(500)).await;

    // Disconnect
    drop(replica);
    sleep(Duration::from_millis(100)).await;

    // Phase 2: Insert more
    for i in 5..10 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, (i + 1) as f32, (i + 2) as f32],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    // Reconnect
    let (replica2, _replica_store2) = create_replica(master_addr).await;
    sleep(Duration::from_secs(1)).await;

    // Disconnect again
    drop(replica2);
    sleep(Duration::from_millis(100)).await;

    // Phase 3: Insert even more
    for i in 10..15 {
        let vec = Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32, (i + 1) as f32, (i + 2) as f32],
            ..Default::default()
        };
        master_store.insert("test", vec![vec]).unwrap();
    }

    // Final reconnect
    let (_replica3, replica_store3) = create_replica(master_addr).await;
    sleep(Duration::from_secs(2)).await;

    // Verify final consistency
    let master_collection = master_store.get_collection("test").unwrap();
    let replica_collection = replica_store3.get_collection("test").unwrap();

    assert_eq!(master_collection.vector_count(), 15);
    assert_eq!(replica_collection.vector_count(), 15);

    // Verify all vectors are identical
    let master_vecs = master_collection.get_all_vectors();
    let replica_vecs = replica_collection.get_all_vectors();

    let mut master_ids: Vec<_> = master_vecs.iter().map(|v| v.id.clone()).collect();
    let mut replica_ids: Vec<_> = replica_vecs.iter().map(|v| v.id.clone()).collect();

    master_ids.sort();
    replica_ids.sort();

    assert_eq!(master_ids, replica_ids);
    info!("✅ Data consistency maintained after multiple disconnects!");
}
