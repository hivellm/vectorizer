//! Single-document text insertion handler + the shared write pipeline that
//! the batch endpoints in `super::vectors` reuse.
//!
//! `insert_text` is the POST /insert handler. Its core chunk+embed+insert
//! body lives in `pub(super) async fn insert_one_text` so the batch
//! endpoints (`batch_insert_texts`, `insert_texts`) can drive the exact
//! same write path without duplicating the pipeline.

use std::collections::HashMap;
use std::path::PathBuf;

use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, info, warn};
use vectorizer::config::FileUploadConfig;
use vectorizer::file_loader::chunker::Chunker;
use vectorizer::file_loader::config::LoaderConfig;
use vectorizer::hub::middleware::RequestTenantContext;
use vectorizer_core::error::VectorizerError;

use super::common::collection_metrics_uuid;
use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_bad_request_error};

/// Outcome of a single text insert — the vector ids that were created plus
/// whether the input was chunked.
#[derive(Debug)]
pub(super) struct InsertOneResult {
    pub vector_ids: Vec<String>,
    pub chunked: bool,
}

/// Maximum length of a client-provided vector id. Chosen to leave room for
/// the `#<chunk_index>` suffix and stay well under any sane key/index size.
pub(super) const MAX_CLIENT_ID_LEN: usize = 256;

/// Separator used when deriving chunk vector ids from a client-provided
/// parent id (e.g. `doc:42#0`, `doc:42#1`, ...). Reserved — client ids may
/// not contain it.
pub(super) const CLIENT_ID_CHUNK_SEPARATOR: char = '#';

/// Validate a client-provided vector id. Returns `Ok(())` when the id is
/// suitable for use as `Vector.id` (and as a parent-id prefix for chunks),
/// otherwise an `Err` with a human-readable reason.
///
/// Rules:
/// - non-empty
/// - no leading or trailing whitespace
/// - does not contain the chunk separator (`#`) — reserved for `parent#N`
/// - length within [`MAX_CLIENT_ID_LEN`]
pub(super) fn validate_client_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("client id must not be empty".to_string());
    }
    if id.len() > MAX_CLIENT_ID_LEN {
        return Err(format!(
            "client id must be at most {} characters (got {})",
            MAX_CLIENT_ID_LEN,
            id.len()
        ));
    }
    if id.trim() != id {
        return Err("client id must not have leading or trailing whitespace".to_string());
    }
    if id.contains(CLIENT_ID_CHUNK_SEPARATOR) {
        return Err(format!(
            "client id must not contain '{}' (reserved as chunk separator)",
            CLIENT_ID_CHUNK_SEPARATOR
        ));
    }
    Ok(())
}

/// Build the flat chunk payload emitted by the chunked write path.
/// Server-provided keys (`content`, `file_path`, `chunk_index`,
/// `parent_id`) take precedence over any colliding keys in user
/// metadata. Extracted as a pure function so the contract can be
/// unit-tested without spinning up a server.
pub(super) fn build_chunk_payload(
    content: &str,
    file_path: &str,
    chunk_index: usize,
    parent_id: &str,
    user_metadata: &HashMap<String, String>,
) -> Value {
    let mut payload_map = serde_json::Map::new();
    payload_map.insert("content".into(), json!(content));
    payload_map.insert("file_path".into(), json!(file_path));
    payload_map.insert("chunk_index".into(), json!(chunk_index));
    payload_map.insert("parent_id".into(), json!(parent_id));
    for (k, v) in user_metadata {
        if !payload_map.contains_key(k) {
            payload_map.insert(k.clone(), json!(v));
        }
    }
    Value::Object(payload_map)
}

/// Parse the optional `metadata` object from a request payload into a
/// `HashMap<String, String>`. Non-string values are stringified via
/// `serde_json::Value::to_string`.
pub(super) fn parse_metadata(payload: &Value) -> HashMap<String, String> {
    payload
        .get("metadata")
        .and_then(|m| m.as_object())
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        },
                    )
                })
                .collect::<HashMap<String, String>>()
        })
        .unwrap_or_default()
}

