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

/// POST /api/hub/tenant/:tenant_id/migrate
/// Migrate tenant data (placeholder for future migration functionality)
pub async fn migrate_tenant_data(
    State(_state): State<VectorizerServer>,
    Path(tenant_id): Path<String>,
) -> Result<impl IntoResponse, VectorizerError> {
    info!("🔄 Tenant migration request for tenant: {}", tenant_id);

    // TODO: Implement tenant migration logic
    // This could include:
    // - Moving tenant data to a different cluster
    // - Exporting tenant data
    // - Changing tenant ownership
    // - Merging tenants

    Ok((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "message": "Tenant migration not yet implemented",
            "tenant_id": tenant_id
        })),
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
