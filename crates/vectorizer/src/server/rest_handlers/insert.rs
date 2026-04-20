//! Single-document text insertion handler.
//!
//! `insert_text` is split into its own file because it is ~475 lines of
//! substantive logic (chunking, embedding, quota, replication, metrics) that
//! would dominate any shared file.

use std::path::PathBuf;

use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use super::common::collection_metrics_uuid;
use crate::config::FileUploadConfig;
use crate::error::VectorizerError;
use crate::file_loader::chunker::Chunker;
use crate::file_loader::config::LoaderConfig;
use crate::hub::middleware::RequestTenantContext;
use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_bad_request_error};

/// POST /insert — insert a single text document, auto-chunking large inputs
pub async fn insert_text(
    State(state): State<VectorizerServer>,
    tenant_ctx: Option<Extension<RequestTenantContext>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    use crate::monitoring::metrics::METRICS;

    // Start latency timer
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
    let metadata = payload
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
                .collect::<std::collections::HashMap<String, String>>()
        })
        .unwrap_or_default();

    let public_key = payload.get("public_key").and_then(|k| k.as_str());

    // Get chunking parameters (optional)
    let auto_chunk = payload
        .get("auto_chunk")
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default: enable auto-chunking for large texts
    let chunk_size = payload
        .get("chunk_size")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let chunk_overlap = payload
        .get("chunk_overlap")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    // Determine if we should chunk (text is large or auto_chunk is enabled)
    let text_len = text.len();
    let should_chunk = auto_chunk && text_len > 2048; // Threshold: 2048 characters (default chunk size)

    info!(
        "Inserting text into collection '{}': {} chars (encrypted: {}, chunking: {})",
        collection_name,
        text_len,
        public_key.is_some(),
        should_chunk
    );

    // Verify collection exists, create if it doesn't
    // The insert operation needs a write lock, so we can't hold a read reference
    {
        match state.store.get_collection(collection_name) {
            Ok(_) => {
                // Collection exists, continue
            }
            Err(VectorizerError::CollectionNotFound(_)) => {
                // Collection doesn't exist, create it with default config
                info!(
                    "Collection '{}' not found, creating automatically with default config",
                    collection_name
                );

                // Try to load defaults from config.yml
                let default_config = {
                    if let Ok(config_content) = std::fs::read_to_string("config.yml") {
                        if let Ok(yaml_value) =
                            serde_yaml::from_str::<serde_json::Value>(&config_content)
                        {
                            // Extract collections.defaults if available
                            if let Some(collections) = yaml_value.get("collections") {
                                if let Some(defaults) = collections.get("defaults") {
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
                                        "cosine" => crate::models::DistanceMetric::Cosine,
                                        "euclidean" => crate::models::DistanceMetric::Euclidean,
                                        "dot" => crate::models::DistanceMetric::DotProduct,
                                        _ => crate::models::DistanceMetric::Cosine,
                                    };

                                    // Extract HNSW config
                                    let hnsw_config = if let Some(index) = defaults.get("index") {
                                        if let Some(hnsw) = index.get("hnsw") {
                                            crate::models::HnswConfig {
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
                                            }
                                        } else {
                                            crate::models::HnswConfig::default()
                                        }
                                    } else {
                                        crate::models::HnswConfig::default()
                                    };

                                    // Extract quantization config
                                    let quantization =
                                        if let Some(quant) = defaults.get("quantization") {
                                            if let Some(typ) =
                                                quant.get("type").and_then(|t| t.as_str())
                                            {
                                                if typ == "sq" {
                                                    let bits = quant
                                                        .get("sq")
                                                        .and_then(|s| s.get("bits"))
                                                        .and_then(|b| b.as_u64())
                                                        .map(|b| b as usize)
                                                        .unwrap_or(8);
                                                    crate::models::QuantizationConfig::SQ { bits }
                                                } else {
                                                    crate::models::QuantizationConfig::None
                                                }
                                            } else {
                                                crate::models::QuantizationConfig::None
                                            }
                                        } else {
                                            crate::models::QuantizationConfig::SQ { bits: 8 }
                                        };

                                    crate::models::CollectionConfig {
                                        dimension,
                                        metric: metric_enum,
                                        hnsw_config,
                                        quantization,
                                        compression: crate::models::CompressionConfig::default(),
                                        normalization: None,
                                        storage_type: Some(crate::models::StorageType::Memory),
                                        sharding: None,
                                        graph: None,
                                        encryption: None,
                                    }
                                } else {
                                    crate::models::CollectionConfig::default()
                                }
                            } else {
                                crate::models::CollectionConfig::default()
                            }
                        } else {
                            crate::models::CollectionConfig::default()
                        }
                    } else {
                        crate::models::CollectionConfig::default()
                    }
                };

                // Create collection
                state
                    .store
                    .create_collection(collection_name, default_config)
                    .map_err(|e| ErrorResponse::from(e))?;

                info!(
                    "Collection '{}' created successfully with auto-creation",
                    collection_name
                );
            }
            Err(e) => {
                // Other error, return it
                return Err(ErrorResponse::from(e));
            }
        }
        // Reference dropped here at end of block
    }

    // Determine number of vectors to create (for quota check)
    let estimated_vectors = if should_chunk {
        // Estimate: divide text length by chunk size
        let upload_config = FileUploadConfig::default();
        let chunk_size_val = chunk_size.unwrap_or(upload_config.default_chunk_size);
        std::cmp::max(1, (text_len + chunk_size_val - 1) / chunk_size_val)
    } else {
        1
    };

    // Check HiveHub quota for vector insertion if enabled
    if let Some(ref hub_manager) = state.hub_manager {
        // Extract tenant ID from request context, or use "default" for anonymous requests
        let tenant_id = tenant_ctx
            .as_ref()
            .map(|ctx| ctx.0.0.tenant_id.as_str())
            .unwrap_or("default");
        match hub_manager
            .check_quota(
                tenant_id,
                crate::hub::QuotaType::VectorCount,
                estimated_vectors as u64,
            )
            .await
        {
            Ok(allowed) => {
                if !allowed {
                    warn!(
                        "Vector insertion denied for tenant {}: quota exceeded (estimated {} vectors)",
                        tenant_id, estimated_vectors
                    );
                    return Err(ErrorResponse::new(
                        "QUOTA_EXCEEDED".to_string(),
                        format!(
                            "Vector quota exceeded. Estimated {} vectors needed. Please upgrade your plan or delete unused vectors.",
                            estimated_vectors
                        ),
                        StatusCode::TOO_MANY_REQUESTS,
                    ));
                }
            }
            Err(e) => {
                warn!("Failed to check quota with HiveHub: {}", e);
            }
        }
    }

    let mut vectors_created = 0;
    let mut vector_ids = Vec::new();

    if should_chunk {
        // Use chunking for large texts
        let upload_config = FileUploadConfig::default();
        let chunk_size_val = chunk_size.unwrap_or(upload_config.default_chunk_size);
        let chunk_overlap_val = chunk_overlap.unwrap_or(upload_config.default_chunk_overlap);

        // Create chunker configuration
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

        // Use filename from metadata if available, otherwise use a default
        let file_path = metadata
            .get("filename")
            .or_else(|| metadata.get("file_path"))
            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|| PathBuf::from("text_input"));

        // Chunk the text
        let chunks = chunker
            .chunk_text(text, &file_path)
            .map_err(|e| create_bad_request_error(&format!("Failed to chunk text: {}", e)))?;

        info!(
            "Text chunked into {} chunks (chunk_size: {}, overlap: {})",
            chunks.len(),
            chunk_size_val,
            chunk_overlap_val
        );

        // Create vectors for each chunk
        for chunk in &chunks {
            // Generate embedding for the chunk
            let embedding = state.embedding_manager.embed(&chunk.content).map_err(|e| {
                create_bad_request_error(&format!("Failed to generate embedding: {}", e))
            })?;
            let embedding_len = embedding.len();

            // Build payload with chunk metadata
            let mut payload_data = json!({
                "content": chunk.content,
                "chunk_index": chunk.chunk_index,
                "file_path": chunk.file_path,
            });

            // Merge user-provided metadata
            if let Some(obj) = payload_data.as_object_mut() {
                for (k, v) in &metadata {
                    if !obj.contains_key(k) {
                        obj.insert(k.clone(), json!(v));
                    }
                }
            }

            // Encrypt payload if public_key is provided
            let payload = if let Some(key) = public_key {
                let encrypted =
                    crate::security::payload_encryption::encrypt_payload(&payload_data, key)
                        .map_err(|e| {
                            create_bad_request_error(&format!("Encryption failed: {}", e))
                        })?;
                crate::models::Payload::from_encrypted(encrypted)
            } else {
                crate::models::Payload::new(payload_data)
            };

            // Create vector with generated ID
            let vector_id = format!("{}", uuid::Uuid::new_v4());
            let vector = crate::models::Vector {
                id: vector_id.clone(),
                data: embedding,
                sparse: None,
                payload: Some(payload),
                document_id: None,
            };

            // Insert the vector
            state
                .store
                .insert(collection_name, vec![vector])
                .map_err(|e| ErrorResponse::from(e))?;

            vector_ids.push(vector_id);
            vectors_created += 1;

            // Record usage metrics if HiveHub is enabled
            if let Some(ref hub_manager) = state.hub_manager {
                let mut metrics = crate::hub::UsageMetrics::new();
                let estimated_storage = (embedding_len * 4) as u64;
                metrics.record_insert(1, estimated_storage);
                let collection_id = collection_metrics_uuid(collection_name);
                if let Err(e) = hub_manager.record_usage(collection_id, metrics).await {
                    warn!("Failed to record vector insertion usage: {}", e);
                }
            }
        }
    } else {
        // Single vector for small texts (original behavior)
        let embedding = state.embedding_manager.embed(text).map_err(|e| {
            create_bad_request_error(&format!("Failed to generate embedding: {}", e))
        })?;
        let embedding_len = embedding.len();

        // Create payload with metadata
        let payload_json = serde_json::Value::Object(
            metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        );

        // Encrypt payload if public_key is provided
        let payload_data = if let Some(key) = public_key {
            let encrypted =
                crate::security::payload_encryption::encrypt_payload(&payload_json, key)
                    .map_err(|e| create_bad_request_error(&format!("Encryption failed: {}", e)))?;
            crate::models::Payload::from_encrypted(encrypted)
        } else {
            crate::models::Payload::new(payload_json)
        };

        // Create vector with generated ID
        let vector_id = format!("{}", uuid::Uuid::new_v4());
        let vector = crate::models::Vector {
            id: vector_id.clone(),
            data: embedding,
            sparse: None,
            payload: Some(payload_data),
            document_id: None,
        };

        // Insert the vector using the store
        state
            .store
            .insert(collection_name, vec![vector])
            .map_err(|e| ErrorResponse::from(e))?;

        vector_ids.push(vector_id);
        vectors_created = 1;

        // Record usage metrics if HiveHub is enabled
        if let Some(ref hub_manager) = state.hub_manager {
            let mut metrics = crate::hub::UsageMetrics::new();
            let estimated_storage = (embedding_len * 4) as u64;
            metrics.record_insert(1, estimated_storage);
            let collection_id = collection_metrics_uuid(collection_name);
            if let Err(e) = hub_manager.record_usage(collection_id, metrics).await {
                warn!("Failed to record vector insertion usage: {}", e);
            }
        }
    }

    // Mark changes for auto-save
    if let Some(ref auto_save) = state.auto_save_manager {
        auto_save.mark_changed();
    }

    // Invalidate cache for this collection
    state.query_cache.invalidate_collection(collection_name);
    debug!(
        "💾 Cache invalidated for collection '{}' after insert",
        collection_name
    );

    // Replicate vector insertions to followers (Raft HA mode)
    let active_master: Option<std::sync::Arc<crate::replication::MasterNode>> = state
        .master_node
        .clone()
        .or_else(|| state.ha_manager.as_ref().and_then(|ha| ha.master_node()));
    if let Some(ref master) = active_master {
        // Re-read the inserted vectors for replication
        if let Ok(col) = state.store.get_collection(collection_name) {
            for vid in &vector_ids {
                if let Ok(v) = col.get_vector(vid) {
                    let payload_bytes = v.payload.as_ref().and_then(|p| serde_json::to_vec(p).ok());
                    let op = crate::replication::VectorOperation::InsertVector {
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

    info!(
        "Successfully inserted {} vector(s) into collection '{}'",
        vectors_created, collection_name
    );

    let label_collection: &str = &collection_name;
    let label_success = "success";
    METRICS
        .insert_requests_total
        .with_label_values(&[label_collection, label_success])
        .inc();
    drop(timer);

    Ok(Json(json!({
        "message": format!("Text inserted successfully ({} vector(s) created)", vectors_created),
        "vectors_created": vectors_created,
        "vector_ids": vector_ids,
        "collection": collection_name,
        "chunked": should_chunk
    })))
}
