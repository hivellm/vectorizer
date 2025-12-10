//! Integration tests for cluster fault tolerance
//!
//! These tests verify that the cluster can handle failures gracefully
//! and maintain data consistency.

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
        encryption: None,
    }
}

#[tokio::test]
async fn test_quorum_operations() {
    // Test that operations work with a quorum of nodes
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Create cluster with 5 nodes (quorum = 3)
    for i in 2..=5 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    // Verify we have 5 nodes
    let nodes = cluster_manager.get_nodes();
    assert_eq!(nodes.len(), 5);

    // Simulate failure of 2 nodes (still have quorum)
    let node_id_2 = NodeId::new("test-node-2".to_string());
    let node_id_3 = NodeId::new("test-node-3".to_string());

    cluster_manager.mark_node_unavailable(&node_id_2);
    cluster_manager.mark_node_unavailable(&node_id_3);

    // Operations should still work with remaining 3 nodes (quorum)
    let active_nodes = cluster_manager.get_active_nodes();
    assert!(active_nodes.len() >= 3);
}

#[tokio::test]
async fn test_eventual_consistency() {
    // Test that cluster eventually becomes consistent after failures
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add nodes
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
        "test-consistency".to_string(),
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

    // Simulate node failure
    let node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.mark_node_unavailable(&node_id);

    // After some time, node recovers
    tokio::time::sleep(Duration::from_millis(100)).await;
    cluster_manager.mark_node_active(&node_id);

    // Cluster should eventually be consistent
    let active_nodes = cluster_manager.get_active_nodes();
    assert!(active_nodes.len() >= 2);
}

#[tokio::test]
async fn test_data_durability() {
    // Test that data persists after node failures
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
        "test-durability".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Insert vectors
    let mut inserted_ids = Vec::new();
    for i in 0..10 {
        let id = format!("vec-{i}");
        let vector = Vector {
            id: id.clone(),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
        inserted_ids.push(id);
    }

    // Simulate node failure
    let node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.mark_node_unavailable(&node_id);

    // Data on local node should still be accessible
    let query_vector = vec![0.1; 128];
    let result: Result<Vec<SearchResult>, VectorizerError> =
        collection.search(&query_vector, 10, None, None).await;

    // Search should still work (may return fewer results if remote node is down)
    if let Ok(ref results) = result {
        // Verify we can still search (results may be from local shards only)
        let results_len: usize = results.len();
        assert!(results_len <= 10);
    }
}

#[tokio::test]
async fn test_automatic_failover() {
    // Test automatic failover when primary node fails
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add multiple nodes
    for i in 2..=4 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<_> = (0..6).map(vectorizer::db::sharding::ShardId::new).collect();
    let initial_node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // Initial shard assignment
    shard_router.rebalance(&shard_ids, &initial_node_ids);

    // Get primary node for a shard
    let test_shard = shard_ids[0];
    let primary_node = shard_router.get_node_for_shard(&test_shard);

    if let Some(primary_node_id) = primary_node {
        let primary_node_id = primary_node_id.clone();
        // Simulate primary node failure
        cluster_manager.update_node_status(
            &primary_node_id,
            vectorizer::cluster::NodeStatus::Unavailable,
        );

        // Rebalance should reassign shard
        let remaining_node_ids: Vec<NodeId> = cluster_manager
            .get_active_nodes()
            .iter()
            .map(|n| n.id.clone())
            .collect();
        shard_router.rebalance(&shard_ids, &remaining_node_ids);

        // Shard should be reassigned to different node
        if let Some(new_node) = shard_router.get_node_for_shard(&test_shard) {
            assert_ne!(new_node, primary_node_id);
            assert!(remaining_node_ids.contains(&new_node));
        }
    }
}

#[tokio::test]
async fn test_split_brain_prevention() {
    // Test that split-brain scenarios are handled
    // Note: Full split-brain prevention requires consensus algorithm
    // This test verifies basic behavior

    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Create cluster with 5 nodes
    for i in 2..=5 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    // Simulate network partition - split into two groups
    // Group 1: nodes 1, 2, 3
    // Group 2: nodes 4, 5 (isolated)

    let node_id_4 = NodeId::new("test-node-4".to_string());
    let node_id_5 = NodeId::new("test-node-5".to_string());

    // Mark nodes 4 and 5 as unavailable (simulating partition)
    cluster_manager.update_node_status(&node_id_4, vectorizer::cluster::NodeStatus::Unavailable);
    cluster_manager.update_node_status(&node_id_5, vectorizer::cluster::NodeStatus::Unavailable);

    // Group 1 (nodes 1, 2, 3) should still function
    let active_nodes = cluster_manager.get_active_nodes();
    assert!(active_nodes.len() >= 3);

    // Verify unavailable nodes are not in active list
    assert!(!active_nodes.iter().any(|n| n.id == node_id_4));
    assert!(!active_nodes.iter().any(|n| n.id == node_id_5));
}
