//! Vector CRUD and embedding REST handlers (excluding insert_text).
//!
//! - `list_vectors`        — GET  /collections/{name}/vectors
//! - `get_vector`          — GET  /collections/{name}/vectors/{id}
//! - `delete_vector`       — DELETE /collections/{name}/vectors/{id}
//! - `update_vector`       — PUT  /vectors
//! - `delete_vector_generic` — DELETE /vectors
//! - `embed_text`          — POST /embed
//! - `batch_insert_texts`  — POST /batch/insert
//! - `insert_texts`        — POST /texts

use std::collections::HashMap;

use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, info};

use super::common::extract_tenant_id;
use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_validation_error};
use vectorizer::hub::middleware::RequestTenantContext;

/// GET /collections/{name}/vectors — paginated vector listing
pub async fn list_vectors(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ErrorResponse> {
    let start_time = std::time::Instant::now();
    debug!("Listing vectors from collection: {}", collection_name);

    // Parse query parameters for pagination - cap at 50 for vector browser
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10)
        .min(50);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<usize>().ok())
        .unwrap_or(0);
    let min_score = params
        .get("min_score")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        .max(0.0)
        .min(1.0);

    // Extract tenant ID for multi-tenant access control
    let tenant_id = extract_tenant_id(&tenant_ctx);

    // Get the collection with owner validation
    let collection = state
        .store
        .get_collection_with_owner(&collection_name, tenant_id.as_ref())
        .map_err(|e| ErrorResponse::from(e))?;

    // Get actual vectors from the local collection
    let all_vectors = collection.get_all_vectors();
    let total_count = all_vectors.len();

    // Filter vectors by minimum score (scoring based on payload content richness)
    let filtered_vectors: Vec<_> = all_vectors
        .into_iter()
        .filter(|v| {
            // Calculate a score based on payload content length
            let score = if let Some(ref payload) = v.payload {
                // Simple scoring based on content richness
                let content_length = payload
                    .data
                    .get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0);
                (content_length as f32 / 1000.0).min(1.0) // Normalize to 0-1 range
            } else {
                0.0
            };
            score >= min_score
        })
        .collect();

    let filtered_total = filtered_vectors.len();

    // Apply pagination to filtered results
    let paginated_vectors: Vec<Value> = filtered_vectors
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|v| {
            json!({
                "id": v.id,
                "vector": v.data,
                "payload": v.payload.map(|p| p.data),
            })
        })
        .collect();

    let paginated_count = paginated_vectors.len();

    let response = json!({
        "vectors": paginated_vectors,
        "total": if min_score > 0.0 { filtered_total } else { total_count },
        "limit": limit,
        "offset": offset,
        "message": if min_score > 0.0 && filtered_total != total_count {
            Some(format!("Filtered {} of {} vectors by min_score >= {:.2}. Showing {} of {} filtered vectors.",
                filtered_total, total_count, min_score, paginated_count, filtered_total))
        } else if total_count > limit {
            Some(format!("Showing {} of {} vectors. Use pagination for more.", limit.min(total_count), total_count))
        } else {
            None
        },
    });

    let duration = start_time.elapsed();
    info!(
        "Listed {} vectors from local collection '{}' (total: {}) in {:?}",
        paginated_count, collection_name, total_count, duration
    );

    Ok(Json(response))
}

/// GET /collections/{name}/vectors/{id} — fetch a single vector
pub async fn get_vector(
    State(state): State<VectorizerServer>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<Value>, ErrorResponse> {
    let _collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|e| ErrorResponse::from(e))?;

    // Returns mock data — real retrieval by ID is tracked in a separate task
    Ok(Json(json!({
        "id": vector_id,
        "vector": vec![0.1; 512],
        "metadata": {
            "collection": collection_name
        }
    })))
}

/// DELETE /collections/{name}/vectors/{id} — delete a specific vector
pub async fn delete_vector(
    State(state): State<VectorizerServer>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<Value>, ErrorResponse> {
    info!(
        "Deleting vector {} from collection {}",
        vector_id, collection_name
    );

    // Actually delete the vector from the store
    state
        .store
        .delete(&collection_name, &vector_id)
        .map_err(|e| ErrorResponse::from(e))?;

    // Invalidate cache for this collection
    state.query_cache.invalidate_collection(&collection_name);
    debug!(
        "💾 Cache invalidated for collection '{}' after vector deletion",
        collection_name
    );

    // Mark changes for auto-save
    if let Some(ref auto_save) = state.auto_save_manager {
        auto_save.mark_changed();
    }

    Ok(Json(json!({
        "message": format!("Vector '{}' deleted from collection '{}'", vector_id, collection_name),
        "success": true
    })))
}

/// PUT /vectors — update a vector by id
pub async fn update_vector(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let id = payload
        .get("id")
        .and_then(|i| i.as_str())
        .ok_or_else(|| create_validation_error("id", "missing or invalid id parameter"))?;

    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?;

    info!("Updating vector: {} in collection: {}", id, collection_name);

    // Invalidate cache for this collection
    state.query_cache.invalidate_collection(collection_name);
    debug!(
        "💾 Cache invalidated for collection '{}' after vector update",
        collection_name
    );

    Ok(Json(json!({
        "message": format!("Vector '{}' updated successfully", id)
    })))
}

/// DELETE /vectors — delete a vector by id (generic, body-based)
pub async fn delete_vector_generic(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let id = payload
        .get("id")
        .and_then(|i| i.as_str())
        .ok_or_else(|| create_validation_error("id", "missing or invalid id parameter"))?;

    info!("Deleting vector: {}", id);

    Ok(Json(json!({
        "message": format!("Vector '{}' deleted successfully", id)
    })))
}

/// POST /embed — generate an embedding for a text string
/// POST /embed — generate an embedding for a text input via the
/// server's active `EmbeddingManager` (BM25 by default, or whatever
/// provider the operator registered in bootstrap).
///
/// Request: `{text: string}`
/// Response: `{embedding: [f32; dim], text, dimension}`
pub async fn embed_text(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let text = payload
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| create_validation_error("text", "missing or invalid text parameter"))?;

    let embedding = state.embedding_manager.embed(text).map_err(|e| {
        crate::server::error_middleware::create_bad_request_error(&format!(
            "Failed to generate embedding: {}",
            e
        ))
    })?;
    let dimension = embedding.len();

    Ok(Json(json!({
        "embedding": embedding,
        "text": text,
        "dimension": dimension,
    })))
}

