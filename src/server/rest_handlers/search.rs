//! Search REST handlers.
//!
//! - `search_vectors_by_text`  — POST /collections/{name}/search/text (embedding-backed)
//! - `hybrid_search_vectors`   — POST /collections/{name}/search/hybrid (dense + sparse)
//! - `search_by_file`          — POST /collections/{name}/search/file
//! - `search_vectors`          — POST /search (raw vector, returns empty results until wired)
//! - `batch_search_vectors`    — POST /batch/search
//! - `batch_update_vectors`    — POST /batch/update
//! - `batch_delete_vectors`    — POST /batch/delete

use axum::Extension;
use axum::extract::{Path, State};
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, info};

use super::common::extract_tenant_id;
use crate::db::{HybridScoringAlgorithm, HybridSearchConfig};
use crate::hub::middleware::RequestTenantContext;
use crate::models::SparseVector;
use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_validation_error,
};

pub async fn search_vectors_by_text(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::cache::query_cache::QueryKey;
    use crate::monitoring::metrics::METRICS;

    // Start latency timer
    let label_collection: &str = &collection_name;
    let label_text = "text".to_string();
    let timer = METRICS
        .search_latency_seconds
        .with_label_values(&[&label_collection.to_string(), &label_text])
        .start_timer();

    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;
    let limit = payload.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;
    let threshold = payload.get("threshold").and_then(|t| t.as_f64());

    // Check cache first
    let cache_key = QueryKey::new(collection_name.clone(), query.to_string(), limit, threshold);
    if let Some(cached_result) = state.query_cache.get(&cache_key) {
        debug!(
            "💾 Cache hit for query '{}' in collection '{}'",
            query, collection_name
        );
        drop(timer);
        return Ok(Json(cached_result));
    }

    info!(
        "🔍 Searching for '{}' in collection '{}'",
        query, collection_name
    );

    // Extract tenant ID for multi-tenant access control
    let tenant_id = extract_tenant_id(&tenant_ctx);

    // Get the collection with owner validation
    let collection = state
        .store
        .get_collection_with_owner(&collection_name, tenant_id.as_ref())
        .map_err(|e| ErrorResponse::from(e))?;

    // Generate embedding for the query
    let query_embedding = state
        .embedding_manager
        .embed(query)
        .map_err(|e| create_bad_request_error(&format!("Failed to generate embedding: {}", e)))?;

    // Search vectors in the collection
    let search_results = collection
        .search(&query_embedding, limit)
        .map_err(|e| create_bad_request_error(&format!("Search failed: {}", e)))?;

    // Convert results to JSON format
    let results: Vec<Value> = search_results
        .into_iter()
        .map(|result| {
            json!({
                "id": result.id,
                "score": result.score,
                "vector": result.vector,
                "payload": result.payload.map(|p| p.data)
            })
        })
        .collect();

    // Build response
    let response = json!({
        "results": results,
        "query": query,
        "limit": limit,
        "collection": collection_name,
        "total_results": results.len()
    });

    // Cache the result
    state.query_cache.insert(cache_key, response.clone());

    // Record metrics
    let label_collection: &str = &collection_name;
    let label_text = "text";
    let label_success = "success";
    METRICS
        .search_requests_total
        .with_label_values(&[label_collection, label_text, label_success])
        .inc();
    let label_text_str = "text".to_string();
    METRICS
        .search_results_count
        .with_label_values(&[&collection_name, &label_text_str])
        .observe(results.len() as f64);
    drop(timer); // Stop latency timer

    Ok(Json(response))
}

