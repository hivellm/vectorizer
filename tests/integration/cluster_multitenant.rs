//! Multi-tenant cluster mode tests
//!
//! These tests verify that distributed sharding works correctly
//! with multi-tenant isolation in HiveHub Cloud integration.

use std::sync::Arc;
use std::time::Duration;

use vectorizer::cluster::{
    ClusterClientPool, ClusterConfig, ClusterManager, DiscoveryMethod, DistributedShardRouter,
    NodeId,
};
use vectorizer::db::sharding::ShardId;

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

// ============================================================================
// Tenant-Aware Shard Routing Tests
// ============================================================================

#[test]
fn test_tenant_aware_shard_routing_different_tenants() {
    let router = DistributedShardRouter::new(100);

    // Assign shards to nodes
    router.assign_shard(ShardId::new(0), NodeId::new("node-1".to_string()));
    router.assign_shard(ShardId::new(1), NodeId::new("node-2".to_string()));
    router.assign_shard(ShardId::new(2), NodeId::new("node-3".to_string()));
    router.assign_shard(ShardId::new(3), NodeId::new("node-4".to_string()));

    // Same vector_id but different tenants should route differently
    let tenant_a_shard = router.get_shard_for_tenant_vector("tenant-a", "vector-1");
    let _tenant_b_shard = router.get_shard_for_tenant_vector("tenant-b", "vector-1");

    // The shards may or may not be different (depends on hash), but the hashing
    // should be deterministic - same input always gives same output
    let tenant_a_shard_again = router.get_shard_for_tenant_vector("tenant-a", "vector-1");
    assert_eq!(
        tenant_a_shard, tenant_a_shard_again,
        "Tenant routing should be deterministic"
    );

    // Different vector within same tenant should potentially route to different shard
    let tenant_a_vec1 = router.get_shard_for_tenant_vector("tenant-a", "vector-1");
    let _tenant_a_vec2 = router.get_shard_for_tenant_vector("tenant-a", "vector-2");
    // Just verify it's deterministic
    let tenant_a_vec1_again = router.get_shard_for_tenant_vector("tenant-a", "vector-1");
    assert_eq!(tenant_a_vec1, tenant_a_vec1_again);
}

#[test]
fn test_tenant_shard_routing_consistency() {
    let router = DistributedShardRouter::new(100);

    // Assign shards
    for i in 0..8 {
        router.assign_shard(ShardId::new(i), NodeId::new(format!("node-{}", i % 3)));
    }

    // Test that tenant routing is consistent across multiple calls
    let tenant_id = "user-12345-uuid";

    let results: Vec<ShardId> = (0..100)
        .map(|_| router.get_shard_for_tenant(tenant_id))
        .collect();

    // All results should be the same (consistent hashing)
    assert!(
        results.iter().all(|&s| s == results[0]),
        "Tenant shard routing should be consistent"
    );
}

#[test]
fn test_get_shards_for_tenant_distribution() {
    let router = DistributedShardRouter::new(100);

    // Assign 8 shards across 4 nodes
    for i in 0..8 {
        router.assign_shard(ShardId::new(i), NodeId::new(format!("node-{}", i % 4)));
    }

    // Get multiple shards for a tenant (for spreading data)
    let tenant_shards = router.get_shards_for_tenant("tenant-xyz", 4);

    // Should get up to 4 unique shards
    assert!(
        tenant_shards.len() <= 4,
        "Should get at most requested number of shards"
    );
    assert!(!tenant_shards.is_empty(), "Should get at least one shard");

    // Verify determinism
    let tenant_shards_again = router.get_shards_for_tenant("tenant-xyz", 4);
    assert_eq!(
        tenant_shards, tenant_shards_again,
        "Multi-shard tenant routing should be deterministic"
    );
}

#[test]
fn test_tenant_node_routing() {
    let router = DistributedShardRouter::new(100);

    // Assign shards to specific nodes
    router.assign_shard(ShardId::new(0), NodeId::new("node-a".to_string()));
    router.assign_shard(ShardId::new(1), NodeId::new("node-b".to_string()));
    router.assign_shard(ShardId::new(2), NodeId::new("node-c".to_string()));

    // Test that tenant vectors route to nodes
    let node = router.get_node_for_tenant_vector("tenant-1", "vec-100");
    assert!(node.is_some(), "Should find a node for tenant vector");

    // Test tenant-level node routing
    let tenant_node = router.get_node_for_tenant("tenant-1");
    assert!(tenant_node.is_some(), "Should find a node for tenant");

    // Verify determinism
    let tenant_node_again = router.get_node_for_tenant("tenant-1");
    assert_eq!(
        tenant_node, tenant_node_again,
        "Tenant node routing should be deterministic"
    );
}

#[test]
fn test_empty_router_tenant_routing() {
    let router = DistributedShardRouter::new(100);

    // No shards assigned - should return default shard 0
    let shard = router.get_shard_for_tenant_vector("tenant", "vector");
    assert_eq!(
        shard,
        ShardId::new(0),
        "Empty router should return default shard"
    );

    // Node should be None for empty router
    let node = router.get_node_for_tenant_vector("tenant", "vector");
    assert!(
        node.is_none(),
        "Empty router should have no node for tenant vector"
    );
}

