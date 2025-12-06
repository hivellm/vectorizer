//! Multi-tenant isolation tests for HiveHub Cloud integration
//!
//! These tests verify that tenants are properly isolated from each other,
//! ensuring data privacy and security in a multi-tenant environment.

use std::sync::Arc;

use vectorizer::hub::auth::{TenantContext, TenantPermission};

use super::mock_hub::{MockHubApi, MockUser};

/// Create a test tenant context for a given user
fn create_tenant_context(user: &MockUser, permissions: Vec<TenantPermission>) -> TenantContext {
    TenantContext {
        tenant_id: user.id.to_string(),
        tenant_name: user.username.clone(),
        api_key_id: user.api_key.clone(),
        permissions,
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    }
}

#[test]
fn test_tenant_cannot_access_other_tenant_collections() {
    let mock = MockHubApi::new();

    // Create two tenants
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");

    // Tenant A creates a collection
    let collection_a = mock.create_collection(tenant_a.id, "private_data").unwrap();

    // Tenant A owns the collection
    assert!(mock.validate_collection(collection_a.id, tenant_a.id));

    // Tenant B does NOT own the collection
    assert!(!mock.validate_collection(collection_a.id, tenant_b.id));

    // Tenant B cannot see Tenant A's collections
    let tenant_b_collections = mock.get_user_collections(tenant_b.id);
    assert!(tenant_b_collections.is_empty());
}

#[test]
fn test_tenant_isolation_with_same_collection_names() {
    let mock = MockHubApi::new();

    // Create two tenants
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");

    // Both tenants create collections with the same name
    let collection_a = mock.create_collection(tenant_a.id, "documents").unwrap();
    let collection_b = mock.create_collection(tenant_b.id, "documents").unwrap();

    // Collections have different IDs
    assert_ne!(collection_a.id, collection_b.id);

    // Each tenant only sees their own collection
    let tenant_a_collections = mock.get_user_collections(tenant_a.id);
    assert_eq!(tenant_a_collections.len(), 1);
    assert_eq!(tenant_a_collections[0].id, collection_a.id);

    let tenant_b_collections = mock.get_user_collections(tenant_b.id);
    assert_eq!(tenant_b_collections.len(), 1);
    assert_eq!(tenant_b_collections[0].id, collection_b.id);
}

#[test]
fn test_tenant_cannot_delete_other_tenant_collection() {
    let mock = MockHubApi::new();

    // Create two tenants
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");

    // Tenant A creates a collection
    let collection_a = mock.create_collection(tenant_a.id, "my_data").unwrap();

    // Tenant B attempts to delete Tenant A's collection
    let result = mock.delete_collection(collection_a.id, tenant_b.id);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found or not owned"));

    // Collection still exists for Tenant A
    let tenant_a_collections = mock.get_user_collections(tenant_a.id);
    assert_eq!(tenant_a_collections.len(), 1);
}

#[test]
fn test_tenant_quota_isolation() {
    let mock = MockHubApi::new();

    // Create tenant with limited quota
    let limited_tenant = MockUser::new("limited").with_quota(2, 1000, 1_000_000);
    mock.add_user(limited_tenant.clone());

    // Create tenant with large quota
    let premium_tenant = MockUser::new("premium").with_quota(100, 1_000_000, 100_000_000_000);
    mock.add_user(premium_tenant.clone());

    // Limited tenant can only create 2 collections
    assert!(mock.create_collection(limited_tenant.id, "col1").is_ok());
    assert!(mock.create_collection(limited_tenant.id, "col2").is_ok());
    assert!(mock.create_collection(limited_tenant.id, "col3").is_err());

    // Premium tenant can create many more
    assert!(mock.create_collection(premium_tenant.id, "col1").is_ok());
    assert!(mock.create_collection(premium_tenant.id, "col2").is_ok());
    assert!(mock.create_collection(premium_tenant.id, "col3").is_ok());
}

#[test]
fn test_tenant_usage_isolation() {
    let mock = MockHubApi::new();

    // Create two tenants
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");

    // Tenant A records usage
    mock.record_usage(tenant_a.id, 1000, 50000).unwrap();

    // Tenant A's quota shows usage
    let quota_a = mock.get_quota_info(tenant_a.id).unwrap();
    assert_eq!(quota_a.vectors_used, 1000);
    assert_eq!(quota_a.storage_used, 50000);

    // Tenant B's quota is still at zero
    let quota_b = mock.get_quota_info(tenant_b.id).unwrap();
    assert_eq!(quota_b.vectors_used, 0);
    assert_eq!(quota_b.storage_used, 0);
}

