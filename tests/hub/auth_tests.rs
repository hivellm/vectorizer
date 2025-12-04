//! Authentication tests for HiveHub integration

use vectorizer::hub::auth::{TenantContext, TenantPermission};
use vectorizer::hub::{HubCacheConfig, HubConfig};

#[test]
fn test_tenant_permission_admin_allows_all() {
    assert!(TenantPermission::Admin.allows("create_collection"));
    assert!(TenantPermission::Admin.allows("delete_collection"));
    assert!(TenantPermission::Admin.allows("list_collections"));
    assert!(TenantPermission::Admin.allows("insert_vectors"));
    assert!(TenantPermission::Admin.allows("update_vectors"));
    assert!(TenantPermission::Admin.allows("delete_vectors"));
    assert!(TenantPermission::Admin.allows("search"));
    assert!(TenantPermission::Admin.allows("admin_only_operation"));
}

#[test]
fn test_tenant_permission_read_write() {
    assert!(TenantPermission::ReadWrite.allows("create_collection"));
    assert!(TenantPermission::ReadWrite.allows("delete_collection"));
    assert!(TenantPermission::ReadWrite.allows("list_collections"));
    assert!(TenantPermission::ReadWrite.allows("insert_vectors"));
    assert!(TenantPermission::ReadWrite.allows("update_vectors"));
    assert!(TenantPermission::ReadWrite.allows("delete_vectors"));
    assert!(TenantPermission::ReadWrite.allows("search"));
    assert!(!TenantPermission::ReadWrite.allows("admin_only_operation"));
}

#[test]
fn test_tenant_permission_read_only() {
    assert!(!TenantPermission::ReadOnly.allows("create_collection"));
    assert!(!TenantPermission::ReadOnly.allows("delete_collection"));
    assert!(TenantPermission::ReadOnly.allows("list_collections"));
    assert!(!TenantPermission::ReadOnly.allows("insert_vectors"));
    assert!(!TenantPermission::ReadOnly.allows("update_vectors"));
    assert!(!TenantPermission::ReadOnly.allows("delete_vectors"));
    assert!(TenantPermission::ReadOnly.allows("search"));
    assert!(!TenantPermission::ReadOnly.allows("admin_only_operation"));
}

#[test]
fn test_tenant_permission_mcp() {
    // MCP can do limited operations
    assert!(!TenantPermission::Mcp.allows("create_collection"));
    assert!(!TenantPermission::Mcp.allows("delete_collection"));
    assert!(TenantPermission::Mcp.allows("list_collections"));
    assert!(TenantPermission::Mcp.allows("insert_vectors"));
    assert!(TenantPermission::Mcp.allows("update_vectors"));
    assert!(!TenantPermission::Mcp.allows("delete_vectors")); // MCP cannot delete
    assert!(TenantPermission::Mcp.allows("search"));
}

#[test]
fn test_tenant_context_has_permission() {
    let context = TenantContext {
        tenant_id: "tenant_123".to_string(),
        tenant_name: "Test Tenant".to_string(),
        api_key_id: "key_abc".to_string(),
        permissions: vec![TenantPermission::ReadWrite],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };

    // Has direct permission
    assert!(context.has_permission(TenantPermission::ReadWrite));

    // Does not have other permissions
    assert!(!context.has_permission(TenantPermission::Admin));
    assert!(!context.has_permission(TenantPermission::ReadOnly));
}

#[test]
fn test_tenant_context_admin_has_all_permissions() {
    let context = TenantContext {
        tenant_id: "tenant_admin".to_string(),
        tenant_name: "Admin Tenant".to_string(),
        api_key_id: "key_admin".to_string(),
        permissions: vec![TenantPermission::Admin],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: false,
    };

    // Admin implies all permissions
    assert!(context.has_permission(TenantPermission::Admin));
    assert!(context.has_permission(TenantPermission::ReadWrite));
    assert!(context.has_permission(TenantPermission::ReadOnly));
    assert!(context.has_permission(TenantPermission::Mcp));
}

#[test]
fn test_tenant_context_can_perform() {
    let context = TenantContext {
        tenant_id: "tenant_rw".to_string(),
        tenant_name: "Read-Write Tenant".to_string(),
        api_key_id: "key_rw".to_string(),
        permissions: vec![TenantPermission::ReadWrite],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };

    assert!(context.can_perform("search"));
    assert!(context.can_perform("insert_vectors"));
    assert!(context.can_perform("delete_vectors"));
}

#[test]
fn test_tenant_context_highest_permission() {
    // Admin is highest
    let admin_context = TenantContext {
        tenant_id: "t1".to_string(),
        tenant_name: "Admin".to_string(),
        api_key_id: "k1".to_string(),
        permissions: vec![TenantPermission::Admin, TenantPermission::ReadWrite],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };
    assert_eq!(admin_context.highest_permission(), TenantPermission::Admin);

    // ReadWrite is higher than ReadOnly and MCP
    let rw_context = TenantContext {
        tenant_id: "t2".to_string(),
        tenant_name: "RW".to_string(),
        api_key_id: "k2".to_string(),
        permissions: vec![TenantPermission::ReadWrite, TenantPermission::ReadOnly],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };
    assert_eq!(rw_context.highest_permission(), TenantPermission::ReadWrite);

    // ReadOnly when that's all there is
    let ro_context = TenantContext {
        tenant_id: "t3".to_string(),
        tenant_name: "RO".to_string(),
        api_key_id: "k3".to_string(),
        permissions: vec![TenantPermission::ReadOnly],
        rate_limits: None,
        validated_at: chrono::Utc::now(),
        is_test: true,
    };
    assert_eq!(ro_context.highest_permission(), TenantPermission::ReadOnly);
}

#[test]
fn test_hub_config_default() {
    let config = HubConfig::default();

    assert!(!config.enabled);
    assert_eq!(config.api_url, "https://api.hivehub.cloud");
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.retries, 3);
    assert!(config.cache.enabled);
    assert_eq!(config.cache.api_key_ttl_seconds, 300);
    assert_eq!(config.cache.quota_ttl_seconds, 60);
}

#[test]
fn test_hub_cache_config_default() {
    let config = HubCacheConfig::default();

    assert!(config.enabled);
    assert_eq!(config.api_key_ttl_seconds, 300);
    assert_eq!(config.quota_ttl_seconds, 60);
    assert_eq!(config.max_entries, 10000);
}