/// Ensure a collection exists, creating it with the project's default
/// configuration (pulled from `config.yml` when present, otherwise
/// `CollectionConfig::default()`) if missing.
pub(super) fn ensure_collection_exists(
    state: &VectorizerServer,
    name: &str,
) -> Result<(), ErrorResponse> {
    match state.store.get_collection(name) {
        Ok(_) => Ok(()),
        Err(VectorizerError::CollectionNotFound(_)) => {
            info!(
                "Collection '{}' not found, creating automatically with default config",
                name
            );
            let cfg = load_default_collection_config();
            state
                .store
                .create_collection(name, cfg)
                .map_err(ErrorResponse::from)?;
            info!(
                "Collection '{}' created successfully with auto-creation",
                name
            );
            Ok(())
        }
        Err(e) => Err(ErrorResponse::from(e)),
    }
}

/// Load `CollectionConfig` defaults from `config.yml`, falling back to
/// `CollectionConfig::default()` on any parse or IO failure.
fn load_default_collection_config() -> vectorizer::models::CollectionConfig {
    let Ok(config_content) = std::fs::read_to_string("config.yml") else {
        return vectorizer::models::CollectionConfig::default();
    };
    let Ok(yaml_value) = serde_yaml::from_str::<serde_json::Value>(&config_content) else {
        return vectorizer::models::CollectionConfig::default();
    };
    let Some(defaults) = yaml_value
        .get("collections")
        .and_then(|c| c.get("defaults"))
    else {
        return vectorizer::models::CollectionConfig::default();
    };

    let dimension = defaults
        .get("dimension")
        .and_then(|d| d.as_u64())
        .map(|d| d as usize)
        .unwrap_or(512);

    let metric = defaults
        .get("metric")
        .and_then(|m| m.as_str())
        .unwrap_or("cosine");

    let metric_enum = match metric {
        "cosine" => vectorizer::models::DistanceMetric::Cosine,
        "euclidean" => vectorizer::models::DistanceMetric::Euclidean,
        "dot" => vectorizer::models::DistanceMetric::DotProduct,
        _ => vectorizer::models::DistanceMetric::Cosine,
    };

    let hnsw_config = defaults
        .get("index")
        .and_then(|i| i.get("hnsw"))
        .map(|hnsw| vectorizer::models::HnswConfig {
            m: hnsw
                .get("m")
                .and_then(|m| m.as_u64())
                .map(|m| m as usize)
                .unwrap_or(16),
            ef_construction: hnsw
                .get("ef_construction")
                .and_then(|e| e.as_u64())
                .map(|e| e as usize)
                .unwrap_or(200),
            ef_search: hnsw
                .get("ef_search")
                .and_then(|e| e.as_u64())
                .map(|e| e as usize)
                .unwrap_or(64),
            seed: None,
        })
        .unwrap_or_default();

    let quantization = match defaults
        .get("quantization")
        .and_then(|q| q.get("type"))
        .and_then(|t| t.as_str())
    {
        Some("sq") => {
            let bits = defaults
                .get("quantization")
                .and_then(|q| q.get("sq"))
                .and_then(|s| s.get("bits"))
                .and_then(|b| b.as_u64())
                .map(|b| b as usize)
                .unwrap_or(8);
            vectorizer::models::QuantizationConfig::SQ { bits }
        }
        Some(_) => vectorizer::models::QuantizationConfig::None,
        None => vectorizer::models::QuantizationConfig::SQ { bits: 8 },
    };

    vectorizer::models::CollectionConfig {
        dimension,
        metric: metric_enum,
        hnsw_config,
        quantization,
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
        graph: None,
        encryption: None,
    }
}

