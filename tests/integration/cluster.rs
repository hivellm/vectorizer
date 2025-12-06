//! Integration tests for distributed cluster functionality

use std::sync::Arc;
use std::time::Duration;

use vectorizer::cluster::{
    ClusterClientPool, ClusterConfig, ClusterManager, DiscoveryMethod, NodeId, ServerConfig,
};
use vectorizer::db::sharding::ShardId;

#[tokio::test]
async fn test_cluster_manager_initialization() {
    let config = ClusterConfig {
        enabled: true,
        node_id: Some("test-node-1".to_string()),
        servers: Vec::new(),
        discovery: DiscoveryMethod::Static,
        timeout_ms: 5000,
        retry_count: 3,
        memory: Default::default(),
    };

    let manager = ClusterManager::new(config).unwrap();
    assert!(manager.is_enabled());
    assert_eq!(manager.local_node_id().as_str(), "test-node-1");
}

#[tokio::test]
async fn test_cluster_manager_add_remove_node() {
    let config = ClusterConfig {
        enabled: true,
        node_id: Some("test-node-1".to_string()),
        servers: Vec::new(),
        discovery: DiscoveryMethod::Static,
        timeout_ms: 5000,
        retry_count: 3,
        memory: Default::default(),
    };

    let manager = Arc::new(ClusterManager::new(config).unwrap());

    // Add a node
    let node = vectorizer::cluster::ClusterNode::new(
        NodeId::new("test-node-2".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );
    manager.add_node(node);

    // Verify node was added
    let nodes = manager.get_nodes();
    assert_eq!(nodes.len(), 2); // local node + added node

    // Remove the node
    let node_id = NodeId::new("test-node-2".to_string());
    let removed = manager.remove_node(&node_id);
    assert!(removed.is_some());

    // Verify node was removed
    let nodes_after = manager.get_nodes();
    assert_eq!(nodes_after.len(), 1); // only local node
}

#[tokio::test]
async fn test_distributed_shard_router() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    // Create test shards and nodes
    let shard_ids = [
        ShardId::new(0),
        ShardId::new(1),
        ShardId::new(2),
        ShardId::new(3),
    ];

    let node_ids = [
        NodeId::new("node-1".to_string()),
        NodeId::new("node-2".to_string()),
    ];

    // Assign shards to nodes
    router.assign_shard(shard_ids[0], node_ids[0].clone());
    router.assign_shard(shard_ids[1], node_ids[0].clone());
    router.assign_shard(shard_ids[2], node_ids[1].clone());
    router.assign_shard(shard_ids[3], node_ids[1].clone());

    // Verify assignments
    assert_eq!(
        router.get_node_for_shard(&shard_ids[0]),
        Some(node_ids[0].clone())
    );
    assert_eq!(
        router.get_node_for_shard(&shard_ids[2]),
        Some(node_ids[1].clone())
    );

    // Verify shards per node
    let node1_shards = router.get_shards_for_node(&node_ids[0]);
    assert_eq!(node1_shards.len(), 2);

    let node2_shards = router.get_shards_for_node(&node_ids[1]);
    assert_eq!(node2_shards.len(), 2);
}

#[tokio::test]
async fn test_distributed_shard_router_rebalance() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    // Create test shards and nodes
    let shard_ids: Vec<ShardId> = (0..8).map(ShardId::new).collect();
    let node_ids = vec![
        NodeId::new("node-1".to_string()),
        NodeId::new("node-2".to_string()),
    ];

    // Rebalance shards
    router.rebalance(&shard_ids, &node_ids);

    // Verify shards are distributed
    let node1_shards = router.get_shards_for_node(&node_ids[0]);
    let node2_shards = router.get_shards_for_node(&node_ids[1]);

    // Should be roughly even distribution (4 shards each for 8 shards, 2 nodes)
    assert_eq!(node1_shards.len() + node2_shards.len(), 8);
    assert!(node1_shards.len() >= 3); // Allow some variance
    assert!(node2_shards.len() >= 3);
}

#[tokio::test]
async fn test_distributed_shard_router_vector_routing() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    // Create test shards and nodes
    let shard_ids = [ShardId::new(0), ShardId::new(1)];

    let node_ids = [
        NodeId::new("node-1".to_string()),
        NodeId::new("node-2".to_string()),
    ];

    router.assign_shard(shard_ids[0], node_ids[0].clone());
    router.assign_shard(shard_ids[1], node_ids[1].clone());

    // Test vector routing
    let vector_id_1 = "vector-1";
    let shard_for_vector = router.get_shard_for_vector(vector_id_1);

    // Should route to one of the assigned shards
    assert!(shard_ids.contains(&shard_for_vector));

    // Get node for vector
    let node_for_vector = router.get_node_for_vector(vector_id_1);
    assert!(node_for_vector.is_some());
}

#[tokio::test]
async fn test_cluster_client_pool() {
    let timeout = Duration::from_secs(5);
    let _pool = ClusterClientPool::new(timeout);

    // Test that pool is created
    // Note: Actual connection tests would require a running server
    // This just verifies the pool structure
    // Placeholder - pool creation doesn't fail
}

#[tokio::test]
async fn test_cluster_config_serialization() {
    let config = ClusterConfig {
        enabled: true,
        node_id: Some("test-node".to_string()),
        servers: vec![
            ServerConfig {
                id: "node-1".to_string(),
                address: "127.0.0.1".to_string(),
                grpc_port: 15003,
            },
            ServerConfig {
                id: "node-2".to_string(),
                address: "127.0.0.1".to_string(),
                grpc_port: 15004,
            },
        ],
        discovery: DiscoveryMethod::Static,
        timeout_ms: 5000,
        retry_count: 3,
        memory: Default::default(),
    };

    // Test serialization
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("enabled: true"));
    assert!(yaml.contains("test-node"));

    // Test deserialization
    let deserialized: ClusterConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.node_id, config.node_id);
    assert_eq!(deserialized.servers.len(), 2);
}

