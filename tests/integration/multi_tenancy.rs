//! Integration tests for multi-tenancy

use vectorizer::db::multi_tenancy::{
    MultiTenancyManager, TenantOperation, TenantQuotas, TenantUsageUpdate,
};
// Unused imports removed

#[test]
fn test_tenant_isolation() {
    let manager = MultiTenancyManager::new();

    // Register two tenants
    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);

    // Associate collections with different tenants
    manager
        .associate_collection("collection1", &"tenant1".to_string())
        .unwrap();
    manager
        .associate_collection("collection2", &"tenant2".to_string())
        .unwrap();

    // Tenant1 should only access collection1
    manager
        .can_access_collection(&"tenant1".to_string(), "collection1")
        .unwrap();
    assert!(
        manager
            .can_access_collection(&"tenant1".to_string(), "collection2")
            .is_err()
    );

    // Tenant2 should only access collection2
    manager
        .can_access_collection(&"tenant2".to_string(), "collection2")
        .unwrap();
    assert!(
        manager
            .can_access_collection(&"tenant2".to_string(), "collection1")
            .is_err()
    );
}

#[test]
fn test_resource_quotas() {
    let manager = MultiTenancyManager::new();

    // Create tenant with limited quotas
    let quotas = TenantQuotas {
        max_collections: Some(2),
        max_vectors_per_collection: Some(1000),
        max_memory_bytes: Some(100_000_000), // 100 MB
        max_qps: Some(10),
        max_storage_bytes: Some(1_000_000_000), // 1 GB
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should be able to create first collection
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
        .unwrap();
    manager
        .associate_collection("collection1", &"tenant1".to_string())
        .unwrap();

    // Should be able to create second collection
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
        .unwrap();
    manager
        .associate_collection("collection2", &"tenant1".to_string())
        .unwrap();

    // Should fail on third collection
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .is_err()
    );
}

#[test]
fn test_memory_quota() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_memory_bytes: Some(1000),
        ..Default::default()
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should be able to use memory within limit
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseMemory(500))
        .unwrap();
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(500));

    // Should fail when exceeding limit
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseMemory(600))
            .is_err()
    );
}

#[test]
fn test_qps_quota() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_qps: Some(5),
        ..Default::default()
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should be able to perform queries within QPS limit
    for _ in 0..5 {
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::PerformQuery)
            .unwrap();
    }

    // Note: QPS is calculated per second, so this test is simplified
    // In real usage, QPS would be tracked over time windows
}

#[test]
fn test_vector_count_quota() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_vectors_per_collection: Some(100),
        ..Default::default()
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should be able to add vectors within limit
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::AddVectors(50))
        .unwrap();
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(50));

    // Should fail when exceeding limit
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::AddVectors(60))
            .is_err()
    );
}

#[test]
fn test_tenant_usage_tracking() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    // Update usage
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(1000));
    manager.update_usage(
        &"tenant1".to_string(),
        TenantUsageUpdate::CollectionCount(1),
    );
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(100));

    // Get stats
    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.memory_bytes, 1000);
    assert_eq!(stats.collection_count, 1);
    assert_eq!(stats.total_vectors, 100);
}

#[test]
fn test_collection_removal() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    manager
        .associate_collection("collection1", &"tenant1".to_string())
        .unwrap();

    let stats_before = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats_before.collection_count, 1);

    manager.remove_collection("collection1");

    let stats_after = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats_after.collection_count, 0);
}

#[test]
fn test_list_tenants() {
    let manager = MultiTenancyManager::new();

    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);
    manager.register_tenant("tenant3".to_string(), None);

    let tenants = manager.list_tenants();
    assert_eq!(tenants.len(), 3);
    assert!(tenants.contains(&"tenant1".to_string()));
    assert!(tenants.contains(&"tenant2".to_string()));
    assert!(tenants.contains(&"tenant3".to_string()));
}

#[test]
fn test_tenant_not_found() {
    let manager = MultiTenancyManager::new();

    // Try to access non-existent tenant
    assert!(manager.get_tenant(&"nonexistent".to_string()).is_none());
    assert!(
        manager
            .check_tenant_quota(
                &"nonexistent".to_string(),
                &TenantOperation::CreateCollection
            )
            .is_err()
    );
}

#[test]
fn test_collection_tenant_mapping() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);

    manager
        .associate_collection("collection1", &"tenant1".to_string())
        .unwrap();
    manager
        .associate_collection("collection2", &"tenant2".to_string())
        .unwrap();

    assert_eq!(
        manager.get_collection_tenant("collection1"),
        Some("tenant1".to_string())
    );
    assert_eq!(
        manager.get_collection_tenant("collection2"),
        Some("tenant2".to_string())
    );
    assert_eq!(manager.get_collection_tenant("nonexistent"), None);
}
