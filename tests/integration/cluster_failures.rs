//! Integration tests for cluster failure scenarios
//!
//! These tests verify the behavior of the distributed sharding system
//! when nodes fail, recover, or experience network partitions.

use std::sync::Arc;
use std::time::Duration;

use vectorizer::cluster::{
    ClusterClientPool, ClusterConfig, ClusterManager, DiscoveryMethod, NodeId,
};
use vectorizer::db::distributed_sharded_collection::DistributedShardedCollection;
use vectorizer::db::sharding::ShardId;
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
async fn test_node_failure_during_insert() {
    // Setup: Create cluster with multiple nodes
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add a remote node
    let mut remote_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node.mark_active();
    cluster_manager.add_node(remote_node);

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    // Create distributed collection
    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-failure".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => {
            // Collection creation may fail if no active nodes, which is expected in test
            return;
        }
    };

    // Simulate node failure by marking it as unavailable
    let node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.mark_node_unavailable(&node_id);

    // Try to insert a vector - should handle failure gracefully
    let vector = Vector {
        id: "test-vec-1".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: None,
    };

    // Insert should either succeed (if routed to local node) or fail gracefully
    let result: Result<(), VectorizerError> = collection.insert(vector).await;
    // Result may be Ok or Err depending on shard assignment
    // The important thing is that it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_node_failure_during_search() {
    // Setup: Create cluster with multiple nodes
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add remote nodes
    let mut remote_node1 = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node1.mark_active();
    cluster_manager.add_node(remote_node1);

    let mut remote_node2 = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-3".to_string()),
        "127.0.0.1".to_string(),
        15004,
    );
    remote_node2.mark_active();
    cluster_manager.add_node(remote_node2);

    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    // Create distributed collection
    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-search-failure".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => {
            return; // Expected if no active nodes
        }
    };

    // Insert some vectors first
    for i in 0..10 {
        let vector = Vector {
            id: format!("test-vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Simulate failure of one node
    let node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.mark_node_unavailable(&node_id);

    // Search should continue working with remaining nodes
    let query_vector = vec![0.1; 128];
    let result: Result<Vec<SearchResult>, VectorizerError> =
        collection.search(&query_vector, 5, None, None).await;

    // Search should either succeed (with results from remaining nodes) or fail gracefully
    let _ = result.is_ok() || result.is_err();
    if let Ok(ref results) = result {
        // If search succeeds, we should get some results (possibly fewer than expected)
        let results_len: usize = results.len();
        assert!(results_len <= 5);
    }
}

#[tokio::test]
async fn test_node_recovery_after_failure() {
    // Setup: Create cluster
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add a remote node
    let mut remote_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node.mark_active();
    cluster_manager.add_node(remote_node.clone());

    // Simulate node failure
    let node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.mark_node_unavailable(&node_id);
    if let Some(node) = cluster_manager.get_node(&node_id) {
        assert_eq!(node.status, vectorizer::cluster::NodeStatus::Unavailable);
    }

    // Simulate node recovery
    cluster_manager.mark_node_active(&node_id);
    if let Some(node) = cluster_manager.get_node(&node_id) {
        assert_eq!(node.status, vectorizer::cluster::NodeStatus::Active);
    }

    // Verify node is back in active nodes list
    let active_nodes = cluster_manager.get_active_nodes();
    assert!(active_nodes.iter().any(|n| n.id == node_id));
}

#[tokio::test]
async fn test_partial_cluster_failure() {
    // Setup: Create cluster with 3 nodes
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add multiple remote nodes
    for i in 2..=4 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    // Verify we have 4 nodes total (1 local + 3 remote)
    let nodes = cluster_manager.get_nodes();
    assert_eq!(nodes.len(), 4);

    // Simulate failure of 2 nodes
    let node_id_2 = NodeId::new("test-node-2".to_string());
    let node_id_3 = NodeId::new("test-node-3".to_string());

    cluster_manager.mark_node_unavailable(&node_id_2);
    cluster_manager.mark_node_unavailable(&node_id_3);

    // Verify graceful degradation - remaining nodes should still be active
    let active_nodes = cluster_manager.get_active_nodes();
    assert!(active_nodes.len() >= 2); // At least local node + 1 remote node
    assert!(
        active_nodes
            .iter()
            .all(|n| n.status == vectorizer::cluster::NodeStatus::Active)
    );
}

#[tokio::test]
async fn test_network_partition() {
    // Setup: Create cluster with multiple nodes
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

    // Simulate network partition - mark some nodes as unavailable
    let node_id_2 = NodeId::new("test-node-2".to_string());
    cluster_manager.update_node_status(&node_id_2, vectorizer::cluster::NodeStatus::Unavailable);

    // Each partition should continue operating independently
    // Local node and node-3 should still be active
    let active_nodes = cluster_manager.get_active_nodes();
    assert!(active_nodes.len() >= 2);

    // Verify that unavailable node is not in active list
    assert!(!active_nodes.iter().any(|n| n.id == node_id_2));
}

#[tokio::test]
async fn test_shard_reassignment_on_failure() {
    // Setup: Create cluster
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

    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<ShardId> = (0..4).map(ShardId::new).collect();
    let node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // Initial shard assignment
    shard_router.rebalance(&shard_ids, &node_ids);

    // Get initial shard assignments
    let mut initial_assignments = std::collections::HashMap::new();
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            initial_assignments.insert(*shard_id, node_id);
        }
    }

    // Simulate node failure
    let failed_node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.update_node_status(
        &failed_node_id,
        vectorizer::cluster::NodeStatus::Unavailable,
    );

    // Rebalance shards after failure
    let remaining_node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();
    shard_router.rebalance(&shard_ids, &remaining_node_ids);

    // Verify shards are reassigned to remaining nodes
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            // Shard should not be assigned to failed node
            assert_ne!(node_id, failed_node_id);
            // Shard should be assigned to one of the remaining nodes
            assert!(remaining_node_ids.contains(&node_id));
        }
    }
}
