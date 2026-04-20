//! Search REST handlers.
//!
//! - `search_vectors_by_text`  — POST /collections/{name}/search/text (embedding-backed)
//! - `hybrid_search_vectors`   — POST /collections/{name}/search/hybrid (dense + sparse)
//! - `search_by_file`          — POST /collections/{name}/search/file
//! - `search_vectors`          — POST /search (raw vector, returns empty results until wired)
//! - `batch_search_vectors`    — POST /batch/search
//! - `batch_update_vectors`    — POST /batch/update
//! - `batch_delete_vectors`    — POST /batch/delete

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]
// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use axum::Extension;
use axum::extract::{Path, State};
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, info};

use super::common::extract_tenant_id;
use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_validation_error,
};
use vectorizer::db::{HybridScoringAlgorithm, HybridSearchConfig};
use vectorizer::hub::middleware::RequestTenantContext;
use vectorizer::models::SparseVector;

pub async fn search_vectors_by_text(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::cache::query_cache::QueryKey;
    use vectorizer::monitoring::metrics::METRICS;

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
    use vectorizer::cache::query_cache::QueryKey;
    use vectorizer::monitoring::metrics::METRICS;

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

/// Core raw-vector search pipeline shared by `search_vectors` (POST
/// /search) and `search_vectors_by_collection` (POST
/// /collections/{name}/search).
///
/// Validates that the query vector's dimension matches the target
/// collection, consults the query cache (via `QueryKey::from_vector`),
/// runs the HNSW search, and records metrics under the `vector` label.
/// Returns the JSON response body.
async fn do_vector_search(
    state: &VectorizerServer,
    collection_name: &str,
    query_embedding: Vec<f32>,
    limit: usize,
    threshold: Option<f64>,
    tenant_ctx: Option<&Extension<RequestTenantContext>>,
) -> Result<Value, ErrorResponse> {
    use vectorizer::cache::query_cache::QueryKey;
    use vectorizer::monitoring::metrics::METRICS;

    let label_vector = "vector".to_string();
    let timer = METRICS
        .search_latency_seconds
        .with_label_values(&[collection_name, &label_vector])
        .start_timer();

    let cache_key = QueryKey::from_vector(
        collection_name.to_string(),
        &query_embedding,
        limit,
        threshold,
    );
    if let Some(cached) = state.query_cache.get(&cache_key) {
        debug!(
            "💾 Cache hit for raw-vector search in collection '{}'",
            collection_name
        );
        drop(timer);
        return Ok(cached);
    }

    let tenant_id = extract_tenant_id(&tenant_ctx.cloned());

    let collection = state
        .store
        .get_collection_with_owner(collection_name, tenant_id.as_ref())
        .map_err(ErrorResponse::from)?;

    if query_embedding.len() != collection.config().dimension {
        return Err(create_validation_error(
            "vector",
            &format!(
                "vector dimension {} does not match collection dimension {}",
                query_embedding.len(),
                collection.config().dimension
            ),
        ));
    }

    let search_results = collection
        .search(&query_embedding, limit)
        .map_err(|e| create_bad_request_error(&format!("Search failed: {}", e)))?;

    let results: Vec<Value> = search_results
        .into_iter()
        .filter(|r| threshold.is_none_or(|t| r.score as f64 >= t))
        .map(|result| {
            json!({
                "id": result.id,
                "score": result.score,
                "vector": result.vector,
                "payload": result.payload.map(|p| p.data)
            })
        })
        .collect();

    let response = json!({
        "results": results,
        "query_type": "vector",
        "limit": limit,
        "collection": collection_name,
        "total_results": results.len(),
    });

    state.query_cache.insert(cache_key, response.clone());

    METRICS
        .search_requests_total
        .with_label_values(&[collection_name, &label_vector, "success"])
        .inc();
    METRICS
        .search_results_count
        .with_label_values(&[collection_name, &label_vector])
        .observe(results.len() as f64);
    drop(timer);

    Ok(response)
}

/// Parse `vector`, `limit`, `threshold` from the request JSON. Returns
/// 400 when `vector` is missing, not an array, or contains non-float
/// entries.
fn parse_vector_search_payload(
    payload: &Value,
) -> Result<(Vec<f32>, usize, Option<f64>), ErrorResponse> {
    let raw = payload
        .get("vector")
        .and_then(|v| v.as_array())
        .ok_or_else(|| create_validation_error("vector", "missing or invalid vector parameter"))?;
    let mut query_vector = Vec::with_capacity(raw.len());
    for (idx, entry) in raw.iter().enumerate() {
        let f = entry.as_f64().ok_or_else(|| {
            create_validation_error("vector", &format!("vector[{}] is not a number", idx))
        })?;
        query_vector.push(f as f32);
    }
    let limit = payload.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;
    let threshold = payload.get("threshold").and_then(|t| t.as_f64());
    Ok((query_vector, limit, threshold))
}

/// POST /search — raw-vector similarity search. The target collection
/// is taken from the JSON body's `collection` field.
///
/// Request: `{collection, vector: [f32; dim], limit?, threshold?}`
/// Response: `{collection, limit, query_type: "vector", total_results,
/// results: [{id, score, vector, payload}]}`
pub async fn search_vectors(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?
        .to_string();

    let (query_vector, limit, threshold) = parse_vector_search_payload(&payload)?;

    let response = do_vector_search(
        &state,
        &collection_name,
        query_vector,
        limit,
        threshold,
        tenant_ctx.as_ref(),
    )
    .await?;

    Ok(Json(response))
}

/// POST /collections/{name}/search — raw-vector similarity search with
/// the collection supplied via URL path. Same request/response shape as
/// `search_vectors` minus the body-level `collection` field.
pub async fn search_vectors_by_collection(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let (query_vector, limit, threshold) = parse_vector_search_payload(&payload)?;

    let response = do_vector_search(
        &state,
        &collection_name,
        query_vector,
        limit,
        threshold,
        tenant_ctx.as_ref(),
    )
    .await?;

    Ok(Json(response))
}

/// POST /batch_search — run multiple searches against one collection.
///
/// Request: `{collection, queries: [{query?, vector?, limit?, threshold?}]}`
/// Each query may carry either a text `query` (embedded server-side via
/// the active `EmbeddingManager`) or a raw `vector` (validated against
/// the collection dimension). Per-query failures are captured in the
/// response without aborting the batch.
///
/// Response: `{collection, count, succeeded, failed, results: [{index,
/// query?, vector?, status: "ok"|"error", results?, total_results?, error?}]}`.
pub async fn batch_search_vectors(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?
        .to_string();

    let queries = payload
        .get("queries")
        .and_then(|q| q.as_array())
        .ok_or_else(|| create_validation_error("queries", "missing or invalid queries parameter"))?
        .clone();

    if queries.is_empty() {
        return Err(create_validation_error(
            "queries",
            "queries array must contain at least one entry",
        ));
    }

    info!(
        "Batch searching {} queries against '{}'",
        queries.len(),
        collection_name
    );

    let mut succeeded: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<Value> = Vec::with_capacity(queries.len());

    for (idx, entry) in queries.iter().enumerate() {
        let limit = entry.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        let threshold = entry.get("threshold").and_then(|v| v.as_f64());

        let outcome = if let Some(vec_arr) = entry.get("vector").and_then(|v| v.as_array()) {
            let mut query_vector = Vec::with_capacity(vec_arr.len());
            let mut bad_entry: Option<String> = None;
            for (i, v) in vec_arr.iter().enumerate() {
                match v.as_f64() {
                    Some(f) => query_vector.push(f as f32),
                    None => {
                        bad_entry = Some(format!("vector[{}] is not a number", i));
                        break;
                    }
                }
            }
            if let Some(msg) = bad_entry {
                Err(create_validation_error("vector", &msg))
            } else {
                do_vector_search(
                    &state,
                    &collection_name,
                    query_vector,
                    limit,
                    threshold,
                    tenant_ctx.as_ref(),
                )
                .await
            }
        } else if let Some(query) = entry.get("query").and_then(|q| q.as_str()) {
            match state.embedding_manager.embed(query) {
                Ok(embedding) => {
                    do_vector_search(
                        &state,
                        &collection_name,
                        embedding,
                        limit,
                        threshold,
                        tenant_ctx.as_ref(),
                    )
                    .await
                }
                Err(e) => Err(create_bad_request_error(&format!(
                    "Failed to embed query: {}",
                    e
                ))),
            }
        } else {
            Err(create_validation_error(
                "queries",
                &format!("entry[{}] missing both `query` and `vector`", idx),
            ))
        };

        match outcome {
            Ok(mut body) => {
                succeeded += 1;
                let hits = body
                    .get("results")
                    .and_then(|r| r.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                if let Some(obj) = body.as_object_mut() {
                    obj.insert("index".to_string(), json!(idx));
                    obj.insert("status".to_string(), json!("ok"));
                    obj.insert("total_results".to_string(), json!(hits));
                    obj.insert(
                        "query".to_string(),
                        entry.get("query").cloned().unwrap_or(Value::Null),
                    );
                }
                results.push(body);
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "status": "error",
                    "error": e.message.clone(),
                    "error_type": e.error_type.clone(),
                    "query": entry.get("query").cloned().unwrap_or(Value::Null),
                }));
            }
        }
    }

    Ok(Json(json!({
        "collection": collection_name,
        "count": queries.len(),
        "succeeded": succeeded,
        "failed": failed,
        "results": results,
    })))
}