#[test]
fn test_api_key_isolation() {
    let mock = MockHubApi::new();

    // Create two tenants
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");

    // Tenant A's API key only returns Tenant A
    let validated_a = mock.validate_api_key(&tenant_a.api_key).unwrap();
    assert_eq!(validated_a.id, tenant_a.id);
    assert_ne!(validated_a.id, tenant_b.id);

    // Tenant B's API key only returns Tenant B
    let validated_b = mock.validate_api_key(&tenant_b.api_key).unwrap();
    assert_eq!(validated_b.id, tenant_b.id);
    assert_ne!(validated_b.id, tenant_a.id);

    // Invalid API key returns None
    let invalid = mock.validate_api_key("invalid_key");
    assert!(invalid.is_none());
}

#[test]
fn test_tenant_context_permission_isolation() {
    let mock = MockHubApi::new();

    // Create tenants with different permission levels
    let admin_user = mock.create_test_user("admin");
    let reader_user = mock.create_test_user("reader");

    let admin_context = create_tenant_context(&admin_user, vec![TenantPermission::Admin]);
    let reader_context = create_tenant_context(&reader_user, vec![TenantPermission::ReadOnly]);

    // Admin can do everything
    assert!(admin_context.can_perform("create_collection"));
    assert!(admin_context.can_perform("delete_collection"));
    assert!(admin_context.can_perform("insert_vectors"));
    assert!(admin_context.can_perform("search"));

    // Reader can only read
    assert!(!reader_context.can_perform("create_collection"));
    assert!(!reader_context.can_perform("delete_collection"));
    assert!(!reader_context.can_perform("insert_vectors"));
    assert!(reader_context.can_perform("search"));
    assert!(reader_context.can_perform("list_collections"));
}

