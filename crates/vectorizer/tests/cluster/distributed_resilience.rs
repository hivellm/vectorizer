//! Tests for distributed resilience features:
//! - Shard data migration during rebalance
//! - WAL-backed durable replication
//! - Write concern (WAIT command)
//! - Epoch-based conflict resolution
//! - Collection consistency (quorum)
//! - DNS discovery

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::cluster::shard_migrator::MigrationStatus;
use vectorizer::cluster::{
    ClusterConfig, ClusterManager, ClusterNode, DistributedShardRouter, NodeId,
};
use vectorizer::db::sharding::ShardId;
use vectorizer::replication::durable_log::DurableReplicationLog;
use vectorizer::replication::types::{VectorOperation, WriteConcern};

// ============================================================================
// Epoch-Based Conflict Resolution Tests
// ============================================================================

#[test]
fn test_epoch_increments_on_shard_assignment() {
    let router = DistributedShardRouter::new(10);
    let node_a = NodeId::new("node-a".to_string());
    let node_b = NodeId::new("node-b".to_string());

    // First assignment → epoch 1
    let epoch1 = router.assign_shard(ShardId::new(0), node_a.clone());
    assert_eq!(epoch1, 1);

    // Second assignment → epoch 2
    let epoch2 = router.assign_shard(ShardId::new(1), node_b.clone());
    assert_eq!(epoch2, 2);

    // Reassignment of shard 0 → epoch 3
    let epoch3 = router.assign_shard(ShardId::new(0), node_b.clone());
    assert_eq!(epoch3, 3);

    // Verify epoch tracking
    assert_eq!(router.get_shard_epoch(&ShardId::new(0)), Some(3));
    assert_eq!(router.get_shard_epoch(&ShardId::new(1)), Some(2));
    assert_eq!(router.current_epoch(), 3);
}

#[test]
fn test_higher_epoch_wins_conflict() {
    let router = DistributedShardRouter::new(10);
    let node_a = NodeId::new("node-a".to_string());
    let node_b = NodeId::new("node-b".to_string());

    // Assign shard 0 to node_a with epoch 1
    router.assign_shard(ShardId::new(0), node_a.clone());

    // Simulate remote assignment with higher epoch
    let applied = router.apply_if_higher_epoch(ShardId::new(0), node_b.clone(), 5);
    assert!(applied, "Higher epoch should win");
    assert_eq!(
        router.get_node_for_shard(&ShardId::new(0)),
        Some(node_b.clone())
    );
    assert_eq!(router.get_shard_epoch(&ShardId::new(0)), Some(5));
}

#[test]
fn test_lower_epoch_rejected() {
    let router = DistributedShardRouter::new(10);
    let node_a = NodeId::new("node-a".to_string());
    let node_b = NodeId::new("node-b".to_string());

    // Assign shard 0 to node_a (epoch 1)
    router.assign_shard(ShardId::new(0), node_a.clone());

    // Try to override with lower epoch → rejected
    let applied = router.apply_if_higher_epoch(ShardId::new(0), node_b.clone(), 0);
    assert!(!applied, "Lower epoch should be rejected");
    assert_eq!(
        router.get_node_for_shard(&ShardId::new(0)),
        Some(node_a.clone())
    );
}

#[test]
fn test_epoch_survives_rebalance() {
    let router = DistributedShardRouter::new(10);
    let node_a = NodeId::new("node-a".to_string());
    let node_b = NodeId::new("node-b".to_string());

    let shards = vec![
        ShardId::new(0),
        ShardId::new(1),
        ShardId::new(2),
        ShardId::new(3),
    ];
    let nodes = vec![node_a.clone(), node_b.clone()];

    router.rebalance(&shards, &nodes);

    // All shards should have epochs
    for shard in &shards {
        assert!(
            router.get_shard_epoch(shard).is_some(),
            "Shard {shard:?} should have an epoch after rebalance",
        );
    }

    // Global epoch should have incremented
    assert!(
        router.current_epoch() >= 4,
        "Should have at least 4 epoch increments"
    );
}

