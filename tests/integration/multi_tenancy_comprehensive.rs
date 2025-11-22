//! Comprehensive integration tests for multi-tenancy
//!
//! Tests cover:
//! - Tenant isolation and access control
//! - Resource quotas and enforcement
//! - Usage tracking and monitoring
//! - Collection-tenant associations
//! - Concurrent tenant operations
//! - Edge cases and error handling

use vectorizer::db::multi_tenancy::{
    MultiTenancyManager, TenantOperation, TenantQuotas, TenantUsageUpdate,
};
// Unused imports removed

// ============================================================================
// Tenant Registration and Management
// ============================================================================

#[test]
fn test_tenant_registration() {
    let manager = MultiTenancyManager::new();

    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);

    assert!(manager.get_tenant(&"tenant1".to_string()).is_some());
    assert!(manager.get_tenant(&"tenant2".to_string()).is_some());
    assert!(manager.get_tenant(&"tenant3".to_string()).is_none());
}

#[test]
fn test_tenant_registration_with_quotas() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_collections: Some(5),
        max_vectors_per_collection: Some(10000),
        max_memory_bytes: Some(500_000_000),
        max_qps: Some(50),
        max_storage_bytes: Some(5_000_000_000),
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas.clone()));

    let tenant = manager.get_tenant(&"tenant1".to_string()).unwrap();
    assert_eq!(tenant.quotas.max_collections, quotas.max_collections);
    assert_eq!(tenant.quotas.max_memory_bytes, quotas.max_memory_bytes);
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

// ============================================================================
// Tenant Isolation Tests
// ============================================================================

#[test]
fn test_tenant_isolation_basic() {
    let manager = MultiTenancyManager::new();

    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);

    // Associate collections
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
fn test_tenant_isolation_multiple_collections() {
    let manager = MultiTenancyManager::new();

    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);

    // Tenant1 has multiple collections
    manager
        .associate_collection("coll1", &"tenant1".to_string())
        .unwrap();
    manager
        .associate_collection("coll2", &"tenant1".to_string())
        .unwrap();
    manager
        .associate_collection("coll3", &"tenant1".to_string())
        .unwrap();

    // Tenant2 has different collections
    manager
        .associate_collection("coll4", &"tenant2".to_string())
        .unwrap();
    manager
        .associate_collection("coll5", &"tenant2".to_string())
        .unwrap();

    // Tenant1 can access its collections
    manager
        .can_access_collection(&"tenant1".to_string(), "coll1")
        .unwrap();
    manager
        .can_access_collection(&"tenant1".to_string(), "coll2")
        .unwrap();
    manager
        .can_access_collection(&"tenant1".to_string(), "coll3")
        .unwrap();

    // Tenant1 cannot access tenant2's collections
    assert!(
        manager
            .can_access_collection(&"tenant1".to_string(), "coll4")
            .is_err()
    );
    assert!(
        manager
            .can_access_collection(&"tenant1".to_string(), "coll5")
            .is_err()
    );

    // Tenant2 can access its collections
    manager
        .can_access_collection(&"tenant2".to_string(), "coll4")
        .unwrap();
    manager
        .can_access_collection(&"tenant2".to_string(), "coll5")
        .unwrap();

    // Tenant2 cannot access tenant1's collections
    assert!(
        manager
            .can_access_collection(&"tenant2".to_string(), "coll1")
            .is_err()
    );
    assert!(
        manager
            .can_access_collection(&"tenant2".to_string(), "coll2")
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

// ============================================================================
// Resource Quota Tests
// ============================================================================

#[test]
fn test_collection_count_quota() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_collections: Some(2),
        ..Default::default()
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
fn test_storage_quota() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_storage_bytes: Some(5000),
        ..Default::default()
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should be able to use storage within limit
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseStorage(2000))
        .unwrap();
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Storage(2000));

    // Should fail when exceeding limit
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseStorage(4000))
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
fn test_multiple_quotas_enforcement() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_collections: Some(2),
        max_vectors_per_collection: Some(100),
        max_memory_bytes: Some(1000),
        max_qps: Some(10),
        max_storage_bytes: Some(5000),
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Test collection quota
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
        .unwrap();
    manager
        .associate_collection("coll1", &"tenant1".to_string())
        .unwrap();

    // Test memory quota
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseMemory(500))
        .unwrap();
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(500));

    // Test vector quota
    manager
        .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::AddVectors(50))
        .unwrap();
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(50));

    // All quotas should be enforced
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .is_ok()
    );
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseMemory(600))
            .is_err()
    );
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::AddVectors(60))
            .is_err()
    );
}