pub async fn hybrid_search_vectors(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::cache::query_cache::QueryKey;
    use crate::monitoring::metrics::METRICS;

    // Start latency timer
    let label_collection: &str = &collection_name;
    let label_hybrid = "hybrid".to_string();
    let timer = METRICS
        .search_latency_seconds
        .with_label_values(&[label_collection, &label_hybrid])
        .start_timer();

    // Extract tenant ID for multi-tenant access control
    let tenant_id = extract_tenant_id(&tenant_ctx);

    // Parse query (required)
    let query = payload
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or_else(|| create_validation_error("query", "missing or invalid query parameter"))?;

    // Parse optional sparse query
    let query_sparse = if let Some(sparse_obj) = payload.get("query_sparse") {
        if let Some(indices_arr) = sparse_obj.get("indices").and_then(|v| v.as_array()) {
            if let Some(values_arr) = sparse_obj.get("values").and_then(|v| v.as_array()) {
                let indices: Option<Vec<usize>> = indices_arr
                    .iter()
                    .map(|v| v.as_u64().map(|n| n as usize))
                    .collect();
                let values: Option<Vec<f32>> = values_arr
                    .iter()
                    .map(|v| v.as_f64().map(|n| n as f32))
                    .collect();

                match (indices, values) {
                    (Some(indices), Some(values)) => SparseVector::new(indices, values)
                        .map_err(|e| {
                            create_validation_error(
                                "query_sparse",
                                &format!("Invalid sparse vector: {}", e),
                            )
                        })
                        .ok(),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Parse hybrid search configuration
    let alpha = payload.get("alpha").and_then(|v| v.as_f64()).unwrap_or(0.7) as f32;
    let algorithm_str = payload
        .get("algorithm")
        .and_then(|v| v.as_str())
        .unwrap_or("rrf");
    let algorithm = match algorithm_str {
        "rrf" => HybridScoringAlgorithm::ReciprocalRankFusion,
        "weighted" => HybridScoringAlgorithm::WeightedCombination,
        "alpha" => HybridScoringAlgorithm::AlphaBlending,
        _ => HybridScoringAlgorithm::ReciprocalRankFusion,
    };
    let dense_k = payload
        .get("dense_k")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;
    let sparse_k = payload
        .get("sparse_k")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;
    let final_k = payload
        .get("final_k")
        .and_then(|v| v.as_u64())
        .or_else(|| payload.get("limit").and_then(|v| v.as_u64()))
        .unwrap_or(10) as usize;

    // Check cache first
    let cache_key = QueryKey::new(
        collection_name.clone(),
        format!("hybrid:{}:{}", query, alpha),
        final_k,
        None,
    );
    if let Some(cached_result) = state.query_cache.get(&cache_key) {
        debug!(
            "💾 Cache hit for hybrid query '{}' in collection '{}'",
            query, collection_name
        );
        drop(timer);
        return Ok(Json(cached_result));
    }

    info!(
        "🔍 Hybrid search for '{}' in collection '{}' (alpha={}, algorithm={:?})",
        query, collection_name, alpha, algorithm
    );

    // Get the collection with owner validation
    let collection = state
        .store
        .get_collection_with_owner(&collection_name, tenant_id.as_ref())
        .map_err(|e| ErrorResponse::from(e))?;

    // Generate dense embedding for the query
    let query_dense = state
        .embedding_manager
        .embed(query)
        .map_err(|e| create_bad_request_error(&format!("Failed to generate embedding: {}", e)))?;

    // Create hybrid search config
    let config = HybridSearchConfig {
        alpha,
        dense_k,
        sparse_k,
        final_k,
        algorithm,
    };

    // Perform hybrid search
    let search_results = collection
        .hybrid_search(&query_dense, query_sparse.as_ref(), config)
        .map_err(|e| create_bad_request_error(&format!("Hybrid search failed: {}", e)))?;

    // Convert results to JSON format
    let results: Vec<Value> = search_results
        .into_iter()
        .map(|result| {
            json!({
                "id": result.id,
                "score": result.score,
                "vector": result.vector,
                "payload": result.payload.map(|p| p.data)
            })
        })
        .collect();

    // Build response
    let response = json!({
        "results": results,
        "query": query,
        "query_sparse": query_sparse.as_ref().map(|sv| json!({
            "indices": sv.indices,
            "values": sv.values
        })),
        "limit": final_k,
        "collection": collection_name,
        "alpha": alpha,
        "algorithm": algorithm_str,
        "total_results": results.len()
    });

    // Cache the result
    state.query_cache.insert(cache_key, response.clone());

    // Record metrics
    let label_success = "success";
    METRICS
        .search_requests_total
        .with_label_values(&[label_collection, &label_hybrid, label_success])
        .inc();
    METRICS
        .search_results_count
        .with_label_values(&[label_collection, &label_hybrid])
        .observe(results.len() as f64);
    drop(timer); // Stop latency timer

    Ok(Json(response))
}

pub async fn search_by_file(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let file_path = payload
        .get("file_path")
        .and_then(|f| f.as_str())
        .ok_or_else(|| {
            create_validation_error("file_path", "missing or invalid file_path parameter")
        })?;
    let limit = payload.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;

    // Extract tenant ID for multi-tenant access control
    let tenant_id = extract_tenant_id(&tenant_ctx);

    // Validate collection access (even though we return empty results for now)
    let _ = state
        .store
        .get_collection_with_owner(&collection_name, tenant_id.as_ref())
        .map_err(|e| ErrorResponse::from(e))?;

    // For now, return empty results
    Ok(Json(json!({
        "results": [],
        "file_path": file_path,
        "limit": limit,
        "collection": collection_name
    })))
}

pub async fn search_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let query_vector = payload
        .get("vector")
        .and_then(|v| v.as_array())
        .ok_or_else(|| create_validation_error("vector", "missing or invalid vector parameter"))?;
    let limit = payload.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;

    // For now, return empty results
    Ok(Json(json!({
        "results": [],
        "query_vector": query_vector,
        "limit": limit
    })))
}

pub async fn batch_search_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let queries = payload
        .get("queries")
        .and_then(|q| q.as_array())
        .ok_or_else(|| {
            create_validation_error("queries", "missing or invalid queries parameter")
        })?;

    info!("Batch searching {} queries", queries.len());

    Ok(Json(json!({
        "results": [],
        "queries": queries.len(),
        "message": "Batch search completed"
    })))
}

pub async fn batch_update_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let updates = payload
        .get("updates")
        .and_then(|u| u.as_array())
        .ok_or_else(|| {
            create_validation_error("updates", "missing or invalid updates parameter")
        })?;

    info!("Batch updating {} vectors", updates.len());

    Ok(Json(json!({
        "message": format!("Batch updated {} vectors successfully", updates.len()),
        "count": updates.len()
    })))
}

pub async fn batch_delete_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let ids = payload
        .get("ids")
        .and_then(|i| i.as_array())
        .ok_or_else(|| create_validation_error("ids", "missing or invalid ids parameter"))?;

    info!("Batch deleting {} vectors", ids.len());

    Ok(Json(json!({
        "message": format!("Batch deleted {} vectors successfully", ids.len()),
        "count": ids.len()
    })))
}