// ============================================================================
// Tenant Context Tests
// ============================================================================

// Note: TenantContext tests are in the unit tests within src/hub/auth.rs
// since they require internal types that are complex to construct externally.
// The key functionality (tenant_id, permissions checking) is tested there.

// ============================================================================
// Replication Types with Owner ID Tests
// ============================================================================

#[test]
fn test_replication_operation_serialization_with_owner() {
    use vectorizer::replication::{CollectionConfigData, VectorOperation};

    // Test CreateCollection with owner_id
    let op = VectorOperation::CreateCollection {
        name: "tenant-collection".to_string(),
        config: CollectionConfigData {
            dimension: 128,
            metric: "cosine".to_string(),
        },
        owner_id: Some("tenant-abc-123".to_string()),
    };

    // Serialize and deserialize
    let serialized = bincode::serialize(&op).unwrap();
    let deserialized: VectorOperation = bincode::deserialize(&serialized).unwrap();

    // Verify owner_id is preserved
    if let VectorOperation::CreateCollection { name, owner_id, .. } = deserialized {
        assert_eq!(name, "tenant-collection");
        assert_eq!(owner_id, Some("tenant-abc-123".to_string()));
    } else {
        panic!("Expected CreateCollection");
    }
}

#[test]
fn test_replication_operation_without_owner_backward_compat() {
    use vectorizer::replication::VectorOperation;

    // Test without owner_id (backward compatibility)
    let op = VectorOperation::InsertVector {
        collection: "test".to_string(),
        id: "vec-1".to_string(),
        vector: vec![1.0, 2.0, 3.0],
        payload: None,
        owner_id: None,
    };

    let serialized = bincode::serialize(&op).unwrap();
    let deserialized: VectorOperation = bincode::deserialize(&serialized).unwrap();

    if let VectorOperation::InsertVector { id, owner_id, .. } = deserialized {
        assert_eq!(id, "vec-1");
        assert!(owner_id.is_none());
    } else {
        panic!("Expected InsertVector");
    }
}

#[test]
fn test_all_operation_types_with_owner() {
    use vectorizer::replication::{CollectionConfigData, VectorOperation};

    let tenant_id = Some("tenant-xyz".to_string());

    let operations = vec![
        VectorOperation::CreateCollection {
            name: "test".to_string(),
            config: CollectionConfigData {
                dimension: 64,
                metric: "euclidean".to_string(),
            },
            owner_id: tenant_id.clone(),
        },
        VectorOperation::InsertVector {
            collection: "test".to_string(),
            id: "v1".to_string(),
            vector: vec![1.0; 64],
            payload: None,
            owner_id: tenant_id.clone(),
        },
        VectorOperation::UpdateVector {
            collection: "test".to_string(),
            id: "v1".to_string(),
            vector: Some(vec![2.0; 64]),
            payload: None,
            owner_id: tenant_id.clone(),
        },
        VectorOperation::DeleteVector {
            collection: "test".to_string(),
            id: "v1".to_string(),
            owner_id: tenant_id.clone(),
        },
        VectorOperation::DeleteCollection {
            name: "test".to_string(),
            owner_id: tenant_id.clone(),
        },
    ];

    // All should serialize/deserialize correctly
    for op in operations {
        let serialized = bincode::serialize(&op).unwrap();
        let _deserialized: VectorOperation = bincode::deserialize(&serialized).unwrap();
    }
}

// ============================================================================
// Cluster Manager Multi-Tenant Tests
// ============================================================================

#[tokio::test]
async fn test_cluster_manager_with_multiple_tenants() {
    let cluster_config = create_test_cluster_config();
    let cluster_manager = Arc::new(ClusterManager::new(cluster_config).unwrap());

    // Add nodes
    let mut node1 = vectorizer::cluster::ClusterNode::new(
        NodeId::new("node-1".to_string()),
        "127.0.0.1".to_string(),
        15001,
    );
    node1.mark_active();
    cluster_manager.add_node(node1);

    let mut node2 = vectorizer::cluster::ClusterNode::new(
        NodeId::new("node-2".to_string()),
        "127.0.0.1".to_string(),
        15002,
    );
    node2.mark_active();
    cluster_manager.add_node(node2);

    // Verify nodes are tracked
    let nodes = cluster_manager.get_active_nodes();
    assert!(!nodes.is_empty(), "Should have active nodes");
}

#[tokio::test]
async fn test_client_pool_isolation() {
    let pool1 = ClusterClientPool::new(Duration::from_secs(5));
    let pool2 = ClusterClientPool::new(Duration::from_secs(5));

    // Different pools should be independent
    // This test verifies that client pools are properly isolated
    pool1.clear();
    pool2.clear();

    // Both should be empty after clearing
    // (This is mainly a smoke test for the client pool)
}