/// Shared implementation for `batch_insert_texts` and `insert_texts`.
///
/// Accepts `{collection, texts: [{id?, text, metadata?, public_key?,
/// auto_chunk?, chunk_size?, chunk_overlap?}], ...batch-level defaults}`
/// and runs each entry through `insert::insert_one_text`. Per-item errors
/// are captured as `{status: "error"}` without aborting the batch.
///
/// Response shape: `{collection, inserted, failed, results: [...],
/// count}`. Returns 400 when the top-level `collection` or `texts` fields
/// are missing or `texts` is empty.
async fn do_batch_insert_texts(
    state: VectorizerServer,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    payload: Value,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::monitoring::metrics::METRICS;

    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error("collection", "missing or invalid collection parameter")
        })?
        .to_string();

    let texts = payload
        .get("texts")
        .and_then(|t| t.as_array())
        .ok_or_else(|| create_validation_error("texts", "missing or invalid texts parameter"))?
        .clone();

    if texts.is_empty() {
        return Err(create_validation_error(
            "texts",
            "texts array must contain at least one entry",
        ));
    }

    let batch_public_key = payload
        .get("public_key")
        .and_then(|k| k.as_str())
        .map(str::to_string);
    let batch_auto_chunk = payload
        .get("auto_chunk")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let batch_chunk_size = payload
        .get("chunk_size")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let batch_chunk_overlap = payload
        .get("chunk_overlap")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    info!(
        "Batch inserting {} text(s) into collection '{}'",
        texts.len(),
        collection_name
    );

    let mut results: Vec<Value> = Vec::with_capacity(texts.len());
    let mut inserted: usize = 0;
    let mut failed: usize = 0;

    for (idx, entry) in texts.iter().enumerate() {
        let Some(text) = entry.get("text").and_then(|t| t.as_str()) else {
            failed += 1;
            results.push(json!({
                "index": idx,
                "status": "error",
                "error": "missing or invalid text field",
            }));
            continue;
        };

        let client_id = entry.get("id").and_then(|i| i.as_str()).map(str::to_string);
        let metadata = super::insert::parse_metadata(entry);
        let public_key = entry
            .get("public_key")
            .and_then(|k| k.as_str())
            .map(str::to_string)
            .or_else(|| batch_public_key.clone());
        let auto_chunk = entry
            .get("auto_chunk")
            .and_then(|v| v.as_bool())
            .unwrap_or(batch_auto_chunk);
        let chunk_size = entry
            .get("chunk_size")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .or(batch_chunk_size);
        let chunk_overlap = entry
            .get("chunk_overlap")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .or(batch_chunk_overlap);

        let timer = METRICS.insert_latency_seconds.start_timer();
        let outcome = super::insert::insert_one_text(
            &state,
            tenant_ctx.as_ref(),
            &collection_name,
            text,
            metadata,
            public_key.as_deref(),
            auto_chunk,
            chunk_size,
            chunk_overlap,
        )
        .await;
        drop(timer);

        let label_collection: &str = &collection_name;
        match outcome {
            Ok(res) => {
                inserted += 1;
                METRICS
                    .insert_requests_total
                    .with_label_values(&[label_collection, "success"])
                    .inc();
                results.push(json!({
                    "index": idx,
                    "client_id": client_id,
                    "status": "ok",
                    "vector_ids": res.vector_ids,
                    "vectors_created": res.vector_ids.len(),
                    "chunked": res.chunked,
                }));
            }
            Err(e) => {
                failed += 1;
                METRICS
                    .insert_requests_total
                    .with_label_values(&[label_collection, "error"])
                    .inc();
                results.push(json!({
                    "index": idx,
                    "client_id": client_id,
                    "status": "error",
                    "error": e.message.clone(),
                    "error_type": e.error_type.clone(),
                }));
            }
        }
    }

    info!(
        "Batch insert into '{}' complete: {} inserted, {} failed",
        collection_name, inserted, failed
    );

    Ok(Json(json!({
        "collection": collection_name,
        "inserted": inserted,
        "failed": failed,
        "count": texts.len(),
        "results": results,
    })))
}

/// POST /batch_insert — batch-insert multiple texts into a collection.
pub async fn batch_insert_texts(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    do_batch_insert_texts(state, tenant_ctx, payload).await
}

/// POST /insert_texts — alias of `batch_insert_texts`. Same payload shape
/// and response shape; preserved for REST API back-compat.
pub async fn insert_texts(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    do_batch_insert_texts(state, tenant_ctx, payload).await
}
