//! REST API handlers for HiveHub usage statistics
//!
//! These endpoints provide usage metrics and statistics for HiveHub cluster mode,
//! with all operations scoped to the authenticated user/tenant.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::VectorizerServer;
use super::error_middleware::ErrorResponse;
use crate::hub::usage::UsageMetrics;
use crate::monitoring::metrics::METRICS;

/// Query parameters for usage statistics
#[derive(Debug, Deserialize)]
pub struct UsageStatsQuery {
    /// User ID to get stats for (required for HiveHub)
    pub user_id: Uuid,
    /// Optional collection ID filter
    #[serde(default)]
    pub collection_id: Option<Uuid>,
}

/// Response for usage statistics
#[derive(Debug, Serialize)]
pub struct UsageStatsResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<UserUsageStats>,
}

/// User usage statistics
#[derive(Debug, Serialize)]
pub struct UserUsageStats {
    /// User ID
    pub user_id: Uuid,
    /// Total collections owned
    pub total_collections: usize,
    /// Total vectors across all collections
    pub total_vectors: u64,
    /// Total storage used in bytes
    pub total_storage: u64,
    /// Number of API requests (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_requests: Option<u64>,
    /// Per-collection breakdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<CollectionUsageStats>>,
}

/// Per-collection usage statistics
#[derive(Debug, Serialize)]
pub struct CollectionUsageStats {
    /// Collection ID
    pub collection_id: Uuid,
    /// Collection name
    pub name: String,
    /// Number of vectors
    pub vectors: u64,
    /// Storage used in bytes
    pub storage: u64,
}