/// HiveHub quota gate for vector insertion. Returns `Ok(())` when either
/// HiveHub is disabled, the quota check passes, or the hub call itself
/// errors (we log + allow to match the existing `insert_text` behavior).
/// Returns `Err(QUOTA_EXCEEDED)` only when HiveHub explicitly denies.
pub(super) async fn check_insert_quota(
    state: &VectorizerServer,
    tenant_ctx: Option<&Extension<RequestTenantContext>>,
    estimated_vectors: usize,
) -> Result<(), ErrorResponse> {
    let Some(hub_manager) = state.hub_manager.as_ref() else {
        return Ok(());
    };
    let tenant_id = tenant_ctx
        .map(|ctx| ctx.0.0.tenant_id.as_str())
        .unwrap_or("default");
    match hub_manager
        .check_quota(
            tenant_id,
            vectorizer::hub::QuotaType::VectorCount,
            estimated_vectors as u64,
        )
        .await
    {
        Ok(true) => Ok(()),
        Ok(false) => {
            warn!(
                "Vector insertion denied for tenant {}: quota exceeded (estimated {} vectors)",
                tenant_id, estimated_vectors
            );
            Err(ErrorResponse::new(
                "QUOTA_EXCEEDED".to_string(),
                format!(
                    "Vector quota exceeded. Estimated {} vectors needed. Please upgrade your plan or delete unused vectors.",
                    estimated_vectors
                ),
                StatusCode::TOO_MANY_REQUESTS,
            ))
        }
        Err(e) => {
            warn!("Failed to check quota with HiveHub: {}", e);
            Ok(())
        }
    }
}

/// Record a successful vector-insert against the HiveHub usage counters.
/// No-op when HiveHub is disabled.
pub(super) async fn record_insert_usage(
    state: &VectorizerServer,
    collection_name: &str,
    embedding_len: usize,
    inserted: u64,
) {
    let Some(hub_manager) = state.hub_manager.as_ref() else {
        return;
    };
    let mut metrics = vectorizer::hub::UsageMetrics::new();
    let estimated_storage = (embedding_len * 4) as u64 * inserted;
    metrics.record_insert(inserted, estimated_storage);
    let collection_id = collection_metrics_uuid(collection_name);
    if let Err(e) = hub_manager.record_usage(collection_id, metrics).await {
        warn!("Failed to record vector insertion usage: {}", e);
    }
}

/// Mark a collection dirty for auto-save, invalidate its query cache, and
/// forward Raft replication for the given vector ids. Meant to run once
/// per request after all vectors are committed.
pub(super) fn mark_collection_dirty(
    state: &VectorizerServer,
    collection_name: &str,
    vector_ids: &[String],
) {
    if let Some(ref auto_save) = state.auto_save_manager {
        auto_save.mark_changed();
    }

    state.query_cache.invalidate_collection(collection_name);
    debug!(
        "💾 Cache invalidated for collection '{}' after insert",
        collection_name
    );

    if vector_ids.is_empty() {
        return;
    }

    let active_master: Option<std::sync::Arc<vectorizer::replication::MasterNode>> = state
        .master_node
        .clone()
        .or_else(|| state.ha_manager.as_ref().and_then(|ha| ha.master_node()));
    if let Some(ref master) = active_master {
        if let Ok(col) = state.store.get_collection(collection_name) {
            for vid in vector_ids {
                if let Ok(v) = col.get_vector(vid) {
                    let payload_bytes = v.payload.as_ref().and_then(|p| serde_json::to_vec(p).ok());
                    let op = vectorizer::replication::VectorOperation::InsertVector {
                        collection: collection_name.to_string(),
                        id: vid.clone(),
                        vector: v.data.clone(),
                        payload: payload_bytes,
                        owner_id: None,
                    };
                    master.replicate(op);
                }
            }
            debug!(
                "Replicated {} vectors for collection '{}'",
                vector_ids.len(),
                collection_name
            );
        }
    }
}

