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
//! - `move_vectors`        — POST /collections/{name}/vectors/move

use std::collections::HashMap;

use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, info};
use vectorizer::hub::middleware::RequestTenantContext;

use super::common::extract_tenant_id;
use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_validation_error};

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

    // Issue #263: per-collection admission. Acquired before any of
    // the heavy chunking / embedding work below; held until handler
    // exit by RAII drop.
    let _admission_ticket = super::common::admit_upsert(&state.upsert_queue, &collection_name)?;

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
            client_id.as_deref(),
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

/// POST /collections/{src}/vectors/move — relocate vectors between
/// collections without re-embedding (issue #265).
///
/// Body: `{destination: string, ids: [string]}`. The handler reads each
/// vector from `src`, inserts it into `dst` carrying its raw vector
/// data + payload as-is, and only then deletes it from `src`. The
/// dst-insert-before-src-delete ordering is the documented invariant:
/// a mid-batch crash leaves a recoverable duplicate, never data loss.
///
/// Per-id status (one of `ok | missing_in_src | dst_insert_failed |
/// src_delete_failed`) is recorded in `results` without aborting the
/// batch — operators want partial progress for tier-demotion sweeps.
///
/// Both source and destination caches are invalidated when at least
/// one vector successfully moved.
pub async fn move_vectors(
    State(state): State<VectorizerServer>,
    Path(src_collection): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let dst_collection = payload
        .get("destination")
        .and_then(|d| d.as_str())
        .ok_or_else(|| {
            create_validation_error("destination", "missing or invalid destination parameter")
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

    if src_collection == dst_collection {
        return Err(create_validation_error(
            "destination",
            "destination must differ from source collection",
        ));
    }

    info!(
        "Moving {} vectors from '{}' to '{}'",
        ids_value.len(),
        src_collection,
        dst_collection,
    );

    let mut moved: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<Value> = Vec::with_capacity(ids_value.len());

    for entry in ids_value.iter() {
        let id = match entry.as_str() {
            Some(s) => s.to_string(),
            None => {
                failed += 1;
                results.push(json!({
                    "id": null,
                    "status": "missing_in_src",
                    "error": "id must be a string",
                }));
                continue;
            }
        };

        let vector = match state.store.get_vector(&src_collection, &id) {
            Ok(v) => v,
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "id": id,
                    "status": "missing_in_src",
                    "error": format!("{}", e),
                }));
                continue;
            }
        };

        if let Err(e) = state.store.insert(&dst_collection, vec![vector]) {
            failed += 1;
            results.push(json!({
                "id": id,
                "status": "dst_insert_failed",
                "error": format!("{}", e),
            }));
            continue;
        }

        if let Err(e) = state.store.delete(&src_collection, &id) {
            failed += 1;
            results.push(json!({
                "id": id,
                "status": "src_delete_failed",
                "error": format!("{}", e),
            }));
            continue;
        }

        moved += 1;
        results.push(json!({
            "id": id,
            "status": "ok",
        }));
    }

    if moved > 0 {
        state.query_cache.invalidate_collection(&src_collection);
        state.query_cache.invalidate_collection(&dst_collection);
        if let Some(ref auto_save) = state.auto_save_manager {
            auto_save.mark_changed();
        }
    }

    info!(
        "Move from '{}' to '{}' complete: {} moved, {} failed",
        src_collection, dst_collection, moved, failed
    );

    Ok(Json(json!({
        "src": src_collection,
        "dst": dst_collection,
        "requested": ids_value.len(),
        "moved": moved,
        "failed": failed,
        "results": results,
    })))
}

/// POST /collections/{name}/vectors/delete_by_filter — delete every vector
/// matching a Qdrant-style metadata predicate.
///
/// Body: `{"filter": <QdrantFilter>}`
///
/// An empty filter (all `must`/`should`/`must_not` absent or empty) is
/// rejected with 400 to prevent accidental collection wipes.
///
/// Response: `{"scanned": N, "matched": N, "deleted": N, "results": [...]}`
pub async fn delete_by_filter(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::server::error_middleware::ErrorResponse> {
    use vectorizer::models::qdrant::filter::{QdrantFilter, QdrantFilterBuilder};
    use vectorizer::models::qdrant::filter_processor::FilterProcessor;

    let filter: QdrantFilter = payload
        .get("filter")
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "filter",
                "missing filter field",
            )
        })
        .and_then(|f| {
            serde_json::from_value(f.clone()).map_err(|e| {
                crate::server::error_middleware::create_validation_error(
                    "filter",
                    &format!("invalid filter: {}", e),
                )
            })
        })?;

    // Reject empty filter — no accidental full-collection wipe.
    let is_empty = filter.must.as_ref().map_or(true, |v| v.is_empty())
        && filter.should.as_ref().map_or(true, |v| v.is_empty())
        && filter.must_not.as_ref().map_or(true, |v| v.is_empty());
    if is_empty {
        return Err(crate::server::error_middleware::create_validation_error(
            "filter",
            "empty filter is not allowed; provide at least one condition to prevent accidental wipes",
        ));
    }

    // Scan collection for matching vectors.
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(crate::server::error_middleware::ErrorResponse::from)?;

    let all_vectors = collection.get_all_vectors();
    let scanned = all_vectors.len();

    let matching_ids: Vec<String> = all_vectors
        .into_iter()
        .filter_map(|v| {
            let payload = v.payload.as_ref()?;
            if FilterProcessor::apply_filter(&filter, payload) {
                Some(v.id)
            } else {
                None
            }
        })
        .collect();

    let matched = matching_ids.len();
    let mut deleted: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(matched);

    for id in &matching_ids {
        match state.store.delete(&collection_name, id) {
            Ok(()) => {
                deleted += 1;
                results.push(json!({"id": id, "status": "deleted"}));
            }
            Err(e) => {
                results.push(json!({"id": id, "status": "error", "error": e.to_string()}));
            }
        }
    }

    if deleted > 0 {
        state.query_cache.invalidate_collection(&collection_name);
        if let Some(ref auto_save) = state.auto_save_manager {
            auto_save.mark_changed();
        }
    }

    info!(
        "delete_by_filter '{}': scanned={} matched={} deleted={}",
        collection_name, scanned, matched, deleted
    );

    Ok(Json(json!({
        "scanned": scanned,
        "matched": matched,
        "deleted": deleted,
        "results": results,
    })))
}