// ============================================================================
// Usage Tracking Tests
// ============================================================================

#[test]
fn test_tenant_usage_tracking() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    // Update various usage metrics
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(1000));
    manager.update_usage(
        &"tenant1".to_string(),
        TenantUsageUpdate::CollectionCount(1),
    );
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(100));
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Storage(5000));

    // Get stats
    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.memory_bytes, 1000);
    assert_eq!(stats.collection_count, 1);
    assert_eq!(stats.total_vectors, 100);
    assert_eq!(stats.storage_bytes, 5000);
}

#[test]
fn test_tenant_usage_accumulation() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    // Add memory incrementally
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(100));
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(200));
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::Memory(300));

    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.memory_bytes, 600);
}

#[test]
fn test_tenant_usage_vector_count() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    // Add vectors incrementally
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(50));
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(75));
    manager.update_usage(&"tenant1".to_string(), TenantUsageUpdate::VectorCount(25));

    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.total_vectors, 150);
}

// ============================================================================
// Collection Management Tests
// ============================================================================

#[test]
fn test_collection_association() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    manager
        .associate_collection("collection1", &"tenant1".to_string())
        .unwrap();

    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.collection_count, 1);
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
    assert_eq!(manager.get_collection_tenant("collection1"), None);
}

#[test]
fn test_multiple_collections_per_tenant() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    manager
        .associate_collection("coll1", &"tenant1".to_string())
        .unwrap();
    manager
        .associate_collection("coll2", &"tenant1".to_string())
        .unwrap();
    manager
        .associate_collection("coll3", &"tenant1".to_string())
        .unwrap();

    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.collection_count, 3);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

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
    assert!(
        manager
            .get_tenant_stats(&"nonexistent".to_string())
            .is_none()
    );
}

#[test]
fn test_collection_not_associated() {
    let manager = MultiTenancyManager::new();
    manager.register_tenant("tenant1".to_string(), None);

    // Try to access collection that doesn't exist
    assert_eq!(manager.get_collection_tenant("nonexistent"), None);
    // If collection is not associated, can_access_collection returns Ok (no restriction)
    assert!(
        manager
            .can_access_collection(&"tenant1".to_string(), "nonexistent")
            .is_ok()
    );
}

#[test]
fn test_quota_exceeded_error() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_collections: Some(1),
        ..Default::default()
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));
    manager
        .associate_collection("coll1", &"tenant1".to_string())
        .unwrap();

    // Should fail when quota exceeded
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .is_err()
    );
}

// ============================================================================
// Concurrent Operations Tests
// ============================================================================

#[test]
fn test_concurrent_tenant_operations() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(MultiTenancyManager::new());
    manager.register_tenant("tenant1".to_string(), None);
    manager.register_tenant("tenant2".to_string(), None);

    let mut handles = Vec::new();

    // Spawn threads for tenant1
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            let coll_name = format!("tenant1_coll_{i}");
            mgr.associate_collection(&coll_name, &"tenant1".to_string())
                .unwrap();
        });
        handles.push(handle);
    }

    // Spawn threads for tenant2
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            let coll_name = format!("tenant2_coll_{i}");
            mgr.associate_collection(&coll_name, &"tenant2".to_string())
                .unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify results
    let stats1 = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    let stats2 = manager.get_tenant_stats(&"tenant2".to_string()).unwrap();

    assert_eq!(stats1.collection_count, 5);
    assert_eq!(stats2.collection_count, 5);
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_unlimited_quotas() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_collections: None,
        max_vectors_per_collection: None,
        max_memory_bytes: None,
        max_qps: None,
        max_storage_bytes: None,
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should be able to create many collections
    for i in 0..100 {
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .unwrap();
        manager
            .associate_collection(&format!("coll_{i}"), &"tenant1".to_string())
            .unwrap();
    }

    let stats = manager.get_tenant_stats(&"tenant1".to_string()).unwrap();
    assert_eq!(stats.collection_count, 100);
}

#[test]
fn test_zero_quotas() {
    let manager = MultiTenancyManager::new();

    let quotas = TenantQuotas {
        max_collections: Some(0),
        max_vectors_per_collection: Some(0),
        max_memory_bytes: Some(0),
        max_qps: Some(0),
        max_storage_bytes: Some(0),
    };

    manager.register_tenant("tenant1".to_string(), Some(quotas));

    // Should fail all operations
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .is_err()
    );
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::UseMemory(1))
            .is_err()
    );
    assert!(
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::AddVectors(1))
            .is_err()
    );
}
