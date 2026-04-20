//! Collection-level REST handlers.
//!
//! - `list_collections`       — GET  /collections
//! - `create_collection`      — POST /collections
//! - `get_collection`         — GET  /collections/{name}
//! - `delete_collection`      — DELETE /collections/{name}
//! - `force_save_collection`  — POST /collections/{name}/save  (GUI)
//! - `list_empty_collections` — GET  /collections/empty        (GUI)
//! - `cleanup_empty_collections` — DELETE /collections/cleanup (GUI)

use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, error, info, warn};

use super::common::{collection_metrics_uuid, extract_tenant_id};
use crate::server::VectorizerServer;
use crate::server::error_middleware::ErrorResponse;
use vectorizer::auth::middleware::AuthState;
use vectorizer::auth::roles::Role;
use vectorizer::hub::middleware::RequestTenantContext;

/// GET /collections — list all collections (admin sees all; tenants see their own)
pub async fn list_collections(
    State(state): State<VectorizerServer>,
    auth_state: Option<Extension<AuthState>>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
) -> Json<Value> {
    // Check if user is admin - admins should see all collections
    let is_admin = auth_state
        .as_ref()
        .map(|auth| auth.user_claims.roles.contains(&Role::Admin))
        .unwrap_or(false);

    // Get collections based on tenant context and admin status
    let mut collections = if is_admin {
        // Admin users see all collections regardless of tenant
        debug!("Listing all collections for admin user");
        state.store.list_collections()
    } else {
        // Non-admin users only see their tenant's collections
        match extract_tenant_id(&tenant_ctx) {
            Some(tenant_id) => {
                debug!("Listing collections for tenant: {}", tenant_id);
                state.store.list_collections_for_owner(&tenant_id)
            }
            None => {
                // No tenant context - list all collections (non-tenant mode)
                state.store.list_collections()
            }
        }
    };

    // Sort alphabetically for consistent dashboard display
    collections.sort();

    let collection_infos: Vec<Value> = collections.iter().map(|name| {
        match state.store.get_collection(name) {
            Ok(collection) => {
                let metadata = collection.metadata();
                let config = collection.config();
                let (index_size, payload_size, total_size) = collection.get_size_info();
                let (index_bytes, payload_bytes, total_bytes) = collection.calculate_memory_usage();

                // Build normalization info
                let normalization_enabled = config.normalization
                    .as_ref()
                    .map(|n| n.enabled)
                    .unwrap_or(false);

                let normalization_level = if normalization_enabled {
                    config.normalization
                        .as_ref()
                        .map(|n| format!("{:?}", n.policy.level))
                        .unwrap_or_else(|| "None".to_string())
                } else {
                    "Disabled".to_string()
                };

                json!({
                    "name": name,
                    "vector_count": collection.vector_count(),
                    "document_count": metadata.document_count,
                    "dimension": config.dimension,
                    "metric": format!("{:?}", config.metric),
                    "embedding_provider": "bm25",
                    "size": {
                        "total": total_size,
                        "total_bytes": total_bytes,
                        "index": index_size,
                        "index_bytes": index_bytes,
                        "payload": payload_size,
                        "payload_bytes": payload_bytes
                    },
                    "quantization": {
                        "enabled": matches!(config.quantization, vectorizer::models::QuantizationConfig::SQ { bits: 8 }),
                        "type": format!("{:?}", config.quantization),
                        "bits": if matches!(config.quantization, vectorizer::models::QuantizationConfig::SQ { bits: 8 }) { 8 } else { 0 }
                    },
                    "normalization": {
                        "enabled": normalization_enabled,
                        "level": normalization_level
                    },
                    "created_at": metadata.created_at.to_rfc3339(),
                    "updated_at": chrono::Utc::now().to_rfc3339(),
                    "indexing_status": {
                        "status": "completed",
                        "progress": 1.0,
                        "total_documents": collection.vector_count(),
                        "processed_documents": collection.vector_count(),
                        "errors": 0,
                        "start_time": chrono::Utc::now().to_rfc3339(),
                        "end_time": chrono::Utc::now().to_rfc3339()
                    }
                })
            },
            Err(_) => json!({
                "name": name,
                "vector_count": 0,
                "document_count": 0,
                "dimension": 512,
                "metric": "Cosine",
                "embedding_provider": "bm25",
                "size": {
                    "total": "0 B",
                    "total_bytes": 0,
                    "index": "0 B",
                    "index_bytes": 0,
                    "payload": "0 B",
                    "payload_bytes": 0
                },
                "created_at": chrono::Utc::now().to_rfc3339(),
                "updated_at": chrono::Utc::now().to_rfc3339(),
                "indexing_status": {
                    "status": "error",
                    "progress": 0.0,
                    "total_documents": 0,
                    "processed_documents": 0,
                    "errors": 1,
                    "start_time": chrono::Utc::now().to_rfc3339(),
                    "end_time": chrono::Utc::now().to_rfc3339()
                }
            })
        }
    }).collect();

    Json(json!({
        "collections": collection_infos,
        "total_collections": collections.len()
    }))
}