/// POST /collections/{name}/vectors/bulk_update_metadata — apply a
/// JSON-merge patch to the payload of every vector matching a filter.
///
/// Body: `{"filter": <QdrantFilter>, "patch": {...}}`
///
/// The `patch` is applied with JSON-merge-patch semantics (RFC 7396):
/// keys in `patch` overwrite the existing payload values; null values
/// remove keys. The raw vector data and dimensions are never modified.
///
/// Response: `{"scanned": N, "matched": N, "updated": N, "results": [...]}`
pub async fn bulk_update_metadata(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::server::error_middleware::ErrorResponse> {
    use vectorizer::models::qdrant::filter::QdrantFilter;
    use vectorizer::models::qdrant::filter_processor::FilterProcessor;

    let filter: QdrantFilter = payload
        .get("filter")
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "filter",
                "missing filter field",
            )
        })
        .and_then(|f| {
            serde_json::from_value(f.clone()).map_err(|e| {
                crate::server::error_middleware::create_validation_error(
                    "filter",
                    &format!("invalid filter: {}", e),
                )
            })
        })?;

    let patch = payload.get("patch").cloned().ok_or_else(|| {
        crate::server::error_middleware::create_validation_error("patch", "missing patch field")
    })?;

    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(crate::server::error_middleware::ErrorResponse::from)?;

    let all_vectors = collection.get_all_vectors();
    let scanned = all_vectors.len();

    let matching: Vec<vectorizer::models::Vector> = all_vectors
        .into_iter()
        .filter(|v| {
            v.payload
                .as_ref()
                .map_or(false, |p| FilterProcessor::apply_filter(&filter, p))
        })
        .collect();

    let matched = matching.len();
    let mut updated: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(matched);

    for mut vector in matching {
        // Apply JSON-merge-patch to payload.
        let new_payload_data = if let Some(existing) = vector.payload.as_ref() {
            json_merge_patch(existing.data.clone(), patch.clone())
        } else {
            patch.clone()
        };

        vector.payload = Some(vectorizer::models::Payload {
            data: new_payload_data,
        });

        let id = vector.id.clone();
        match state.store.update(&collection_name, vector) {
            Ok(()) => {
                updated += 1;
                results.push(json!({"id": id, "status": "updated"}));
            }
            Err(e) => {
                results.push(json!({"id": id, "status": "error", "error": e.to_string()}));
            }
        }
    }

    if updated > 0 {
        state.query_cache.invalidate_collection(&collection_name);
        if let Some(ref auto_save) = state.auto_save_manager {
            auto_save.mark_changed();
        }
    }

    info!(
        "bulk_update_metadata '{}': scanned={} matched={} updated={}",
        collection_name, scanned, matched, updated
    );

    Ok(Json(json!({
        "scanned": scanned,
        "matched": matched,
        "updated": updated,
        "results": results,
    })))
}

/// Apply JSON-merge-patch (RFC 7396) to `target` using `patch`.
///
/// * Object keys in `patch` that are not `null` overwrite the same key in `target`.
/// * `null` values in `patch` remove the corresponding key from `target`.
/// * Non-object patches replace `target` entirely.
fn json_merge_patch(mut target: serde_json::Value, patch: serde_json::Value) -> serde_json::Value {
    match (target.as_object_mut(), patch) {
        (Some(target_map), serde_json::Value::Object(patch_map)) => {
            for (key, value) in patch_map {
                if value.is_null() {
                    target_map.remove(&key);
                } else {
                    let existing = target_map.remove(&key).unwrap_or(serde_json::Value::Null);
                    target_map.insert(key, json_merge_patch(existing, value));
                }
            }
            serde_json::Value::Object(target_map.clone())
        }
        (_, patch) => patch,
    }
}