/// POST /batch_update — update a vector's payload (and optionally its
/// dense data) in bulk.
///
/// Request: `{collection, updates: [{id, vector?, payload?}]}`. Each
/// entry either replaces the stored vector's `data` or its `payload`.
/// Per-entry failures are captured without aborting the batch. The
/// collection's query cache is invalidated once after the batch.
pub async fn batch_update_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::models::{Payload, Vector};

    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?
        .to_string();

    let updates = payload
        .get("updates")
        .and_then(|u| u.as_array())
        .ok_or_else(|| create_validation_error("updates", "missing or invalid updates parameter"))?
        .clone();

    if updates.is_empty() {
        return Err(create_validation_error(
            "updates",
            "updates array must contain at least one entry",
        ));
    }

    info!(
        "Batch updating {} vectors in '{}'",
        updates.len(),
        collection_name
    );

    // Capture collection dimension up front; do NOT keep the collection
    // handle live across `state.store.update` or we'll deadlock on the
    // write lock that update() needs.
    let collection_dim = {
        let c = state
            .store
            .get_collection(&collection_name)
            .map_err(ErrorResponse::from)?;
        c.config().dimension
    };

    let mut updated: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<Value> = Vec::with_capacity(updates.len());

    for (idx, entry) in updates.iter().enumerate() {
        let id = match entry.get("id").and_then(|i| i.as_str()) {
            Some(id) => id.to_string(),
            None => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "status": "error",
                    "error": "missing `id` field",
                }));
                continue;
            }
        };

        // Read the existing vector in its own scope so the collection
        // read reference is dropped before we call update().
        let existing = {
            match state.store.get_collection(&collection_name) {
                Ok(c) => match c.get_vector(&id) {
                    Ok(v) => v,
                    Err(e) => {
                        failed += 1;
                        results.push(json!({
                            "index": idx,
                            "id": id,
                            "status": "error",
                            "error": format!("{}", e),
                        }));
                        continue;
                    }
                },
                Err(e) => {
                    failed += 1;
                    results.push(json!({
                        "index": idx,
                        "id": id,
                        "status": "error",
                        "error": format!("{}", e),
                    }));
                    continue;
                }
            }
        };

        let new_data = match entry.get("vector").and_then(|v| v.as_array()) {
            Some(arr) => {
                let mut v = Vec::with_capacity(arr.len());
                let mut bad = false;
                for x in arr {
                    match x.as_f64() {
                        Some(f) => v.push(f as f32),
                        None => {
                            bad = true;
                            break;
                        }
                    }
                }
                if bad {
                    failed += 1;
                    results.push(json!({
                        "index": idx,
                        "id": id,
                        "status": "error",
                        "error": "vector entries must be numbers",
                    }));
                    continue;
                }
                if v.len() != collection_dim {
                    failed += 1;
                    results.push(json!({
                        "index": idx,
                        "id": id,
                        "status": "error",
                        "error": format!(
                            "vector dim {} != collection dim {}",
                            v.len(),
                            collection_dim
                        ),
                    }));
                    continue;
                }
                v
            }
            None => existing.data.clone(),
        };

        let new_payload = match entry.get("payload") {
            Some(p) if !p.is_null() => Some(Payload::new(p.clone())),
            Some(_) => None,
            None => existing.payload.clone(),
        };

        let updated_vector = Vector {
            id: id.clone(),
            data: new_data,
            sparse: existing.sparse.clone(),
            payload: new_payload,
            document_id: existing.document_id.clone(),
        };

        match state.store.update(&collection_name, updated_vector) {
            Ok(()) => {
                updated += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "ok",
                }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "error",
                    "error": format!("{}", e),
                }));
            }
        }
    }

    if updated > 0 {
        state.query_cache.invalidate_collection(&collection_name);
        if let Some(ref auto_save) = state.auto_save_manager {
            auto_save.mark_changed();
        }
    }

    Ok(Json(json!({
        "collection": collection_name,
        "count": updates.len(),
        "updated": updated,
        "failed": failed,
        "results": results,
    })))
}