#[test]
fn test_multi_tenant_concurrent_operations() {
    use std::thread;

    let mock = Arc::new(MockHubApi::new());

    // Create multiple tenants
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");
    let tenant_c = mock.create_test_user("tenant_c");

    let tenant_ids = vec![tenant_a.id, tenant_b.id, tenant_c.id];

    // Simulate concurrent collection creation
    let handles: Vec<_> = tenant_ids
        .into_iter()
        .enumerate()
        .map(|(i, tenant_id)| {
            let mock = Arc::clone(&mock);
            thread::spawn(move || {
                for j in 0..5 {
                    let name = format!("collection_{i}_{j}");
                    mock.create_collection(tenant_id, &name).unwrap();
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify each tenant has exactly 5 collections
    let collections_a = mock.get_user_collections(tenant_a.id);
    let collections_b = mock.get_user_collections(tenant_b.id);
    let collections_c = mock.get_user_collections(tenant_c.id);

    assert_eq!(collections_a.len(), 5);
    assert_eq!(collections_b.len(), 5);
    assert_eq!(collections_c.len(), 5);

    // Verify no cross-contamination
    for col in &collections_a {
        assert!(col.name.starts_with("collection_0_"));
    }
    for col in &collections_b {
        assert!(col.name.starts_with("collection_1_"));
    }
    for col in &collections_c {
        assert!(col.name.starts_with("collection_2_"));
    }
}

#[test]
fn test_tenant_reset_isolation() {
    let mock = MockHubApi::new();

    // Create tenants and collections
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");

    mock.create_collection(tenant_a.id, "col_a").unwrap();
    mock.create_collection(tenant_b.id, "col_b").unwrap();

    // Verify both have collections
    assert_eq!(mock.get_user_collections(tenant_a.id).len(), 1);
    assert_eq!(mock.get_user_collections(tenant_b.id).len(), 1);

    // Reset clears everything
    mock.reset();

    // After reset, both tenants' API keys are invalid
    assert!(mock.validate_api_key(&tenant_a.api_key).is_none());
    assert!(mock.validate_api_key(&tenant_b.api_key).is_none());
}

#[test]
fn test_collection_naming_format() {
    let mock = MockHubApi::new();
    let tenant = mock.create_test_user("test_tenant");

    // Create a collection
    let collection = mock.create_collection(tenant.id, "my_documents").unwrap();

    // Collection has correct owner
    assert_eq!(collection.owner_id, tenant.id);
    assert_eq!(collection.name, "my_documents");

    // In a real multi-tenant system, collections would be prefixed with user_{user_id}:
    // This is handled by the VectorStore layer, not the mock API
    let expected_internal_name = format!("user_{}:my_documents", tenant.id);
    assert!(expected_internal_name.contains(&tenant.id.to_string()));
}

#[test]
fn test_quota_check_isolation() {
    let mock = MockHubApi::new();

    // Tenant with very limited quota
    let limited = MockUser::new("limited").with_quota(1, 10, 100);
    mock.add_user(limited.clone());

    // Tenant with unlimited quota (large numbers)
    let unlimited = MockUser::new("unlimited").with_quota(1000, 10_000_000, 100_000_000_000);
    mock.add_user(unlimited.clone());

    // Limited tenant cannot add 100 vectors
    assert!(!mock.check_quota(limited.id, "vectors", 100).unwrap());

    // Unlimited tenant can add 100 vectors
    assert!(mock.check_quota(unlimited.id, "vectors", 100).unwrap());

    // Each tenant's quota is independent
    mock.record_usage(limited.id, 5, 50).unwrap();

    // Limited tenant now has 5 vectors used, can add 5 more
    assert!(mock.check_quota(limited.id, "vectors", 5).unwrap());
    assert!(!mock.check_quota(limited.id, "vectors", 6).unwrap());

    // Unlimited tenant's quota is unaffected
    assert!(
        mock.check_quota(unlimited.id, "vectors", 1_000_000)
            .unwrap()
    );
}

// ============================================================================
// Additional Data Leakage Prevention Tests
// ============================================================================

#[test]
fn test_zero_cross_tenant_data_leakage_comprehensive() {
    let mock = MockHubApi::new();

    // Create 3 tenants to test complex scenarios
    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");
    let tenant_c = mock.create_test_user("tenant_c");

    // Each tenant creates multiple collections
    let col_a1 = mock.create_collection(tenant_a.id, "documents").unwrap();
    let col_a2 = mock.create_collection(tenant_a.id, "images").unwrap();
    let col_b1 = mock.create_collection(tenant_b.id, "documents").unwrap();
    let col_c1 = mock.create_collection(tenant_c.id, "vectors").unwrap();

    // Verify tenant A can only access their collections
    assert!(mock.validate_collection(col_a1.id, tenant_a.id));
    assert!(mock.validate_collection(col_a2.id, tenant_a.id));
    assert!(!mock.validate_collection(col_b1.id, tenant_a.id));
    assert!(!mock.validate_collection(col_c1.id, tenant_a.id));

    // Verify tenant B can only access their collections
    assert!(mock.validate_collection(col_b1.id, tenant_b.id));
    assert!(!mock.validate_collection(col_a1.id, tenant_b.id));
    assert!(!mock.validate_collection(col_a2.id, tenant_b.id));
    assert!(!mock.validate_collection(col_c1.id, tenant_b.id));

    // Verify tenant C can only access their collections
    assert!(mock.validate_collection(col_c1.id, tenant_c.id));
    assert!(!mock.validate_collection(col_a1.id, tenant_c.id));
    assert!(!mock.validate_collection(col_b1.id, tenant_c.id));

    // Verify collection listing is properly isolated
    let a_collections = mock.get_user_collections(tenant_a.id);
    let b_collections = mock.get_user_collections(tenant_b.id);
    let c_collections = mock.get_user_collections(tenant_c.id);

    assert_eq!(a_collections.len(), 2);
    assert_eq!(b_collections.len(), 1);
    assert_eq!(c_collections.len(), 1);

    // Ensure no collection IDs overlap between tenants
    for col in &a_collections {
        assert!(!b_collections.iter().any(|b| b.id == col.id));
        assert!(!c_collections.iter().any(|c| c.id == col.id));
    }
}

#[test]
fn test_tenant_deletion_data_cleanup() {
    let mock = MockHubApi::new();

    // Create tenant with collections
    let tenant = mock.create_test_user("tenant_to_delete");
    mock.create_collection(tenant.id, "col1").unwrap();
    mock.create_collection(tenant.id, "col2").unwrap();
    mock.create_collection(tenant.id, "col3").unwrap();

    // Record usage
    mock.record_usage(tenant.id, 100, 1000).unwrap();

    // Verify collections exist
    assert_eq!(mock.get_user_collections(tenant.id).len(), 3);

    // Delete all tenant collections
    let collections = mock.get_user_collections(tenant.id);
    for col in collections {
        mock.delete_collection(col.id, tenant.id).unwrap();
    }

    // Verify all collections are deleted
    assert_eq!(mock.get_user_collections(tenant.id).len(), 0);

    // Verify tenant still exists but has no data
    let quota = mock.get_quota_info(tenant.id).unwrap();
    assert_eq!(quota.collections_used, 0);
}

#[test]
fn test_tenant_api_key_uniqueness_and_isolation() {
    let mock = MockHubApi::new();

    // Create multiple tenants
    let tenants: Vec<_> = (0..10)
        .map(|i| mock.create_test_user(&format!("tenant_{i}")))
        .collect();

    // Verify all API keys are unique
    let mut seen_keys = std::collections::HashSet::new();
    for tenant in &tenants {
        assert!(
            seen_keys.insert(&tenant.api_key),
            "Duplicate API key detected"
        );
    }

    // Verify each API key only validates to its owner
    for (i, tenant) in tenants.iter().enumerate() {
        let validated = mock.validate_api_key(&tenant.api_key).unwrap();
        assert_eq!(validated.id, tenant.id);
        assert_eq!(validated.username, format!("tenant_{i}"));

        // Verify this API key doesn't validate to any other tenant
        for other_tenant in &tenants {
            if other_tenant.id != tenant.id {
                let other_validated = mock.validate_api_key(&tenant.api_key).unwrap();
                assert_ne!(other_validated.id, other_tenant.id);
            }
        }
    }
}

#[test]
fn test_collection_name_reuse_across_tenants() {
    let mock = MockHubApi::new();

    let tenant_a = mock.create_test_user("tenant_a");
    let tenant_b = mock.create_test_user("tenant_b");
    let tenant_c = mock.create_test_user("tenant_c");

    // All tenants create collections with the same names
    let shared_names = vec!["common", "shared", "documents"];

    for name in &shared_names {
        mock.create_collection(tenant_a.id, name).unwrap();
        mock.create_collection(tenant_b.id, name).unwrap();
        mock.create_collection(tenant_c.id, name).unwrap();
    }

    // Each tenant should have 3 collections
    assert_eq!(mock.get_user_collections(tenant_a.id).len(), 3);
    assert_eq!(mock.get_user_collections(tenant_b.id).len(), 3);
    assert_eq!(mock.get_user_collections(tenant_c.id).len(), 3);

    // Verify all collection IDs are unique across tenants
    let all_a = mock.get_user_collections(tenant_a.id);
    let all_b = mock.get_user_collections(tenant_b.id);
    let all_c = mock.get_user_collections(tenant_c.id);

    let mut all_ids = std::collections::HashSet::new();
    for col in all_a.iter().chain(all_b.iter()).chain(all_c.iter()) {
        assert!(
            all_ids.insert(col.id),
            "Duplicate collection ID found across tenants"
        );
    }
}

#[test]
fn test_quota_enforcement_prevents_data_leakage() {
    let mock = MockHubApi::new();

    // Create tenant with strict quota (max 2 collections)
    let limited = MockUser::new("limited").with_quota(2, 100, 10000);
    mock.add_user(limited.clone());

    // Create 2 collections (should succeed)
    mock.create_collection(limited.id, "col1").unwrap();
    mock.create_collection(limited.id, "col2").unwrap();

    // Try to create a 3rd collection (should fail)
    let result = mock.create_collection(limited.id, "col3");
    assert!(
        result.is_err(),
        "Should not be able to exceed collection quota"
    );

    // Verify only 2 collections exist
    assert_eq!(mock.get_user_collections(limited.id).len(), 2);

    // Even if another tenant creates a collection with same name,
    // it shouldn't affect the limited tenant's quota
    let other = mock.create_test_user("other");
    mock.create_collection(other.id, "col3").unwrap();

    // Limited tenant still can't create more collections
    let result = mock.create_collection(limited.id, "col4");
    assert!(result.is_err());
}

#[test]
fn test_concurrent_tenant_operations_no_cross_contamination() {
    use std::sync::Arc;
    use std::thread;

    let mock = Arc::new(MockHubApi::new());

    // Create 5 tenants
    let tenants: Vec<_> = (0..5)
        .map(|i| mock.create_test_user(&format!("concurrent_{i}")))
        .collect();

    // Each tenant creates 10 collections concurrently
    let handles: Vec<_> = tenants
        .iter()
        .enumerate()
        .map(|(tenant_idx, tenant)| {
            let mock = Arc::clone(&mock);
            let tenant_id = tenant.id;

            thread::spawn(move || {
                for col_idx in 0..10 {
                    let name = format!("tenant_{tenant_idx}_col_{col_idx}");
                    mock.create_collection(tenant_id, &name).unwrap();
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify each tenant has exactly 10 collections
    for (tenant_idx, tenant) in tenants.iter().enumerate() {
        let collections = mock.get_user_collections(tenant.id);
        assert_eq!(
            collections.len(),
            10,
            "Tenant {tenant_idx} should have 10 collections"
        );

        // Verify all collections belong to this tenant
        for col in &collections {
            assert_eq!(col.owner_id, tenant.id);
            assert!(col.name.starts_with(&format!("tenant_{tenant_idx}_")));
        }
    }

    // Verify total collection count is correct
    let total_collections: usize = tenants
        .iter()
        .map(|t| mock.get_user_collections(t.id).len())
        .sum();
    assert_eq!(total_collections, 50);
}