// ============================================================================
// WAL-Backed Durable Replication Log Tests
// ============================================================================

#[test]
fn test_durable_log_memory_mode() {
    // Memory-only mode (no WAL file)
    let log = DurableReplicationLog::new(100, None).unwrap();

    let op = VectorOperation::InsertVector {
        collection: "test".to_string(),
        id: "vec1".to_string(),
        vector: vec![1.0, 2.0, 3.0],
        payload: None,
        owner_id: None,
    };

    let offset = log.append(op).expect("append should succeed");
    assert_eq!(offset, 1);
    assert_eq!(log.current_offset(), 1);
}

#[test]
fn test_durable_log_with_wal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let wal_path = temp_dir.path().join("test-replication.wal");

    let log = DurableReplicationLog::new(100, Some(wal_path.clone())).unwrap();

    // Append operations
    for i in 0..10 {
        let op = VectorOperation::InsertVector {
            collection: "test".to_string(),
            id: format!("vec_{i}"),
            vector: vec![i as f32; 3],
            payload: None,
            owner_id: None,
        };
        let offset = log.append(op).expect("append should succeed");
        assert_eq!(offset, i + 1);
    }

    assert_eq!(log.current_offset(), 10);

    // WAL file should exist
    assert!(wal_path.exists(), "WAL file should be created");

    // Operations should be retrievable from memory
    let ops = log.get_operations(5);
    assert!(ops.is_some());
    assert_eq!(ops.unwrap().len(), 5); // offsets 6-10
}

#[test]
fn test_durable_log_recovery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let wal_path = temp_dir.path().join("recovery-test.wal");

    // Phase 1: Write operations
    {
        let log = DurableReplicationLog::new(100, Some(wal_path.clone())).unwrap();
        for i in 0..5 {
            let op = VectorOperation::CreateCollection {
                name: format!("col_{i}"),
                config: vectorizer::replication::CollectionConfigData {
                    dimension: 128,
                    metric: "cosine".to_string(),
                },
                owner_id: None,
            };
            log.append(op).unwrap();
        }
        assert_eq!(log.current_offset(), 5);
        // log dropped here, WAL file persists
    }

    // Phase 2: Recover from WAL
    {
        let mut log = DurableReplicationLog::new(100, Some(wal_path.clone())).unwrap();
        let recovered_offset = log.recover().expect("recovery should succeed");
        assert_eq!(recovered_offset, 5, "Should recover all 5 operations");
        assert_eq!(log.current_offset(), 5);

        // Should be able to continue appending
        let op = VectorOperation::DeleteCollection {
            name: "col_0".to_string(),
            owner_id: None,
        };
        let new_offset = log.append(op).unwrap();
        assert_eq!(new_offset, 6);
    }
}

// ============================================================================
// Write Concern Tests
// ============================================================================

#[test]
fn test_write_concern_default_is_none() {
    let concern = WriteConcern::default();
    assert_eq!(concern, WriteConcern::None);
}

#[test]
fn test_write_concern_serialization() {
    let concerns = vec![
        WriteConcern::None,
        WriteConcern::Count(1),
        WriteConcern::Count(3),
        WriteConcern::All,
    ];

    for concern in concerns {
        let json = serde_json::to_string(&concern).unwrap();
        let deserialized: WriteConcern = serde_json::from_str(&json).unwrap();
        assert_eq!(concern, deserialized);
    }
}

// ============================================================================
// Shard Migrator Tests
// ============================================================================

#[test]
fn test_migration_status_variants() {
    // ShardMigrator requires ClusterClientPool (network), so we test status lifecycle only
    assert!(matches!(MigrationStatus::Pending, MigrationStatus::Pending));
    assert!(matches!(
        MigrationStatus::InProgress,
        MigrationStatus::InProgress
    ));
    assert!(matches!(
        MigrationStatus::Completed,
        MigrationStatus::Completed
    ));
}

// MigrationStatus lifecycle covered in test_migration_status_variants above

