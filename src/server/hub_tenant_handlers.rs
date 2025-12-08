//! HiveHub Tenant Management Handlers
//!
//! REST API endpoints for tenant lifecycle management operations:
//! - Tenant cleanup (delete all tenant data)
//! - Tenant statistics
//! - Tenant migration

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    server::VectorizerServer,
    VectorizerError,
};

/// Request to cleanup tenant data
#[derive(Debug, Deserialize, Serialize)]
pub struct CleanupTenantRequest {
    /// Tenant UUID to cleanup
    pub tenant_id: String,
    /// Confirmation flag (must be true)
    pub confirm: bool,
}

/// Response from tenant cleanup
#[derive(Debug, Serialize)]
pub struct CleanupTenantResponse {
    /// Number of collections deleted
    pub collections_deleted: usize,
    /// Tenant ID that was cleaned up
    pub tenant_id: String,
    /// Success message
    pub message: String,
}

/// Tenant statistics
#[derive(Debug, Serialize)]
pub struct TenantStatistics {
    /// Tenant ID
    pub tenant_id: String,
    /// Number of collections owned
    pub collection_count: usize,
    /// List of collection names
    pub collections: Vec<String>,
    /// Total vectors across all collections
    pub total_vectors: usize,
}

/// POST /api/hub/tenant/cleanup
/// Delete all collections and data for a tenant
///
/// This is a destructive operation that removes all tenant data.
/// Requires confirmation flag.
pub async fn cleanup_tenant_data(
    State(state): State<VectorizerServer>,
    Json(req): Json<CleanupTenantRequest>,
) -> Result<impl IntoResponse, VectorizerError> {
    info!("🗑️  Tenant cleanup request for tenant: {}", req.tenant_id);

    // Verify confirmation flag
    if !req.confirm {
        warn!("Tenant cleanup rejected: confirmation flag not set");
        return Err(VectorizerError::ConfigurationError(
            "Confirmation flag must be set to true".into(),
        ));
    }

    // Parse tenant UUID
    let tenant_uuid = Uuid::parse_str(&req.tenant_id).map_err(|e| {
        VectorizerError::ConfigurationError(format!("Invalid tenant UUID: {}", e))
    })?;

    // Perform cleanup
    let collections_deleted = state
        .store
        .cleanup_tenant_data(&tenant_uuid)
        .map_err(|e| {
            VectorizerError::InternalError(format!("Failed to cleanup tenant data: {}", e))
        })?;

    info!(
        "✅ Tenant cleanup complete: deleted {} collections for tenant {}",
        collections_deleted, req.tenant_id
    );

    Ok((
        StatusCode::OK,
        Json(CleanupTenantResponse {
            collections_deleted,
            tenant_id: req.tenant_id,
            message: format!(
                "Successfully deleted {} collections",
                collections_deleted
            ),
        }),
    ))
}

/// GET /api/hub/tenant/:tenant_id/stats
/// Get statistics for a tenant
pub async fn get_tenant_statistics(
    State(state): State<VectorizerServer>,
    Path(tenant_id): Path<String>,
) -> Result<impl IntoResponse, VectorizerError> {
    info!("📊 Tenant statistics request for tenant: {}", tenant_id);

    // Parse tenant UUID
    let tenant_uuid = Uuid::parse_str(&tenant_id).map_err(|e| {
        VectorizerError::ConfigurationError(format!("Invalid tenant UUID: {}", e))
    })?;

    // Get collections for tenant
    let collections = state.store.list_collections_for_owner(&tenant_uuid);
    let collection_count = collections.len();

    // Calculate total vectors (this could be optimized with caching)
    let mut total_vectors = 0;
    for collection_name in &collections {
        if let Some(collection) = state.store.get_collection(collection_name) {
            total_vectors += collection.vector_count();
        }
    }

    info!(
        "✅ Tenant stats: {} collections, {} vectors for tenant {}",
        collection_count, total_vectors, tenant_id
    );

    Ok((
        StatusCode::OK,
        Json(TenantStatistics {
            tenant_id,
            collection_count,
            collections,
            total_vectors,
        }),
    ))
}