/// POST /batch_delete — delete a list of vector ids from a single
/// collection.
///
/// Request: `{collection, ids: [string]}`. Per-id failures (e.g.
/// not-found) are captured without aborting the batch. The
/// collection's query cache is invalidated once after the batch.
pub async fn batch_delete_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?
        .to_string();

    let ids_value = payload
        .get("ids")
        .and_then(|i| i.as_array())
        .ok_or_else(|| create_validation_error("ids", "missing or invalid ids parameter"))?
        .clone();

    if ids_value.is_empty() {
        return Err(create_validation_error(
            "ids",
            "ids array must contain at least one entry",
        ));
    }

    info!(
        "Batch deleting {} vectors from '{}'",
        ids_value.len(),
        collection_name
    );

    let mut deleted: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<Value> = Vec::with_capacity(ids_value.len());

    for (idx, entry) in ids_value.iter().enumerate() {
        let id = match entry.as_str() {
            Some(s) => s.to_string(),
            None => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "status": "error",
                    "error": "id must be a string",
                }));
                continue;
            }
        };

        match state.store.delete(&collection_name, &id) {
            Ok(()) => {
                deleted += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "ok",
                }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "error",
                    "error": format!("{}", e),
                }));
            }
        }
    }

    if deleted > 0 {
        state.query_cache.invalidate_collection(&collection_name);
        if let Some(ref auto_save) = state.auto_save_manager {
            auto_save.mark_changed();
        }
    }

    Ok(Json(json!({
        "collection": collection_name,
        "count": ids_value.len(),
        "deleted": deleted,
        "failed": failed,
        "results": results,
    })))
}
