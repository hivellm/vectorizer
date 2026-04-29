//! `POST /insert_vectors` — bulk-insert pre-computed embeddings.
//!
//! Companion to `insert.rs`'s text-insertion pipeline. This endpoint
//! skips the embedding stage entirely; the request body carries the
//! vectors as raw `Vec<f32>`, the server only validates dimensions and
//! writes. Sized to its own file because it doesn't share much logic
//! with `insert_text` / `insert_one_text` beyond the post-write
//! housekeeping (`ensure_collection_exists`, `check_insert_quota`,
//! `record_insert_usage`, `mark_collection_dirty`), which are reused
//! verbatim from `super::insert`.

use axum::Extension;
use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::info;
use vectorizer::hub::middleware::RequestTenantContext;

use super::insert::{
    check_insert_quota, ensure_collection_exists, mark_collection_dirty, parse_metadata,
    record_insert_usage, validate_client_id,
};
use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_bad_request_error};

/// POST /insert_vectors — bulk-insert pre-computed embeddings with
/// caller-supplied vector ids. Skips the embedding pipeline entirely;
/// the request body carries the vectors as raw `Vec<f32>`. Useful when
/// the client owns its own embedder, needs deterministic ids for
/// idempotent re-ingest, or wants to upsert without going through the
/// chunk-and-embed path.
///
/// Request shape:
/// ```json
/// {
///   "collection": "docs",
///   "vectors": [
///     {
///       "id": "doc:1",                  // optional — falls back to UUID v4
///       "embedding": [0.1, 0.2, ...],   // required, length == collection.dimension
///       "payload": { ... },             // optional, arbitrary JSON
///       "metadata": { "k": "v", ... }   // optional fallback when `payload` absent
///     }
///   ],
///   "public_key": "..."                 // optional batch-level encryption
/// }
/// ```
///
/// Response shape mirrors `/insert_texts`: `{collection, inserted,
/// failed, count, results: [{index, client_id, status, vector_ids}]}`.
pub async fn insert_vectors(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::monitoring::metrics::METRICS;

    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "collection",
                "missing or invalid collection parameter",
            )
        })?
        .to_string();

    let vectors_in = payload
        .get("vectors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "vectors",
                "missing or invalid vectors parameter (expected an array)",
            )
        })?
        .clone();

    if vectors_in.is_empty() {
        return Err(crate::server::error_middleware::create_validation_error(
            "vectors",
            "vectors array must contain at least one entry",
        ));
    }

    let batch_public_key = payload
        .get("public_key")
        .and_then(|k| k.as_str())
        .map(str::to_string);

    info!(
        "insert_vectors: {} vector(s) into collection '{}'",
        vectors_in.len(),
        collection_name
    );

    ensure_collection_exists(&state, &collection_name)?;

    let collection_dim = state
        .store
        .get_collection(&collection_name)
        .map(|c| c.config().dimension)
        .map_err(ErrorResponse::from)?;

    check_insert_quota(&state, tenant_ctx.as_ref(), vectors_in.len()).await?;

    let mut results: Vec<Value> = Vec::with_capacity(vectors_in.len());
    let mut inserted_ids: Vec<String> = Vec::with_capacity(vectors_in.len());
    let mut inserted: usize = 0;
    let mut failed: usize = 0;
    let mut last_embedding_len = 0usize;
    let label_collection: &str = &collection_name;

    for (idx, entry) in vectors_in.iter().enumerate() {
        let timer = METRICS.insert_latency_seconds.start_timer();
        let outcome = insert_one_vector(
            &state,
            &collection_name,
            collection_dim,
            entry,
            batch_public_key.as_deref(),
        );
        drop(timer);

        match outcome {
            Ok((vector_id, embedding_len, client_id_echo)) => {
                inserted += 1;
                last_embedding_len = embedding_len;
                inserted_ids.push(vector_id.clone());
                METRICS
                    .insert_requests_total
                    .with_label_values(&[label_collection, "success"])
                    .inc();
                results.push(json!({
                    "index": idx,
                    "client_id": client_id_echo,
                    "status": "ok",
                    "vector_ids": [vector_id],
                }));
            }
            Err(e) => {
                failed += 1;
                METRICS
                    .insert_requests_total
                    .with_label_values(&[label_collection, "error"])
                    .inc();
                let client_id_echo = entry.get("id").and_then(|i| i.as_str()).map(str::to_string);
                results.push(json!({
                    "index": idx,
                    "client_id": client_id_echo,
                    "status": "error",
                    "error": e.message.clone(),
                    "error_type": e.error_type.clone(),
                }));
            }
        }
    }

    if !inserted_ids.is_empty() {
        record_insert_usage(
            &state,
            &collection_name,
            last_embedding_len,
            inserted_ids.len() as u64,
        )
        .await;
        mark_collection_dirty(&state, &collection_name, &inserted_ids);
    }

    info!(
        "insert_vectors into '{}' complete: {} inserted, {} failed",
        collection_name, inserted, failed
    );

    Ok(Json(json!({
        "collection": collection_name,
        "inserted": inserted,
        "failed": failed,
        "count": vectors_in.len(),
        "results": results,
    })))
}