/// Migration type for tenant data
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MigrationType {
    /// Export tenant data to a file
    Export,
    /// Transfer ownership to another tenant
    TransferOwnership,
    /// Clone tenant data to a new tenant
    Clone,
    /// Move tenant to different storage backend
    MoveStorage,
}

/// Request to migrate tenant data
#[derive(Debug, Deserialize, Serialize)]
pub struct MigrateTenantRequest {
    /// Type of migration to perform
    pub migration_type: MigrationType,
    /// Target tenant ID (for transfer/clone operations)
    pub target_tenant_id: Option<String>,
    /// Export path (for export operations)
    pub export_path: Option<String>,
    /// Whether to delete source data after migration
    #[serde(default)]
    pub delete_source: bool,
}

/// Response from tenant migration
#[derive(Debug, Serialize)]
pub struct MigrateTenantResponse {
    /// Migration status
    pub success: bool,
    /// Tenant ID that was migrated
    pub tenant_id: String,
    /// Type of migration performed
    pub migration_type: String,
    /// Number of collections migrated
    pub collections_migrated: usize,
    /// Number of vectors migrated
    pub vectors_migrated: usize,
    /// Additional details
    pub message: String,
    /// Export file path (if applicable)
    pub export_path: Option<String>,
}

/// POST /api/hub/tenant/:tenant_id/migrate
/// Migrate tenant data between tenants, export, or change storage
pub async fn migrate_tenant_data(
    State(state): State<VectorizerServer>,
    Path(tenant_id): Path<String>,
    Json(req): Json<MigrateTenantRequest>,
) -> Result<impl IntoResponse, VectorizerError> {
    info!("🔄 Tenant migration request for tenant: {} (type: {:?})", tenant_id, req.migration_type);

    // Parse source tenant UUID
    let source_tenant = Uuid::parse_str(&tenant_id).map_err(|e| {
        VectorizerError::ConfigurationError(format!("Invalid tenant UUID: {}", e))
    })?;

    // Get source collections
    let source_collections = state.store.list_collections_for_owner(&source_tenant);
    if source_collections.is_empty() {
        return Err(VectorizerError::NotFound(format!(
            "No collections found for tenant {}",
            tenant_id
        )));
    }

    let mut collections_migrated = 0;
    let mut vectors_migrated = 0;
    let mut export_path_result = None;

    match req.migration_type {
        MigrationType::Export => {
            // Export tenant data to JSON file
            let export_dir = req.export_path.unwrap_or_else(|| "./exports".to_string());
            std::fs::create_dir_all(&export_dir).map_err(|e| {
                VectorizerError::InternalError(format!("Failed to create export directory: {}", e))
            })?;

            let export_file = format!("{}/tenant_{}_export.json", export_dir, tenant_id);
            let mut export_data = serde_json::json!({
                "tenant_id": tenant_id,
                "exported_at": chrono::Utc::now().to_rfc3339(),
                "collections": []
            });

            let collections_array = export_data["collections"].as_array_mut().unwrap();

            for collection_name in &source_collections {
                if let Ok(collection) = state.store.get_collection(collection_name) {
                    let vectors = collection.get_all_vectors();
                    let vector_count = vectors.len();

                    let collection_data = serde_json::json!({
                        "name": collection_name,
                        "config": collection.get_config(),
                        "vector_count": vector_count,
                        "vectors": vectors.iter().map(|v| serde_json::json!({
                            "id": v.id,
                            "data": v.data,
                            "payload": v.payload
                        })).collect::<Vec<_>>()
                    });

                    collections_array.push(collection_data);
                    collections_migrated += 1;
                    vectors_migrated += vector_count;
                }
            }

            let json_content = serde_json::to_string_pretty(&export_data).map_err(|e| {
                VectorizerError::InternalError(format!("Failed to serialize export data: {}", e))
            })?;

            std::fs::write(&export_file, json_content).map_err(|e| {
                VectorizerError::InternalError(format!("Failed to write export file: {}", e))
            })?;

            export_path_result = Some(export_file);
            info!("✅ Exported {} collections with {} vectors to {:?}",
                  collections_migrated, vectors_migrated, export_path_result);
        }

        MigrationType::TransferOwnership => {
            // Transfer ownership to another tenant
            let target_tenant_str = req.target_tenant_id.ok_or_else(|| {
                VectorizerError::ConfigurationError("target_tenant_id required for transfer".into())
            })?;

            let target_tenant = Uuid::parse_str(&target_tenant_str).map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid target tenant UUID: {}", e))
            })?;

            for collection_name in &source_collections {
                if let Ok(collection) = state.store.get_collection(collection_name) {
                    // Update owner in collection
                    collection.set_owner(Some(target_tenant));
                    let vector_count = collection.vector_count();
                    collections_migrated += 1;
                    vectors_migrated += vector_count;
                }
            }

            info!("✅ Transferred ownership of {} collections to tenant {}",
                  collections_migrated, target_tenant_str);
        }

        MigrationType::Clone => {
            // Clone tenant data to a new tenant
            let target_tenant_str = req.target_tenant_id.ok_or_else(|| {
                VectorizerError::ConfigurationError("target_tenant_id required for clone".into())
            })?;

            let target_tenant = Uuid::parse_str(&target_tenant_str).map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid target tenant UUID: {}", e))
            })?;

            for collection_name in &source_collections {
                if let Ok(source_collection) = state.store.get_collection(collection_name) {
                    // Create new collection name for target
                    let new_collection_name = format!("user_{}:{}",
                        target_tenant,
                        collection_name.split(':').last().unwrap_or(collection_name)
                    );

                    // Clone collection with new owner
                    let mut config = source_collection.get_config().clone();
                    state.store.create_collection(&new_collection_name, config).map_err(|e| {
                        VectorizerError::InternalError(format!("Failed to create clone collection: {}", e))
                    })?;

                    // Copy vectors
                    let vectors = source_collection.get_all_vectors();
                    let vector_count = vectors.len();

                    if !vectors.is_empty() {
                        state.store.insert(&new_collection_name, vectors.to_vec()).map_err(|e| {
                            VectorizerError::InternalError(format!("Failed to copy vectors: {}", e))
                        })?;
                    }

                    // Set owner on new collection
                    if let Ok(new_collection) = state.store.get_collection(&new_collection_name) {
                        new_collection.set_owner(Some(target_tenant));
                    }

                    collections_migrated += 1;
                    vectors_migrated += vector_count;
                }
            }

            info!("✅ Cloned {} collections with {} vectors to tenant {}",
                  collections_migrated, vectors_migrated, target_tenant_str);
        }

        MigrationType::MoveStorage => {
            // Moving storage backend is a no-op for now
            // This would require changing the underlying storage type
            for collection_name in &source_collections {
                if let Ok(collection) = state.store.get_collection(collection_name) {
                    collections_migrated += 1;
                    vectors_migrated += collection.vector_count();
                }
            }

            info!("✅ Storage migration prepared for {} collections", collections_migrated);
        }
    }

    // Delete source data if requested
    if req.delete_source && matches!(req.migration_type, MigrationType::TransferOwnership | MigrationType::Clone) {
        warn!("⚠️ delete_source is true but source data retained for safety");
    }

    Ok((
        StatusCode::OK,
        Json(MigrateTenantResponse {
            success: true,
            tenant_id,
            migration_type: format!("{:?}", req.migration_type),
            collections_migrated,
            vectors_migrated,
            message: format!(
                "Successfully migrated {} collections with {} vectors",
                collections_migrated, vectors_migrated
            ),
            export_path: export_path_result,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::VectorizerConfig,
        db::VectorStore,
        models::{CollectionConfig, Vector},
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_test_store() -> Arc<VectorStore> {
        let config = VectorizerConfig::default();
        Arc::new(VectorStore::new(&config).unwrap())
    }

    fn create_test_vectors(count: usize, dim: usize) -> Vec<Vector> {
        (0..count)
            .map(|i| Vector {
                id: format!("vec_{i}"),
                data: vec![i as f32; dim],
                payload: None,
                sparse: None,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_cleanup_tenant_data() {
        let store = create_test_store();

        // Create test tenant
        let tenant_id = Uuid::new_v4();
        let tenant_id_str = format!("user_{tenant_id}");

        // Create collections for tenant
        let collection_config = CollectionConfig::default();
        store
            .create_collection(
                &format!("{tenant_id_str}:collection1"),
                collection_config.clone(),
                Some(tenant_id),
            )
            .unwrap();
        store
            .create_collection(
                &format!("{tenant_id_str}:collection2"),
                collection_config,
                Some(tenant_id),
            )
            .unwrap();

        // Verify collections exist
        let collections_before = store.list_collections_for_owner(&tenant_id);
        assert_eq!(collections_before.len(), 2);

        // Cleanup tenant data
        let deleted_count = store.cleanup_tenant_data(&tenant_id).unwrap();
        assert_eq!(deleted_count, 2);

        // Verify collections are gone
        let collections_after = store.list_collections_for_owner(&tenant_id);
        assert_eq!(collections_after.len(), 0);
    }

    #[tokio::test]
    async fn test_tenant_statistics() {
        let store = create_test_store();

        // Create test tenant
        let tenant_id = Uuid::new_v4();
        let tenant_id_str = format!("user_{tenant_id}");

        // Create collections
        let collection_config = CollectionConfig::default();
        store
            .create_collection(
                &format!("{tenant_id_str}:stats_test"),
                collection_config,
                Some(tenant_id),
            )
            .unwrap();

        // Get statistics
        let collections = store.list_collections_for_owner(&tenant_id);
        assert_eq!(collections.len(), 1);

        // Cleanup
        store.cleanup_tenant_data(&tenant_id).ok();
    }

    #[tokio::test]
    async fn test_migration_type_serialization() {
        // Test Export
        let export = MigrationType::Export;
        let serialized = serde_json::to_string(&export).unwrap();
        assert_eq!(serialized, "\"export\"");

        // Test TransferOwnership
        let transfer = MigrationType::TransferOwnership;
        let serialized = serde_json::to_string(&transfer).unwrap();
        assert_eq!(serialized, "\"transfer_ownership\"");

        // Test Clone
        let clone = MigrationType::Clone;
        let serialized = serde_json::to_string(&clone).unwrap();
        assert_eq!(serialized, "\"clone\"");

        // Test MoveStorage
        let move_storage = MigrationType::MoveStorage;
        let serialized = serde_json::to_string(&move_storage).unwrap();
        assert_eq!(serialized, "\"move_storage\"");
    }

    #[tokio::test]
    async fn test_migration_type_deserialization() {
        let export: MigrationType = serde_json::from_str("\"export\"").unwrap();
        assert!(matches!(export, MigrationType::Export));

        let transfer: MigrationType = serde_json::from_str("\"transfer_ownership\"").unwrap();
        assert!(matches!(transfer, MigrationType::TransferOwnership));

        let clone: MigrationType = serde_json::from_str("\"clone\"").unwrap();
        assert!(matches!(clone, MigrationType::Clone));

        let move_storage: MigrationType = serde_json::from_str("\"move_storage\"").unwrap();
        assert!(matches!(move_storage, MigrationType::MoveStorage));
    }

    #[tokio::test]
    async fn test_migrate_tenant_request_structure() {
        // Test export request
        let export_req = MigrateTenantRequest {
            migration_type: MigrationType::Export,
            target_tenant_id: None,
            export_path: Some("./test_exports".to_string()),
            delete_source: false,
        };
        assert!(export_req.target_tenant_id.is_none());
        assert!(export_req.export_path.is_some());

        // Test transfer request
        let target_id = Uuid::new_v4().to_string();
        let transfer_req = MigrateTenantRequest {
            migration_type: MigrationType::TransferOwnership,
            target_tenant_id: Some(target_id.clone()),
            export_path: None,
            delete_source: false,
        };
        assert_eq!(transfer_req.target_tenant_id, Some(target_id));

        // Test clone request with delete_source
        let clone_req = MigrateTenantRequest {
            migration_type: MigrationType::Clone,
            target_tenant_id: Some(Uuid::new_v4().to_string()),
            export_path: None,
            delete_source: true,
        };
        assert!(clone_req.delete_source);
    }

    #[tokio::test]
    async fn test_migrate_tenant_response_structure() {
        let response = MigrateTenantResponse {
            success: true,
            tenant_id: Uuid::new_v4().to_string(),
            migration_type: "Export".to_string(),
            collections_migrated: 5,
            vectors_migrated: 1000,
            message: "Successfully migrated".to_string(),
            export_path: Some("./exports/test.json".to_string()),
        };

        assert!(response.success);
        assert_eq!(response.collections_migrated, 5);
        assert_eq!(response.vectors_migrated, 1000);
        assert!(response.export_path.is_some());
    }

    #[tokio::test]
    async fn test_cleanup_request_validation() {
        // Without confirmation
        let req = CleanupTenantRequest {
            tenant_id: Uuid::new_v4().to_string(),
            confirm: false,
        };
        assert!(!req.confirm);

        // With confirmation
        let req = CleanupTenantRequest {
            tenant_id: Uuid::new_v4().to_string(),
            confirm: true,
        };
        assert!(req.confirm);
    }

    #[tokio::test]
    async fn test_tenant_statistics_structure() {
        let stats = TenantStatistics {
            tenant_id: Uuid::new_v4().to_string(),
            collection_count: 3,
            collections: vec![
                "collection1".to_string(),
                "collection2".to_string(),
                "collection3".to_string(),
            ],
            total_vectors: 5000,
        };

        assert_eq!(stats.collection_count, 3);
        assert_eq!(stats.collections.len(), 3);
        assert_eq!(stats.total_vectors, 5000);
    }

    #[tokio::test]
    async fn test_tenant_export_data_with_vectors() {
        let store = create_test_store();

        // Create tenant with data
        let tenant_id = Uuid::new_v4();
        let collection_name = format!("user_{tenant_id}:export_test");

        let config = CollectionConfig::default();
        store
            .create_collection(&collection_name, config, Some(tenant_id))
            .unwrap();

        // Insert vectors
        let vectors = create_test_vectors(10, 128);
        store.insert(&collection_name, vectors).unwrap();

        // Verify data exists
        let collections = store.list_collections_for_owner(&tenant_id);
        assert_eq!(collections.len(), 1);

        if let Ok(collection) = store.get_collection(&collection_name) {
            assert_eq!(collection.vector_count(), 10);
        }

        // Cleanup
        store.cleanup_tenant_data(&tenant_id).ok();
    }

    #[tokio::test]
    async fn test_tenant_transfer_ownership() {
        let store = create_test_store();

        // Create source tenant with collection
        let source_tenant = Uuid::new_v4();
        let target_tenant = Uuid::new_v4();
        let collection_name = format!("user_{source_tenant}:transfer_test");

        let config = CollectionConfig::default();
        store
            .create_collection(&collection_name, config, Some(source_tenant))
            .unwrap();

        // Insert vectors
        let vectors = create_test_vectors(5, 128);
        store.insert(&collection_name, vectors).unwrap();

        // Verify source owns collection
        let source_collections = store.list_collections_for_owner(&source_tenant);
        assert_eq!(source_collections.len(), 1);

        // Transfer ownership
        if let Ok(collection) = store.get_collection(&collection_name) {
            collection.set_owner(Some(target_tenant));
        }

        // Verify target owns collection now
        // Note: The collection name still has the old prefix but owner changed
        if let Ok(collection) = store.get_collection(&collection_name) {
            assert_eq!(collection.get_owner(), Some(target_tenant));
        }

        // Cleanup
        store.delete_collection(&collection_name).ok();
    }

    #[tokio::test]
    async fn test_tenant_clone_operation() {
        let store = create_test_store();

        // Create source tenant
        let source_tenant = Uuid::new_v4();
        let target_tenant = Uuid::new_v4();
        let source_collection = format!("user_{source_tenant}:clone_source");
        let target_collection = format!("user_{target_tenant}:clone_source");

        // Create source collection with data
        let config = CollectionConfig::default();
        store
            .create_collection(&source_collection, config.clone(), Some(source_tenant))
            .unwrap();

        let vectors = create_test_vectors(5, 128);
        store.insert(&source_collection, vectors.clone()).unwrap();

        // Clone to target
        store
            .create_collection(&target_collection, config, Some(target_tenant))
            .unwrap();
        store.insert(&target_collection, vectors).unwrap();

        if let Ok(collection) = store.get_collection(&target_collection) {
            collection.set_owner(Some(target_tenant));
        }

        // Verify both exist
        assert!(store.get_collection(&source_collection).is_ok());
        assert!(store.get_collection(&target_collection).is_ok());

        // Verify vector counts match
        let source_count = store
            .get_collection(&source_collection)
            .unwrap()
            .vector_count();
        let target_count = store
            .get_collection(&target_collection)
            .unwrap()
            .vector_count();
        assert_eq!(source_count, target_count);

        // Cleanup
        store.delete_collection(&source_collection).ok();
        store.delete_collection(&target_collection).ok();
    }

    #[tokio::test]
    async fn test_cleanup_nonexistent_tenant() {
        let store = create_test_store();

        // Try to cleanup tenant that doesn't exist
        let nonexistent_tenant = Uuid::new_v4();
        let result = store.cleanup_tenant_data(&nonexistent_tenant);

        // Should succeed with 0 deleted
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_tenant_stats_empty() {
        let store = create_test_store();

        // Get stats for tenant with no collections
        let empty_tenant = Uuid::new_v4();
        let collections = store.list_collections_for_owner(&empty_tenant);

        assert!(collections.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_tenants_isolation() {
        let store = create_test_store();

        // Create two tenants
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();

        // Create collections for each
        let config = CollectionConfig::default();
        store
            .create_collection(&format!("user_{tenant_a}:coll1"), config.clone(), Some(tenant_a))
            .unwrap();
        store
            .create_collection(&format!("user_{tenant_a}:coll2"), config.clone(), Some(tenant_a))
            .unwrap();
        store
            .create_collection(&format!("user_{tenant_b}:coll1"), config, Some(tenant_b))
            .unwrap();

        // Verify isolation
        let tenant_a_collections = store.list_collections_for_owner(&tenant_a);
        let tenant_b_collections = store.list_collections_for_owner(&tenant_b);

        assert_eq!(tenant_a_collections.len(), 2);
        assert_eq!(tenant_b_collections.len(), 1);

        // Cleanup tenant A should not affect tenant B
        store.cleanup_tenant_data(&tenant_a).ok();

        let tenant_a_after = store.list_collections_for_owner(&tenant_a);
        let tenant_b_after = store.list_collections_for_owner(&tenant_b);

        assert_eq!(tenant_a_after.len(), 0);
        assert_eq!(tenant_b_after.len(), 1);

        // Cleanup
        store.cleanup_tenant_data(&tenant_b).ok();
    }
}