/// Get usage statistics for a user
///
/// GET /api/hub/usage/statistics?user_id=<uuid>&collection_id=<uuid>
pub async fn get_usage_statistics(
    State(state): State<VectorizerServer>,
    Query(query): Query<UsageStatsQuery>,
) -> Result<Json<UsageStatsResponse>, ErrorResponse> {
    info!("Getting usage statistics for user {}", query.user_id);

    // Check if HiveHub mode is enabled
    if state.hub_manager.is_none() {
        return Err(ErrorResponse::new(
            "HUB_DISABLED".to_string(),
            "HiveHub functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        ));
    }

    // Get hub manager (usage reporter is inside it)
    let hub_manager = match &state.hub_manager {
        Some(manager) => manager,
        None => {
            return Err(ErrorResponse::new(
                "USAGE_TRACKING_DISABLED".to_string(),
                "Usage tracking is not enabled".to_string(),
                StatusCode::SERVICE_UNAVAILABLE,
            ));
        }
    };

    // Get all collections for the user
    let user_prefix = format!("user_{}:", query.user_id);
    let all_collections = state.store.list_collections();
    let user_collections: Vec<String> = all_collections
        .into_iter()
        .filter(|name| name.starts_with(&user_prefix))
        .collect();

    // If specific collection requested, filter to just that one
    let filtered_collections: Vec<String> = if let Some(collection_id) = query.collection_id {
        let collection_name = format!("{}collection_{}", user_prefix, collection_id);
        if user_collections.contains(&collection_name) {
            vec![collection_name]
        } else {
            return Err(ErrorResponse::new(
                "COLLECTION_NOT_FOUND".to_string(),
                format!(
                    "Collection {} not found for user {}",
                    collection_id, query.user_id
                ),
                StatusCode::NOT_FOUND,
            ));
        }
    } else {
        user_collections
    };

    // Aggregate usage statistics
    let mut total_vectors = 0u64;
    let mut total_storage = 0u64;
    let mut collection_stats = Vec::new();

    for collection_name in &filtered_collections {
        // Try to get collection UUID from name
        // Format: user_{uuid}:collection_{uuid} or user_{uuid}:{name}
        let collection_id = if let Some(uuid_str) =
            collection_name.strip_prefix(&format!("{}collection_", user_prefix))
        {
            Uuid::parse_str(uuid_str).ok()
        } else {
            None
        };

        // Get collection info
        if let Ok(collection) = state.store.get_collection(collection_name) {
            let vectors = collection.vector_count() as u64;
            // Estimate storage based on vectors and dimension (rough estimate)
            // In a real implementation, collection should track actual storage
            let storage = vectors * 1024; // Rough estimate: 1KB per vector

            total_vectors += vectors;
            total_storage += storage;

            if let Some(coll_id) = collection_id {
                collection_stats.push(CollectionUsageStats {
                    collection_id: coll_id,
                    name: collection_name
                        .strip_prefix(&user_prefix)
                        .unwrap_or(collection_name)
                        .to_string(),
                    vectors,
                    storage,
                });
            }
        }
    }

    // Record metrics
    let user_id_str = query.user_id.to_string();
    let vector_count_str = "vector_count".to_string();
    let storage_str = "storage".to_string();

    METRICS
        .hub_quota_usage
        .with_label_values(&[&user_id_str, &vector_count_str])
        .set(total_vectors as f64);
    METRICS
        .hub_quota_usage
        .with_label_values(&[&user_id_str, &storage_str])
        .set(total_storage as f64);

    let stats = UserUsageStats {
        user_id: query.user_id,
        total_collections: filtered_collections.len(),
        total_vectors,
        total_storage,
        api_requests: Some(METRICS.get_tenant_api_requests(&query.user_id.to_string())),
        collections: if query.collection_id.is_some() || !collection_stats.is_empty() {
            Some(collection_stats)
        } else {
            None
        },
    };

    info!(
        "Usage stats for user {}: {} collections, {} vectors, {} bytes",
        query.user_id, stats.total_collections, stats.total_vectors, stats.total_storage
    );

    Ok(Json(UsageStatsResponse {
        success: true,
        message: "Usage statistics retrieved successfully".to_string(),
        stats: Some(stats),
    }))
}

/// Get current quota information for a user
///
/// GET /api/hub/usage/quota?user_id=<uuid>
pub async fn get_quota_info(
    State(state): State<VectorizerServer>,
    Query(query): Query<UsageStatsQuery>,
) -> Result<Json<Value>, ErrorResponse> {
    info!("Getting quota info for user {}", query.user_id);

    // Check if HiveHub mode is enabled
    let hub_manager = state.hub_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "HUB_DISABLED".to_string(),
            "HiveHub functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Get quota manager
    let quota_manager = hub_manager.quota();

    // Fetch quota information from HiveHub
    let quota_info = quota_manager
        .get_quota(&query.user_id.to_string())
        .await
        .map_err(|e| {
            error!("Failed to get quota info: {}", e);
            ErrorResponse::new(
                "QUOTA_FETCH_FAILED".to_string(),
                format!("Failed to retrieve quota information: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Convert to JSON response
    let response = serde_json::json!({
        "success": true,
        "message": "Quota information retrieved successfully",
        "quota": {
            "tenant_id": quota_info.tenant_id,
            "storage": {
                "limit": quota_info.storage.limit,
                "used": quota_info.storage.used,
                "remaining": quota_info.storage.remaining(),
                "usage_percent": quota_info.storage.usage_percent(),
                "can_allocate": quota_info.storage.can_allocate,
            },
            "vectors": {
                "limit": quota_info.vectors.limit,
                "used": quota_info.vectors.used,
                "remaining": quota_info.vectors.remaining(),
                "can_insert": quota_info.vectors.can_insert,
            },
            "collections": {
                "limit": quota_info.collections.limit,
                "used": quota_info.collections.used,
                "remaining": quota_info.collections.remaining(),
                "can_create": quota_info.collections.can_create,
            },
            "rate_limits": {
                "requests_per_minute": quota_info.rate_limits.requests_per_minute,
                "requests_per_hour": quota_info.rate_limits.requests_per_hour,
                "requests_per_day": quota_info.rate_limits.requests_per_day,
            },
            "updated_at": quota_info.updated_at,
        }
    });

    Ok(Json(response))
}

/// Validate API key
///
/// POST /api/hub/validate-key
/// Headers: Authorization: Bearer <api_key>
pub async fn validate_api_key(
    State(state): State<VectorizerServer>,
    headers: HeaderMap,
) -> Result<Json<Value>, ErrorResponse> {
    info!("Validating API key");

    // Check if HiveHub mode is enabled
    let hub_manager = state.hub_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "HUB_DISABLED".to_string(),
            "HiveHub functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Extract API key from Authorization header
    let api_key = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            warn!("Missing or invalid Authorization header");
            ErrorResponse::new(
                "MISSING_API_KEY".to_string(),
                "Missing or invalid Authorization header. Use 'Bearer <api_key>' format"
                    .to_string(),
                StatusCode::UNAUTHORIZED,
            )
        })?;

    // Validate the API key via auth manager
    let tenant_context = hub_manager
        .auth()
        .validate_api_key(api_key)
        .await
        .map_err(|e| {
            warn!("API key validation failed: {}", e);
            ErrorResponse::new(
                "INVALID_API_KEY".to_string(),
                "API key validation failed".to_string(),
                StatusCode::UNAUTHORIZED,
            )
        })?;

    // Record metrics
    let endpoint = "/api/hub/validate-key".to_string();
    let method = "POST".to_string();
    let status = "200".to_string();
    METRICS
        .hub_api_requests_total
        .with_label_values(&[&tenant_context.tenant_id, &endpoint, &method, &status])
        .inc();
    // Also record for fast tenant lookup
    METRICS.record_tenant_api_request(&tenant_context.tenant_id);

    // Return validation result with tenant info
    let response = serde_json::json!({
        "valid": true,
        "tenant_id": tenant_context.tenant_id,
        "tenant_name": tenant_context.tenant_name,
        "permissions": tenant_context.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
        "validated_at": tenant_context.validated_at.to_rfc3339(),
    });

    info!("API key validated for tenant: {}", tenant_context.tenant_id);

    Ok(Json(response))
}