/// Core write path: chunk + embed + insert a single text into the target
/// collection. Ensures the collection exists (auto-creating with defaults
/// when missing), enforces the HiveHub quota for the estimated vector
/// count, records usage metrics, marks auto-save dirty, invalidates the
/// query cache, and forwards Raft replication.
///
/// `client_id`, when provided, becomes the `Vector.id` for non-chunked
/// inputs and the prefix for `<id>#<chunk_index>` chunk ids when the input
/// is auto-chunked. It is also stored as `parent_id` on chunk payloads so
/// chunks can be grouped or deleted by source document. When absent, the
/// server falls back to a UUID v4 per vector and uses a single shared UUID
/// as `parent_id` for the chunk group.
///
/// Returns the new vector ids + whether the text was chunked.
pub(super) async fn insert_one_text(
    state: &VectorizerServer,
    tenant_ctx: Option<&Extension<RequestTenantContext>>,
    collection_name: &str,
    text: &str,
    metadata: HashMap<String, String>,
    public_key: Option<&str>,
    auto_chunk: bool,
    chunk_size: Option<usize>,
    chunk_overlap: Option<usize>,
    client_id: Option<&str>,
) -> Result<InsertOneResult, ErrorResponse> {
    if let Some(id) = client_id {
        validate_client_id(id).map_err(|reason| {
            crate::server::error_middleware::create_validation_error("id", &reason)
        })?;
    }

    let text_len = text.len();
    let should_chunk = auto_chunk && text_len > 2048;

    info!(
        "Inserting text into collection '{}': {} chars (encrypted: {}, chunking: {})",
        collection_name,
        text_len,
        public_key.is_some(),
        should_chunk
    );

    ensure_collection_exists(state, collection_name)?;

    let upload_config = FileUploadConfig::default();
    let chunk_size_val = chunk_size.unwrap_or(upload_config.default_chunk_size);
    let estimated_vectors = if should_chunk {
        std::cmp::max(1, text_len.div_ceil(chunk_size_val))
    } else {
        1
    };

    check_insert_quota(state, tenant_ctx, estimated_vectors).await?;

    let mut vector_ids: Vec<String> = Vec::new();
    let mut last_embedding_len = 0usize;

    if should_chunk {
        let chunk_overlap_val = chunk_overlap.unwrap_or(upload_config.default_chunk_overlap);

        let loader_config = LoaderConfig {
            max_chunk_size: chunk_size_val,
            chunk_overlap: chunk_overlap_val,
            include_patterns: vec![],
            exclude_patterns: vec![],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: collection_name.to_string(),
            max_file_size: upload_config.max_file_size,
        };

        let chunker = Chunker::new(loader_config);

        let file_path = metadata
            .get("filename")
            .or_else(|| metadata.get("file_path"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("text_input"));

        let chunks = chunker
            .chunk_text(text, &file_path)
            .map_err(|e| create_bad_request_error(&format!("Failed to chunk text: {}", e)))?;

        info!(
            "Text chunked into {} chunks (chunk_size: {}, overlap: {})",
            chunks.len(),
            chunk_size_val,
            chunk_overlap_val
        );

        // `parent_id` is recorded on every chunk's payload so chunks of the
        // same source document can be located together (delete-by-doc, RAG
        // citation, group-by-parent searches). When the caller supplied a
        // client id we use it verbatim; otherwise we mint a single UUID
        // shared by all chunks of this insert.
        let parent_id: String = client_id
            .map(str::to_string)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        for chunk in &chunks {
            let embedding = state.embedding_manager.embed(&chunk.content).map_err(|e| {
                create_bad_request_error(&format!("Failed to generate embedding: {}", e))
            })?;
            last_embedding_len = embedding.len();

            // Flat payload shape (phase9): all fields live at the payload
            // root so Qdrant payload filters (`payload.parlamentar = "X"`)
            // and direct field reads work uniformly across short and long
            // texts. The legacy nested layout (`{content, metadata: {...}}`)
            // is still tolerated by readers via
            // `FileOperations::metadata_view`, but new writes never
            // produce it.
            let payload_data = build_chunk_payload(
                &chunk.content,
                &chunk.file_path,
                chunk.chunk_index,
                &parent_id,
                &metadata,
            );

            let payload = if let Some(key) = public_key {
                let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
                    &payload_data,
                    key,
                )
                .map_err(|e| create_bad_request_error(&format!("Encryption failed: {}", e)))?;
                vectorizer::models::Payload::from_encrypted(encrypted)
            } else {
                vectorizer::models::Payload::new(payload_data)
            };

            let vector_id = match client_id {
                Some(id) => format!("{}{}{}", id, CLIENT_ID_CHUNK_SEPARATOR, chunk.chunk_index),
                None => uuid::Uuid::new_v4().to_string(),
            };
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

            vector_ids.push(vector_id);
        }
    } else {
        let embedding = state.embedding_manager.embed(text).map_err(|e| {
            create_bad_request_error(&format!("Failed to generate embedding: {}", e))
        })?;
        last_embedding_len = embedding.len();

        let payload_json = serde_json::Value::Object(
            metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        );

        let payload_data = if let Some(key) = public_key {
            let encrypted =
                vectorizer::security::payload_encryption::encrypt_payload(&payload_json, key)
                    .map_err(|e| create_bad_request_error(&format!("Encryption failed: {}", e)))?;
            vectorizer::models::Payload::from_encrypted(encrypted)
        } else {
            vectorizer::models::Payload::new(payload_json)
        };

        let vector_id = client_id
            .map(str::to_string)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let vector = vectorizer::models::Vector {
            id: vector_id.clone(),
            data: embedding,
            sparse: None,
            payload: Some(payload_data),
            document_id: None,
        };

        state
            .store
            .insert(collection_name, vec![vector])
            .map_err(ErrorResponse::from)?;

        vector_ids.push(vector_id);
    }

    record_insert_usage(
        state,
        collection_name,
        last_embedding_len,
        vector_ids.len() as u64,
    )
    .await;

    mark_collection_dirty(state, collection_name, &vector_ids);

    info!(
        "Successfully inserted {} vector(s) into collection '{}'",
        vector_ids.len(),
        collection_name
    );

    Ok(InsertOneResult {
        vector_ids,
        chunked: should_chunk,
    })
}

