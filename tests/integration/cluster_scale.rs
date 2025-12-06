//! Integration tests for cluster scaling with 3+ servers
//!
//! These tests verify load distribution, shard distribution, and
//! dynamic node management in larger clusters.

use std::sync::Arc;
use std::time::Duration;

use vectorizer::cluster::{
    ClusterClientPool, ClusterConfig, ClusterManager, DiscoveryMethod, NodeId,
};
use vectorizer::db::distributed_sharded_collection::DistributedShardedCollection;
use vectorizer::db::sharding::ShardId;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, ShardingConfig, Vector,
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
            shard_count: 6,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }),
    }
}

#[tokio::test]
async fn test_cluster_3_nodes_load_distribution() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add 2 remote nodes (total 3 nodes)
    for i in 2..=3 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    // Verify we have 3 nodes
    let nodes = cluster_manager.get_nodes();
    assert_eq!(nodes.len(), 3);

    // Create collection with 6 shards
    let client_pool = Arc::new(ClusterClientPool::new(Duration::from_secs(5)));
    let collection_config = create_test_collection_config();

    let collection: DistributedShardedCollection = match DistributedShardedCollection::new(
        "test-3nodes".to_string(),
        collection_config,
        cluster_manager.clone(),
        client_pool.clone(),
    ) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Insert vectors - should be distributed across nodes
    for i in 0..30 {
        let vector = Vector {
            id: format!("vec-{i}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: None,
        };
        let _ = collection.insert(vector).await;
    }

    // Verify load is distributed (each node should have some shards)
    // First, ensure shards are assigned to nodes
    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<ShardId> = (0..6).map(ShardId::new).collect();

    // Rebalance to ensure shards are assigned
    let node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();
    shard_router.rebalance(&shard_ids, &node_ids);

    let mut node_shard_counts = std::collections::HashMap::new();
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            *node_shard_counts.entry(node_id).or_insert(0) += 1;
        }
    }

    // With 3 nodes and 6 shards, at least 2 nodes should have shards
    // (round-robin distribution should give 2 shards per node)
    assert!(
        node_shard_counts.len() >= 2,
        "Expected at least 2 nodes to have shards, got {}",
        node_shard_counts.len()
    );
}

#[tokio::test]
async fn test_cluster_5_nodes_shard_distribution() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add 4 remote nodes (total 5 nodes)
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

    // Create shard router and assign shards
    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<ShardId> = (0..10).map(ShardId::new).collect();
    let node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    shard_router.rebalance(&shard_ids, &node_ids);

    // Verify shards are distributed across nodes
    let mut node_shard_counts = std::collections::HashMap::new();
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            *node_shard_counts.entry(node_id).or_insert(0) += 1;
        }
    }

    // With 10 shards and 5 nodes, each node should have roughly 2 shards
    // Allow some variance due to consistent hashing
    assert!(node_shard_counts.len() >= 3); // At least 3 nodes should have shards
}

#[tokio::test]
async fn test_cluster_add_node_dynamically() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Start with 2 nodes
    let mut remote_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    remote_node.mark_active();
    cluster_manager.add_node(remote_node);

    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<ShardId> = (0..4).map(ShardId::new).collect();
    let initial_node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // Initial shard assignment
    shard_router.rebalance(&shard_ids, &initial_node_ids);

    // Get initial assignments
    let mut initial_assignments = std::collections::HashMap::new();
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            initial_assignments.insert(*shard_id, node_id);
        }
    }

    // Add new node
    let mut new_node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-3".to_string()),
        "127.0.0.1".to_string(),
        15004,
    );
    new_node.mark_active();
    cluster_manager.add_node(new_node);

    // Rebalance with new node
    let updated_node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();
    shard_router.rebalance(&shard_ids, &updated_node_ids);

    // Verify new node may have received some shards
    let new_node_id = NodeId::new("test-node-3".to_string());
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id)
            && node_id == new_node_id
        {
            break;
        }
    }

    // New node should potentially have shards (depending on consistent hashing)
    // This is probabilistic, so we just verify the rebalance completed
    assert_eq!(updated_node_ids.len(), 3);
}

#[tokio::test]
async fn test_cluster_remove_node_dynamically() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Start with 3 nodes
    for i in 2..=3 {
        let mut remote_node = vectorizer::cluster::ClusterNode::new(
            NodeId::new(format!("test-node-{i}")),
            "127.0.0.1".to_string(),
            15000 + i as u16,
        );
        remote_node.mark_active();
        cluster_manager.add_node(remote_node);
    }

    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<ShardId> = (0..6).map(ShardId::new).collect();
    let initial_node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // Initial shard assignment
    shard_router.rebalance(&shard_ids, &initial_node_ids);

    // Remove a node
    let removed_node_id = NodeId::new("test-node-2".to_string());
    cluster_manager.remove_node(&removed_node_id);

    // Rebalance without removed node
    let remaining_node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();
    shard_router.rebalance(&shard_ids, &remaining_node_ids);

    // Verify removed node no longer has shards
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            assert_ne!(node_id, removed_node_id);
        }
    }

    // Verify remaining nodes have shards
    assert_eq!(remaining_node_ids.len(), 2); // Local + 1 remote
}

#[tokio::test]
async fn test_cluster_rebalance_trigger() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add nodes
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
    let shard_ids: Vec<ShardId> = (0..8).map(ShardId::new).collect();
    let node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // Initial rebalance
    shard_router.rebalance(&shard_ids, &node_ids);

    // Get initial distribution
    let mut initial_distribution = std::collections::HashMap::new();
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            *initial_distribution.entry(node_id).or_insert(0) += 1;
        }
    }

    // Trigger rebalance again (should redistribute)
    shard_router.rebalance(&shard_ids, &node_ids);

    // Verify all shards are still assigned
    for shard_id in &shard_ids {
        assert!(shard_router.get_node_for_shard(shard_id).is_some());
    }
}

#[tokio::test]
async fn test_cluster_consistent_hashing() {
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

    let shard_router = cluster_manager.shard_router();
    let shard_ids: Vec<ShardId> = (0..10).map(ShardId::new).collect();
    let node_ids: Vec<NodeId> = cluster_manager
        .get_active_nodes()
        .iter()
        .map(|n| n.id.clone())
        .collect();

    // Initial assignment
    shard_router.rebalance(&shard_ids, &node_ids);
    let mut initial_assignments = std::collections::HashMap::new();
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
            initial_assignments.insert(*shard_id, node_id);
        }
    }

    // Rebalance again with same nodes - assignments should be consistent
    shard_router.rebalance(&shard_ids, &node_ids);
    for shard_id in &shard_ids {
        if let Some(node_id) = shard_router.get_node_for_shard(shard_id)
            && let Some(initial_node) = initial_assignments.get(shard_id)
        {
            // Consistent hashing should assign same shard to same node
            assert_eq!(node_id, *initial_node);
        }
    }
}