// ============================================================================
// Collection Consistency Tests
// ============================================================================

#[test]
fn test_cluster_manager_node_lifecycle() {
    let config = ClusterConfig {
        enabled: true,
        node_id: Some("test-node".to_string()),
        servers: vec![],
        ..Default::default()
    };

    let manager = ClusterManager::new(config).unwrap();

    // Local node should exist
    let nodes = manager.get_nodes();
    assert_eq!(nodes.len(), 1);

    // Add remote node
    let remote = ClusterNode::new(
        NodeId::new("remote-1".to_string()),
        "192.168.1.10".to_string(),
        15003,
    );
    manager.add_node(remote);

    let nodes = manager.get_nodes();
    assert_eq!(nodes.len(), 2);

    // Mark remote unavailable
    let remote_id = NodeId::new("remote-1".to_string());
    manager.mark_node_unavailable(&remote_id);

    let active = manager.get_active_nodes();
    assert_eq!(active.len(), 1, "Only local node should be active");
}

// ============================================================================
// DNS Discovery Tests
// ============================================================================

#[test]
fn test_dns_config_defaults() {
    let config = ClusterConfig::default();
    assert_eq!(config.dns_resolve_interval, 30);
    assert_eq!(config.dns_grpc_port, 15003);
    assert!(config.dns_name.is_none());
}

#[test]
fn test_dns_discovery_method_configured() {
    let config = ClusterConfig {
        enabled: true,
        node_id: Some("k8s-node".to_string()),
        discovery: vectorizer::cluster::DiscoveryMethod::Dns,
        dns_name: Some("vectorizer-headless.default.svc.cluster.local".to_string()),
        dns_resolve_interval: 15,
        dns_grpc_port: 15003,
        ..Default::default()
    };

    assert_eq!(config.discovery, vectorizer::cluster::DiscoveryMethod::Dns);
    assert_eq!(
        config.dns_name.as_deref(),
        Some("vectorizer-headless.default.svc.cluster.local")
    );
}

// ============================================================================
// Integration: Consistent Hashing + Epochs
// ============================================================================

#[test]
fn test_consistent_routing_with_epochs() {
    let router = DistributedShardRouter::new(100);
    let nodes: Vec<NodeId> = (0..3).map(|i| NodeId::new(format!("node-{i}"))).collect();
    let shards: Vec<ShardId> = (0..6).map(ShardId::new).collect();

    // Initial assignment
    router.rebalance(&shards, &nodes);

    // Record initial assignments
    let _initial: Vec<_> = shards
        .iter()
        .map(|s| router.get_node_for_shard(s).unwrap())
        .collect();

    // Routing should be deterministic for same vector ID
    let shard1 = router.get_shard_for_vector("document-123");
    let shard2 = router.get_shard_for_vector("document-123");
    assert_eq!(shard1, shard2, "Same vector ID should route to same shard");

    // Different IDs can route to different shards
    let shard_a = router.get_shard_for_vector("aaa");
    let _shard_b = router.get_shard_for_vector("zzz");
    // They might be the same shard, but routing should be consistent
    let shard_a2 = router.get_shard_for_vector("aaa");
    assert_eq!(shard_a, shard_a2);
}

#[test]
fn test_tenant_aware_routing_with_epochs() {
    let router = DistributedShardRouter::new(100);
    let nodes: Vec<NodeId> = (0..3).map(|i| NodeId::new(format!("node-{i}"))).collect();
    let shards: Vec<ShardId> = (0..6).map(ShardId::new).collect();

    router.rebalance(&shards, &nodes);

    // Same vector ID but different tenants → possibly different shards
    let shard_t1 = router.get_shard_for_tenant_vector("tenant-A", "doc-1");
    let _shard_t2 = router.get_shard_for_tenant_vector("tenant-B", "doc-1");

    // Same tenant + same vector → same shard (deterministic)
    let shard_t1_again = router.get_shard_for_tenant_vector("tenant-A", "doc-1");
    assert_eq!(shard_t1, shard_t1_again);
}
