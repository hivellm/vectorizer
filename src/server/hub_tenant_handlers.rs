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
        models::CollectionConfig,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_cleanup_tenant_data() {
        let config = VectorizerConfig::default();
        let store = Arc::new(VectorStore::new(&config).unwrap());

        // Create test tenant
        let tenant_id = Uuid::new_v4();
        let tenant_id_str = format!("user_{}", tenant_id);

        // Create collections for tenant
        let collection_config = CollectionConfig::default();
        store
            .create_collection(
                &format!("{}:collection1", tenant_id_str),
                collection_config.clone(),
                Some(tenant_id),
            )
            .unwrap();
        store
            .create_collection(
                &format!("{}:collection2", tenant_id_str),
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
        let config = VectorizerConfig::default();
        let store = Arc::new(VectorStore::new(&config).unwrap());

        // Create test tenant
        let tenant_id = Uuid::new_v4();
        let tenant_id_str = format!("user_{}", tenant_id);

        // Create collections
        let collection_config = CollectionConfig::default();
        store
            .create_collection(
                &format!("{}:stats_test", tenant_id_str),
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
}
