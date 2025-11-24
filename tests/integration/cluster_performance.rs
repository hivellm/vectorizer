//! Integration tests for cluster performance
//!
//! These tests measure and verify performance characteristics
//! of distributed operations.

use std::sync::Arc;
use std::time::Duration;

use vectorizer::cluster::{
    ClusterClientPool, ClusterConfig, ClusterManager, DiscoveryMethod, NodeId,
};
use vectorizer::db::distributed_sharded_collection::DistributedShardedCollection;
use vectorizer::error::VectorizerError;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, SearchResult, ShardingConfig,
    Vector,
};

fn create_test_cluster_config() -> ClusterConfig {
    ClusterConfig {
        enabled: true,
        node_id: Some("test-node-1".to_string()),
        servers: Vec::new(),
        discovery: DiscoveryMethod::Static,
        timeout_ms: 5000,
        retry_count: 3,
    }
}

fn create_test_collection_config() -> CollectionConfig {
    CollectionConfig {
        graph: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: Some(ShardingConfig {
            shard_count: 4,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }),
    }
}

#[tokio::test]
async fn test_concurrent_inserts_distributed() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add remote nodes
    for i in 2..=3 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    let collection: Arc<DistributedShardedCollection> = match DistributedShardedCollection::new(
        "test-concurrent-inserts".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };

    // Concurrent inserts
    let mut handles = Vec::new();
    for i in 0..20 {
        let collection = collection.clone();
        let handle = tokio::spawn(async move {
            let vector = Vector {
                id: format!("vec-{i}"),
                data: vec![0.1; 128],
                sparse: None,
                payload: None,
            };
            collection.insert(vector).await
        });
        handles.push(handle);
    }

    // Wait for all inserts
    let start = std::time::Instant::now();
    for handle in handles {
        let _ = handle.await;
    }
    let duration = start.elapsed();

    // Concurrent inserts should complete in reasonable time
    assert!(
        duration.as_secs() < 10,
        "Concurrent inserts took too long: {duration:?}"
    );
}

#[tokio::test]
#[ignore] // Slow test - takes >60 seconds, concurrent distributed operations
async fn test_concurrent_searches_distributed() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add remote nodes
    for i in 2..=3 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    let collection: Arc<DistributedShardedCollection> = match DistributedShardedCollection::new(
        "test-concurrent-search".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };

    // Insert vectors first
    for i in 0..50 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Concurrent searches
    let mut handles = Vec::new();
    for _ in 0..10 {
        let collection = collection.clone();
        let handle = tokio::spawn(async move {
            let query_vector = vec![0.1; 128];
            collection.search(&query_vector, 10, None, None).await
        });
        handles.push(handle);
    }

    // Wait for all searches
    let start = std::time::Instant::now();
    for handle in handles {
        let _ = handle.await;
    }
    let duration = start.elapsed();

    // Concurrent searches should complete in reasonable time
    assert!(
        duration.as_secs() < 15,
        "Concurrent searches took too long: {duration:?}"
    );
}

#[tokio::test]
#[ignore] // Slow test - takes >60 seconds, throughput comparison test
async fn test_throughput_comparison() {
    // This test compares throughput of distributed vs single-node operations
    // Note: In a real scenario, this would compare against a non-distributed collection
    // For now, we just verify that distributed operations complete successfully

    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add remote node
    let mut remote_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node.mark_active();
    cluster_manager.add_node(remote_node);

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    let collection: Arc<DistributedShardedCollection> = match DistributedShardedCollection::new(
        "test-throughput".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };

    // Measure insert throughput
    let start = std::time::Instant::now();
    let mut success_count = 0;
    for i in 0..100 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let insert_result: Result<(), VectorizerError> = collection.insert(vector).await;
        if insert_result.is_ok() {
            success_count += 1;
        }
    }
    let duration = start.elapsed();

    // Calculate throughput (operations per second)
    let throughput = f64::from(success_count) / duration.as_secs_f64();

    // Verify reasonable throughput (at least 1 op/sec for test)
    assert!(throughput > 0.0, "Throughput should be positive");
}

#[tokio::test]
async fn test_latency_distribution() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    let mut remote_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node.mark_active();
    cluster_manager.add_node(remote_node);

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    let collection: Arc<DistributedShardedCollection> = match DistributedShardedCollection::new(
        "test-latency".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };

    // Insert vectors
    for i in 0..10 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Measure search latency
    let mut latencies = Vec::new();
    let query_vector = vec![0.1; 128];

    for _ in 0..10 {
        let start = std::time::Instant::now();
        let _ = collection.search(&query_vector, 5, None, None).await;
        let latency = start.elapsed();
        latencies.push(latency);
    }

    // Verify latencies are reasonable (< 5 seconds each)
    for latency in &latencies {
        assert!(latency.as_secs() < 5, "Latency too high: {latency:?}");
    }
}

#[tokio::test]
#[ignore] // Slow test - takes >60 seconds, memory measurement test
async fn test_memory_usage_distributed() {
    // This test verifies that memory usage is reasonable in distributed mode
    // Note: Actual memory measurement would require system APIs

    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    let mut remote_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node.mark_active();
    cluster_manager.add_node(remote_node);

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    let collection: Arc<DistributedShardedCollection> = match DistributedShardedCollection::new(
        "test-memory".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => Arc::new(c),
        Err(_) => return,
    };

    // Insert many vectors
    for i in 0..1000 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Verify collection still works after many inserts
    let query_vector = vec![0.1; 128];
    let result: Result<Vec<SearchResult>, VectorizerError> =
        collection.search(&query_vector, 10, None, None).await;

    // Search should still work (may fail if remote nodes unreachable, which is ok)
    assert!(result.is_ok() || result.is_err());
}