/// POST /collections — create a new collection
pub async fn create_collection(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let name = payload
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "name",
                "missing or invalid name parameter",
            )
        })?;
    let dimension = payload
        .get("dimension")
        .and_then(|d| d.as_u64())
        .unwrap_or(512) as usize;
    let metric = payload
        .get("metric")
        .and_then(|m| m.as_str())
        .unwrap_or("cosine");

    // Extract tenant ID for multi-tenant mode
    let tenant_id = extract_tenant_id(&tenant_ctx);

    info!(
        "Creating collection: {} with dimension {} and metric {} (tenant: {:?})",
        name, dimension, metric, tenant_id
    );

    // Check HiveHub quota if enabled
    // In cluster mode, verify the tenant has quota for new collections
    if let Some(ref hub_manager) = state.hub_manager {
        let tenant_id_str = tenant_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "default".to_string());

        match hub_manager
            .check_quota(
                &tenant_id_str,
                vectorizer::hub::QuotaType::CollectionCount,
                1,
            )
            .await
        {
            Ok(allowed) => {
                if !allowed {
                    warn!(
                        "Collection creation denied for tenant {}: quota exceeded",
                        tenant_id_str
                    );
                    return Err(ErrorResponse::new(
                        "QUOTA_EXCEEDED".to_string(),
                        "Collection quota exceeded. Please upgrade your plan or delete unused collections.".to_string(),
                        StatusCode::TOO_MANY_REQUESTS,
                    ));
                }
            }
            Err(e) => {
                // Log the error but don't fail - the Hub handles actual enforcement
                warn!("Failed to check quota with HiveHub: {}", e);
            }
        }
    }

    // Parse graph configuration if provided
    let graph_config = payload.get("graph").and_then(|g| {
        if let Some(enabled) = g.get("enabled").and_then(|e| e.as_bool()) {
            if enabled {
                Some(vectorizer::models::GraphConfig {
                    enabled: true,
                    auto_relationship: vectorizer::models::AutoRelationshipConfig::default(),
                })
            } else {
                None
            }
        } else {
            None
        }
    });

    // Determine storage type: use MMap in cluster mode (enforce_mmap_storage),
    // otherwise default to Memory for standalone deployments.
    let storage_type = if let Some(ref cluster_mgr) = state.cluster_manager {
        // In cluster mode, MMap is required for data persistence across pod restarts
        info!(
            "Cluster mode active — using MMap storage for collection '{}'",
            name
        );
        Some(vectorizer::models::StorageType::Mmap)
    } else {
        Some(vectorizer::models::StorageType::Memory)
    };

    // Create collection configuration
    let config = vectorizer::models::CollectionConfig {
        dimension,
        metric: match metric {
            "cosine" => vectorizer::models::DistanceMetric::Cosine,
            "euclidean" => vectorizer::models::DistanceMetric::Euclidean,
            "dot" => vectorizer::models::DistanceMetric::DotProduct,
            _ => vectorizer::models::DistanceMetric::Cosine,
        },
        hnsw_config: vectorizer::models::HnswConfig::default(),
        quantization: vectorizer::models::QuantizationConfig::None,
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type,
        sharding: None,
        graph: graph_config,
        encryption: None,
    };

    // Actually create the collection in the store
    // In multi-tenant mode, associate collection with the owner
    if let Some(owner_id) = tenant_id {
        state
            .store
            .create_collection_with_owner(name, config, owner_id)
            .map_err(|e| ErrorResponse::from(e))?;
    } else {
        state
            .store
            .create_collection(name, config)
            .map_err(|e| ErrorResponse::from(e))?;
    }

    // Replicate collection creation to replicas.
    // Check both static master_node and HA manager (Raft-managed master).
    let active_master: Option<std::sync::Arc<vectorizer::replication::MasterNode>> = state
        .master_node
        .clone()
        .or_else(|| state.ha_manager.as_ref().and_then(|ha| ha.master_node()));

    if let Some(ref master) = active_master {
        let op = vectorizer::replication::VectorOperation::CreateCollection {
            name: name.to_string(),
            config: vectorizer::replication::CollectionConfigData {
                dimension,
                metric: metric.to_string(),
            },
            owner_id: tenant_id.map(|id| id.to_string()),
        };
        master.replicate(op);
        debug!("Replicated collection creation: {}", name);
    }

    // Mark changes for auto-save
    if let Some(ref auto_save) = state.auto_save_manager {
        auto_save.mark_changed();
    }

    // Record usage metrics if HiveHub is enabled
    if let Some(ref hub_manager) = state.hub_manager {
        let mut metrics = vectorizer::hub::UsageMetrics::new();
        metrics.record_collection_create();
        // Stable UUID derived from the collection name so subsequent calls
        // aggregate under the same Hub usage row.
        let collection_id = collection_metrics_uuid(&name);
        if let Err(e) = hub_manager.record_usage(collection_id, metrics).await {
            warn!("Failed to record collection creation usage: {}", e);
        }
    }

    info!("Collection '{}' created successfully", name);
    Ok(Json(json!({
        "message": format!("Collection '{}' created successfully", name),
        "collection": name,
        "dimension": dimension,
        "metric": metric
    })))
}