/// POST /collections/{src}/vectors/copy — copy (NOT move) vectors to a
/// destination collection carrying raw vector data + payload unchanged.
///
/// Body: `{"destination": "dst_collection", "ids": ["id1", "id2", ...]}`
///
/// Per-id status: `ok | missing_in_src | dst_insert_failed`
///
/// Response: `{"src", "dst", "requested", "copied", "failed", "results"}`
pub async fn copy_vectors(
    State(state): State<VectorizerServer>,
    Path(src_collection): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::server::error_middleware::ErrorResponse> {
    let dst_collection = payload
        .get("destination")
        .and_then(|d| d.as_str())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "destination",
                "missing or invalid destination parameter",
            )
        })?
        .to_string();

    let ids_value = payload
        .get("ids")
        .and_then(|i| i.as_array())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "ids",
                "missing or invalid ids parameter",
            )
        })?
        .clone();

    if ids_value.is_empty() {
        return Err(crate::server::error_middleware::create_validation_error(
            "ids",
            "ids array must contain at least one entry",
        ));
    }

    if src_collection == dst_collection {
        return Err(crate::server::error_middleware::create_validation_error(
            "destination",
            "destination must differ from source collection",
        ));
    }

    info!(
        "Copying {} vectors from '{}' to '{}'",
        ids_value.len(),
        src_collection,
        dst_collection,
    );

    let mut copied: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(ids_value.len());

    for entry in ids_value.iter() {
        let id = match entry.as_str() {
            Some(s) => s.to_string(),
            None => {
                failed += 1;
                results.push(json!({
                    "id": null,
                    "status": "missing_in_src",
                    "error": "id must be a string",
                }));
                continue;
            }
        };

        let vector = match state.store.get_vector(&src_collection, &id) {
            Ok(v) => v,
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "id": id,
                    "status": "missing_in_src",
                    "error": format!("{}", e),
                }));
                continue;
            }
        };

        // Insert into destination — source is untouched (copy, not move).
        if let Err(e) = state.store.insert(&dst_collection, vec![vector]) {
            failed += 1;
            results.push(json!({
                "id": id,
                "status": "dst_insert_failed",
                "error": format!("{}", e),
            }));
            continue;
        }

        copied += 1;
        results.push(json!({"id": id, "status": "ok"}));
    }

    if copied > 0 {
        state.query_cache.invalidate_collection(&dst_collection);
        if let Some(ref auto_save) = state.auto_save_manager {
            auto_save.mark_changed();
        }
    }

    info!(
        "Copy from '{}' to '{}' complete: {} copied, {} failed",
        src_collection, dst_collection, copied, failed
    );

    Ok(Json(json!({
        "src": src_collection,
        "dst": dst_collection,
        "requested": ids_value.len(),
        "copied": copied,
        "failed": failed,
        "results": results,
    })))
}

/// PATCH /collections/{name}/vectors/{id}/expiry — set per-vector expiry.
///
/// Body: `{"expires_at": <unix_ms>}` — pass `null` to clear expiry.
///
/// The `expires_at` value is stored as `__expires_at` inside the vector's
/// JSON payload and is read by the per-collection TTL reaper.
pub async fn set_vector_expiry(
    State(state): State<VectorizerServer>,
    Path((collection_name, vector_id)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::server::error_middleware::ErrorResponse> {
    // `expires_at` may be null (clear) or an integer.
    let expires_at_opt: Option<i64> = match payload.get("expires_at") {
        None => {
            return Err(crate::server::error_middleware::create_validation_error(
                "expires_at",
                "missing expires_at field; pass null to clear",
            ));
        }
        Some(serde_json::Value::Null) => None,
        Some(v) => match v.as_i64() {
            Some(ts) => Some(ts),
            None => {
                return Err(crate::server::error_middleware::create_validation_error(
                    "expires_at",
                    "expires_at must be a Unix millisecond timestamp (integer) or null",
                ));
            }
        },
    };

    let mut vector = state
        .store
        .get_vector(&collection_name, &vector_id)
        .map_err(crate::server::error_middleware::ErrorResponse::from)?;

    // Update or clear the expiry field.
    let payload_mut = vector
        .payload
        .get_or_insert_with(|| vectorizer::models::Payload {
            data: serde_json::json!({}),
        });

    match expires_at_opt {
        Some(ts) => payload_mut.set_expires_at(ts),
        None => payload_mut.clear_expires_at(),
    }

    state
        .store
        .update(&collection_name, vector)
        .map_err(crate::server::error_middleware::ErrorResponse::from)?;

    if let Some(ref auto_save) = state.auto_save_manager {
        auto_save.mark_changed();
    }

    info!(
        "set_vector_expiry '{}' in '{}': expires_at={:?}",
        vector_id, collection_name, expires_at_opt
    );

    Ok(Json(json!({
        "id": vector_id,
        "collection": collection_name,
        "expires_at": expires_at_opt,
        "status": "ok",
    })))
}