/// Insert a single pre-vectorized entry. Returns `(vector_id,
/// embedding_len, client_id_echo)` on success.
fn insert_one_vector(
    state: &VectorizerServer,
    collection_name: &str,
    collection_dim: usize,
    entry: &Value,
    batch_public_key: Option<&str>,
) -> Result<(String, usize, Option<String>), ErrorResponse> {
    let client_id = entry.get("id").and_then(|i| i.as_str());
    if let Some(id) = client_id {
        validate_client_id(id).map_err(|reason| {
            crate::server::error_middleware::create_validation_error("id", &reason)
        })?;
    }
    let client_id_echo = client_id.map(str::to_string);

    let embedding_value = entry.get("embedding").ok_or_else(|| {
        crate::server::error_middleware::create_validation_error(
            "embedding",
            "missing or invalid embedding parameter (expected an array of f32)",
        )
    })?;

    let embedding_arr = embedding_value.as_array().ok_or_else(|| {
        crate::server::error_middleware::create_validation_error(
            "embedding",
            "embedding must be an array of f32",
        )
    })?;

    if embedding_arr.len() != collection_dim {
        return Err(crate::server::error_middleware::create_validation_error(
            "embedding",
            &format!(
                "embedding length {} does not match collection dimension {}",
                embedding_arr.len(),
                collection_dim
            ),
        ));
    }

    let mut embedding: Vec<f32> = Vec::with_capacity(embedding_arr.len());
    for (i, v) in embedding_arr.iter().enumerate() {
        let f = v.as_f64().ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "embedding",
                &format!("embedding[{}] is not a number", i),
            )
        })?;
        embedding.push(f as f32);
    }

    let payload_data = build_vector_payload(entry);

    let entry_public_key = entry
        .get("public_key")
        .and_then(|k| k.as_str())
        .or(batch_public_key);

    let payload = if let Some(key) = entry_public_key {
        let encrypted =
            vectorizer::security::payload_encryption::encrypt_payload(&payload_data, key)
                .map_err(|e| create_bad_request_error(&format!("Encryption failed: {}", e)))?;
        vectorizer::models::Payload::from_encrypted(encrypted)
    } else {
        vectorizer::models::Payload::new(payload_data)
    };

    let vector_id = client_id
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let embedding_len = embedding.len();
    let vector = vectorizer::models::Vector {
        id: vector_id.clone(),
        data: embedding,
        sparse: None,
        payload: Some(payload),
        document_id: None,
    };

    state
        .store
        .insert(collection_name, vec![vector])
        .map_err(ErrorResponse::from)?;

    Ok((vector_id, embedding_len, client_id_echo))
}

/// Build the payload Value for `/insert_vectors` from the request entry.
/// Prefers `payload` (free-form JSON) when present; otherwise falls back
/// to `metadata` (string→string map, mirroring `/insert_texts`); empty
/// object when neither is provided.
pub(super) fn build_vector_payload(entry: &Value) -> Value {
    if let Some(p) = entry.get("payload") {
        return p.clone();
    }
    let metadata = parse_metadata(entry);
    Value::Object(
        metadata
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect(),
    )
}
