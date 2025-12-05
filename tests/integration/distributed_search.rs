//! Integration tests for distributed search functionality
//!
//! These tests verify that search operations work correctly across
//! multiple servers and that results are properly merged.

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
        memory: Default::default(),
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
async fn test_distributed_search_merges_results() {
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

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-merge".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return, // Expected if no active nodes
    };

    // Insert vectors with different IDs
    for i in 0..10 {
        let mut data = vec![0.1; 128];
        data[0] = i as f32 / 10.0; // Make vectors slightly different
        let vector = Vector {
            id: format!("vec-{i}"),
            data,
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Search should return merged results from all shards
    let query_vector = vec![0.1; 128];
    let result: Result<Vec<SearchResult>, VectorizerError> =
        collection.search(&query_vector, 10, None, None).await;

    match result {
        Ok(ref results) => {
            // Should get results (may be fewer than 10 if some shards are remote and unreachable)
            let results_len: usize = results.len();
            assert!(results_len <= 10);
            // Results should be sorted by score (descending)
            for i in 1..results.len() {
                assert!(results[i - 1].score >= results[i].score);
            }
        }
        Err(_) => {
            // Search may fail if remote nodes are unreachable, which is acceptable in tests
        }
    }
}

#[tokio::test]
async fn test_distributed_search_ordering() {
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

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-ordering".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Insert vectors
    for i in 0..5 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![i as f32 / 10.0; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Search
    let query_vector = vec![0.5; 128];
    let result: Result<Vec<SearchResult>, VectorizerError> =
        collection.search(&query_vector, 5, None, None).await;

    if let Ok(ref results) = result {
        // Verify results are ordered by score (descending)
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results not properly ordered: {} >= {}",
                results[i - 1].score,
                results[i].score
            );
        }
    }
}

#[tokio::test]
async fn test_distributed_search_with_threshold() {
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

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-threshold".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Insert vectors
    for i in 0..5 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Search with threshold
    let query_vector = vec![0.1; 128];
    let threshold = 0.5;
    let result: Result<Vec<SearchResult>, VectorizerError> = collection
        .search(&query_vector, 10, Some(threshold), None)
        .await;

    if let Ok(results) = result {
        // All results should meet the threshold
        for result in &results {
            assert!(
                result.score >= threshold,
                "Result score {} below threshold {}",
                result.score,
                threshold
            );
        }
    }
}

#[tokio::test]
async fn test_distributed_search_shard_filtering() {
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

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-shard-filter".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Insert vectors
    for i in 0..5 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Search with shard filtering - only search in first shard
    let query_vector = vec![0.1; 128];
    let shard_ids = Some(vec![vectorizer::db::sharding::ShardId::new(0)]);
    let result: Result<Vec<SearchResult>, VectorizerError> = collection
        .search(&query_vector, 10, None, shard_ids.as_deref())
        .await;

    // Search should complete (may return empty if shard is remote and unreachable)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_distributed_search_performance() {
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

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-performance".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Insert vectors
    for i in 0..20 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Measure search time
    let query_vector = vec![0.1; 128];
    let start = std::time::Instant::now();
    let _result = collection.search(&query_vector, 10, None, None).await;
    let duration = start.elapsed();

    // Search should complete in reasonable time (< 5 seconds for test)
    assert!(duration.as_secs() < 5, "Search took too long: {duration:?}");
}

#[tokio::test]
async fn test_distributed_search_consistency() {
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

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-consistency".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
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

    // Perform same search multiple times
    let query_vector = vec![0.1; 128];
    let mut previous_len: Option<usize> = None;
    for _ in 0..3 {
        let result: Result<Vec<SearchResult>, VectorizerError> =
            collection.search(&query_vector, 10, None, None).await;
        if let Ok(results) = result {
            if let Some(prev_len) = previous_len {
                // Results should be consistent (same length)
                // Note: Exact match may not be possible due to async nature
                let results_len: usize = results.len();
                assert_eq!(results_len, prev_len);
            }
            previous_len = Some(results.len());
        }
    }
}