#[tokio::test]
async fn test_cluster_node_health_checking() {
    use vectorizer::cluster::{ClusterNode, NodeStatus};

    let mut node = ClusterNode::new(
        NodeId::new("test-node".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );

    // Initially should be healthy (just created)
    assert!(node.is_healthy(Duration::from_secs(10)));

    // Update heartbeat
    node.update_heartbeat();
    assert!(node.is_healthy(Duration::from_secs(10)));

    // Mark as unavailable
    node.mark_unavailable();
    assert_eq!(node.status, NodeStatus::Unavailable);

    // Mark as active again
    node.mark_active();
    assert_eq!(node.status, NodeStatus::Active);
}

#[tokio::test]
async fn test_cluster_node_shard_management() {
    use vectorizer::cluster::ClusterNode;

    let mut node = ClusterNode::new(
        NodeId::new("test-node".to_string()),
        "127.0.0.1".to_string(),
        15003,
    );

    // Add shards
    node.add_shard(ShardId::new(0));
    node.add_shard(ShardId::new(1));
    node.add_shard(ShardId::new(2));

    assert_eq!(node.shard_count(), 3);
    assert!(node.has_shard(&ShardId::new(0)));
    assert!(node.has_shard(&ShardId::new(1)));
    assert!(!node.has_shard(&ShardId::new(3)));

    // Remove shard
    let removed = node.remove_shard(&ShardId::new(1));
    assert!(removed);
    assert_eq!(node.shard_count(), 2);
    assert!(!node.has_shard(&ShardId::new(1)));
}

#[tokio::test]
async fn test_shard_migration() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    let shard_id = ShardId::new(0);
    let node1 = NodeId::new("node-1".to_string());
    let node2 = NodeId::new("node-2".to_string());

    // Assign shard to node1
    router.assign_shard(shard_id, node1.clone());
    assert_eq!(router.get_node_for_shard(&shard_id), Some(node1.clone()));

    // Migrate shard from node1 to node2
    let previous_node = router.migrate_shard(shard_id, &node1, &node2);
    assert_eq!(previous_node, Some(node1.clone()));
    assert_eq!(router.get_node_for_shard(&shard_id), Some(node2.clone()));

    // Verify shard is no longer on node1
    let node1_shards = router.get_shards_for_node(&node1);
    assert!(!node1_shards.contains(&shard_id));

    // Verify shard is now on node2
    let node2_shards = router.get_shards_for_node(&node2);
    assert!(node2_shards.contains(&shard_id));
}

#[tokio::test]
async fn test_shard_migration_invalid() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    let shard_id = ShardId::new(0);
    let node1 = NodeId::new("node-1".to_string());
    let node2 = NodeId::new("node-2".to_string());
    let node3 = NodeId::new("node-3".to_string());

    // Assign shard to node1
    router.assign_shard(shard_id, node1.clone());

    // Try to migrate from wrong node (should fail)
    let result = router.migrate_shard(shard_id, &node2, &node3);
    assert_eq!(result, None);
    assert_eq!(router.get_node_for_shard(&shard_id), Some(node1.clone()));
}

#[tokio::test]
async fn test_calculate_migration_plan() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    // Create 6 shards
    let shard_ids: Vec<ShardId> = (0..6).map(ShardId::new).collect();
    let node1 = NodeId::new("node-1".to_string());
    let node2 = NodeId::new("node-2".to_string());

    // Assign all shards to node1 (uneven distribution)
    for shard_id in &shard_ids {
        router.assign_shard(*shard_id, node1.clone());
    }

    // Calculate migration plan to balance across 2 nodes
    let migrations = router.calculate_migration_plan(&shard_ids, &[node1.clone(), node2.clone()]);

    // Should have migrations to balance (at least 2-3 shards should move)
    assert!(!migrations.is_empty());
    assert!(migrations.len() >= 2); // At least 2 shards should migrate

    // Verify all migrations are from node1 to node2
    for (_, from, to) in &migrations {
        assert_eq!(*from, node1);
        assert_eq!(*to, node2);
    }

    // Verify shard IDs in migrations are valid
    for (shard_id, _, _) in &migrations {
        assert!(shard_ids.contains(shard_id));
    }
}

#[tokio::test]
async fn test_calculate_migration_plan_empty_nodes() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    let shard_ids: Vec<ShardId> = (0..6).map(ShardId::new).collect();

    // Calculate migration plan with no nodes (should return empty)
    let migrations = router.calculate_migration_plan(&shard_ids, &[]);
    assert!(migrations.is_empty());
}

#[tokio::test]
async fn test_calculate_migration_plan_already_balanced() {
    let router = vectorizer::cluster::DistributedShardRouter::new(100);

    // Create 4 shards
    let shard_ids: Vec<ShardId> = (0..4).map(ShardId::new).collect();
    let node1 = NodeId::new("node-1".to_string());
    let node2 = NodeId::new("node-2".to_string());

    // Assign 2 shards to each node (already balanced)
    router.assign_shard(shard_ids[0], node1.clone());
    router.assign_shard(shard_ids[1], node1.clone());
    router.assign_shard(shard_ids[2], node2.clone());
    router.assign_shard(shard_ids[3], node2.clone());

    // Calculate migration plan
    let migrations = router.calculate_migration_plan(&shard_ids, &[node1.clone(), node2.clone()]);

    // Should have minimal or no migrations (already balanced)
    assert!(migrations.len() <= 1); // Allow for minor adjustments
}