/// GET /collections/{name} — retrieve collection details
pub async fn get_collection(
    State(state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ErrorResponse> {
    let collection = state
        .store
        .get_collection(&name)
        .map_err(|e| ErrorResponse::from(e))?;

    let metadata = collection.metadata();
    let config = collection.config();
    let (index_size, payload_size, total_size) = collection.get_size_info();
    let (index_bytes, payload_bytes, total_bytes) = collection.calculate_memory_usage();

    // Build normalization info
    let normalization_info = if let Some(norm_config) = &config.normalization {
        json!({
            "enabled": norm_config.enabled,
            "level": format!("{:?}", norm_config.policy.level),
            "preserve_case": norm_config.policy.preserve_case,
            "collapse_whitespace": norm_config.policy.collapse_whitespace,
            "remove_html": norm_config.policy.remove_html,
            "cache_enabled": norm_config.cache_enabled,
            "cache_size_mb": norm_config.hot_cache_size / (1024 * 1024),
            "normalize_queries": norm_config.normalize_queries,
            "store_raw_text": norm_config.store_raw_text,
        })
    } else {
        json!({
            "enabled": false,
            "message": "Text normalization is disabled for this collection"
        })
    };

    Ok(Json(json!({
        "name": name,
        "vector_count": collection.vector_count(),
        "document_count": metadata.document_count,
        "dimension": config.dimension,
        "metric": format!("{:?}", config.metric),
        "created_at": metadata.created_at.to_rfc3339(),
        "updated_at": metadata.updated_at.to_rfc3339(),
        "size": {
            "total": total_size,
            "total_bytes": total_bytes,
            "index": index_size,
            "index_bytes": index_bytes,
            "payload": payload_size,
            "payload_bytes": payload_bytes
        },
        "quantization": {
            "enabled": matches!(config.quantization, vectorizer::models::QuantizationConfig::SQ { bits: 8 }),
            "type": format!("{:?}", config.quantization),
            "bits": if matches!(config.quantization, vectorizer::models::QuantizationConfig::SQ { bits: 8 }) { 8 } else { 0 }
        },
        "normalization": normalization_info,
        "status": "ready"
    })))
}

/// DELETE /collections/{name} — delete a collection
pub async fn delete_collection(
    State(state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ErrorResponse> {
    info!("Deleting collection: {}", name);

    state
        .store
        .delete_collection(&name)
        .map_err(|e| ErrorResponse::from(e))?;

    // Mark changes for auto-save
    if let Some(ref auto_save) = state.auto_save_manager {
        auto_save.mark_changed();
    }

    // Invalidate cache for this collection
    state.query_cache.invalidate_collection(&name);
    debug!(
        "💾 Cache invalidated for collection '{}' after deletion",
        name
    );

    Ok(Json(json!({
        "message": format!("Collection '{}' deleted successfully", name)
    })))
}

/// POST /collections/{name}/save — force-save a collection (GUI)
pub async fn force_save_collection(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<Value>, ErrorResponse> {
    info!("💾 Force saving collection: {}", collection_name);

    // Verify collection exists
    match state.store.get_collection(&collection_name) {
        Ok(_) => {
            // Force save all collections (including the requested one)
            match state.store.force_save_all() {
                Ok(_) => Ok(Json(json!({
                    "success": true,
                    "message": format!("Collection '{}' saved successfully", collection_name)
                }))),
                Err(e) => {
                    error!("Failed to force save: {}", e);
                    Ok(Json(json!({
                        "success": false,
                        "message": format!("Failed to save collection: {}", e)
                    })))
                }
            }
        }
        Err(e) => {
            error!("Collection not found: {}", e);
            Err(ErrorResponse::from(e))
        }
    }
}

/// GET /collections/empty — list empty collections (GUI)
pub async fn list_empty_collections(State(state): State<VectorizerServer>) -> Json<Value> {
    let empty_collections = state.store.list_empty_collections();

    info!("Found {} empty collections", empty_collections.len());

    json!({
        "status": "success",
        "empty_collections": empty_collections,
        "count": empty_collections.len()
    })
    .into()
}

/// DELETE /collections/cleanup — delete all empty collections (GUI)
pub async fn cleanup_empty_collections(
    State(state): State<VectorizerServer>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    match state.store.cleanup_empty_collections(dry_run) {
        Ok(deleted_count) => {
            let message = if dry_run {
                format!("Would delete {} empty collections", deleted_count)
            } else {
                format!("Successfully deleted {} empty collections", deleted_count)
            };

            info!("{}", message);

            Ok(Json(json!({
                "status": "success",
                "message": message,
                "deleted_count": deleted_count,
                "dry_run": dry_run
            })))
        }
        Err(e) => {
            error!("Failed to cleanup empty collections: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Cleanup failed: {}", e)
                })),
            ))
        }
    }
}