/// POST /insert — insert a single text document, auto-chunking large inputs.
pub async fn insert_text(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use vectorizer::monitoring::metrics::METRICS;

    let timer = METRICS.insert_latency_seconds.start_timer();

    let collection_name = payload
        .get("collection")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "collection",
                "missing or invalid collection parameter",
            )
        })?;
    let text = payload
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            crate::server::error_middleware::create_validation_error(
                "text",
                "missing or invalid text parameter",
            )
        })?;

    let metadata = parse_metadata(&payload);
    let public_key = payload.get("public_key").and_then(|k| k.as_str());
    let auto_chunk = payload
        .get("auto_chunk")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let chunk_size = payload
        .get("chunk_size")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let chunk_overlap = payload
        .get("chunk_overlap")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let client_id = payload.get("id").and_then(|i| i.as_str());

    let result = insert_one_text(
        &state,
        tenant_ctx.as_ref(),
        collection_name,
        text,
        metadata,
        public_key,
        auto_chunk,
        chunk_size,
        chunk_overlap,
        client_id,
    )
    .await?;

    let label_collection: &str = collection_name;
    let label_success = "success";
    METRICS
        .insert_requests_total
        .with_label_values(&[label_collection, label_success])
        .inc();
    drop(timer);

    Ok(Json(json!({
        "message": format!(
            "Text inserted successfully ({} vector(s) created)",
            result.vector_ids.len()
        ),
        "vectors_created": result.vector_ids.len(),
        "vector_ids": result.vector_ids,
        "collection": collection_name,
        "chunked": result.chunked
    })))
}
