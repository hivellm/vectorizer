//! MCP Tool handlers

use std::sync::Arc;

use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock, ErrorCode, ErrorData};
use serde_json::json;
use vectorizer::VectorStore;
use vectorizer::VectorizerError;
use vectorizer::db::graph::RelationshipType;
use vectorizer::db::{HybridScoringAlgorithm, HybridSearchConfig};
use vectorizer::discovery::{
    CollectionRef, Discovery, DiscoveryConfig, ExpansionConfig, expand_queries_baseline,
    filter_collections,
};
use vectorizer::embedding::EmbeddingManager;
use vectorizer::file_operations::{FileListFilter, FileOperations, SortBy, SummaryType};
use vectorizer::intelligent_search::mcp_tools::*;
use vectorizer::models::SparseVector;
use vectorizer_core::error::mapping;

use crate::server::discovery_handlers::*;
use crate::server::files::operations::*;
use crate::server::graph_handlers::*;

/// Convert a [`VectorizerError`] into the matching MCP [`ErrorData`],
/// routing through the centralized `ErrorKind` → JSON-RPC code mapping
/// (`mapping::mcp_code`) instead of hardcoding `internal_error`
/// (`-32603`) for every failure mode (phase40 §2.3). A
/// `CollectionNotFound` now surfaces as the mapped not-found code
/// instead of Internal, matching the REST `404` / gRPC `NotFound`
/// codes for the same underlying error. The stable
/// [`VectorizerError::code`] identifier travels in `data.code` so MCP
/// clients get the same machine-readable signal REST's `error_type`
/// field carries.
fn to_mcp_error(err: VectorizerError) -> ErrorData {
    let code = mapping::mcp_code(&err);
    let data = json!({ "code": err.code() });
    ErrorData::new(ErrorCode(code), err.to_string(), Some(data))
}

/// Same idea as [`to_mcp_error`] but for
/// [`vectorizer::file_operations::FileOperationError`] — a distinct
/// error type from [`VectorizerError`] predating the centralized error
/// taxonomy. Reclassifies each variant into the closest
/// [`VectorizerError`] kind so file-not-found / invalid-parameter
/// errors get the same non-Internal MCP codes that collection/vector
/// lookups already do.
fn to_mcp_error_file_op(err: vectorizer::file_operations::FileOperationError) -> ErrorData {
    use vectorizer::file_operations::FileOperationError as FileOpError;

    let mapped = match err {
        FileOpError::FileNotFound {
            file_path,
            collection,
        } => VectorizerError::NotFound(format!(
            "file '{}' not found in collection '{}'",
            file_path, collection
        )),
        FileOpError::NoChunksFound { file_path } => {
            VectorizerError::NotFound(format!("no chunks found for file '{}'", file_path))
        }
        FileOpError::CollectionNotFound { collection } => {
            VectorizerError::CollectionNotFound(collection)
        }
        FileOpError::FileTooLarge {
            size_kb,
            max_size_kb,
        } => VectorizerError::InvalidConfiguration {
            message: format!(
                "file too large: {}KB exceeds limit of {}KB",
                size_kb, max_size_kb
            ),
        },
        FileOpError::InvalidPath { path, reason } => VectorizerError::InvalidConfiguration {
            message: format!("invalid path '{}': {}", path, reason),
        },
        FileOpError::InvalidParameter { param, reason } => VectorizerError::InvalidConfiguration {
            message: format!("invalid parameter '{}': {}", param, reason),
        },
        FileOpError::CacheError { message } => {
            VectorizerError::Other(format!("cache error: {}", message))
        }
        FileOpError::SummarizationError { message } => {
            VectorizerError::Other(format!("summarization error: {}", message))
        }
        FileOpError::VectorStoreError(message) => VectorizerError::Other(message),
        FileOpError::IoError(e) => VectorizerError::IoError(e),
        FileOpError::JsonError(e) => VectorizerError::JsonError(e),
    };
    to_mcp_error(mapped)
}

/// Helper function to create an embedding manager for a specific collection
fn create_embedding_manager_for_collection(
    embedding_type: &str,
    dimension: usize,
) -> Result<EmbeddingManager, String> {
    let mut manager = EmbeddingManager::new();

    match embedding_type {
        "bm25" => {
            let bm25 = vectorizer::embedding::Bm25Embedding::new(dimension);
            manager.register_provider("bm25".to_string(), Box::new(bm25));
            manager
                .set_default_provider("bm25")
                .map_err(|e| format!("Failed to set BM25 provider: {}", e))?;
        }
        "tfidf" => {
            let tfidf = vectorizer::embedding::TfIdfEmbedding::new(dimension);
            manager.register_provider("tfidf".to_string(), Box::new(tfidf));
            manager
                .set_default_provider("tfidf")
                .map_err(|e| format!("Failed to set TF-IDF provider: {}", e))?;
        }
        "svd" => {
            let svd = vectorizer::embedding::SvdEmbedding::new(dimension, dimension);
            manager.register_provider("svd".to_string(), Box::new(svd));
            manager
                .set_default_provider("svd")
                .map_err(|e| format!("Failed to set SVD provider: {}", e))?;
        }
        "bert" => {
            let bert = vectorizer::embedding::BertEmbedding::new(dimension);
            manager.register_provider("bert".to_string(), Box::new(bert));
            manager
                .set_default_provider("bert")
                .map_err(|e| format!("Failed to set BERT provider: {}", e))?;
        }
        "minilm" => {
            let minilm = vectorizer::embedding::MiniLmEmbedding::new(dimension);
            manager.register_provider("minilm".to_string(), Box::new(minilm));
            manager
                .set_default_provider("minilm")
                .map_err(|e| format!("Failed to set MiniLM provider: {}", e))?;
        }
        "bagofwords" => {
            let bow = vectorizer::embedding::BagOfWordsEmbedding::new(dimension);
            manager.register_provider("bagofwords".to_string(), Box::new(bow));
            manager
                .set_default_provider("bagofwords")
                .map_err(|e| format!("Failed to set BagOfWords provider: {}", e))?;
        }
        "charngram" => {
            let char_ngram = vectorizer::embedding::CharNGramEmbedding::new(dimension, 3);
            manager.register_provider("charngram".to_string(), Box::new(char_ngram));
            manager
                .set_default_provider("charngram")
                .map_err(|e| format!("Failed to set CharNGram provider: {}", e))?;
        }
        _ => {
            // Default to BM25 if unknown type
            let bm25 = vectorizer::embedding::Bm25Embedding::new(dimension);
            manager.register_provider("bm25".to_string(), Box::new(bm25));
            manager
                .set_default_provider("bm25")
                .map_err(|e| format!("Failed to set default BM25 provider: {}", e))?;
        }
    }

    Ok(manager)
}

pub async fn handle_mcp_tool(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    cluster_manager: Option<Arc<vectorizer::cluster::ClusterManager>>,
    upsert_queue: Arc<vectorizer::db::UpsertQueue>,
) -> Result<CallToolResult, ErrorData> {
    match request.name.as_ref() {
        // Core Collection/Vector Operations
        "list_collections" => handle_list_collections(store).await,
        "list_providers" => handle_list_providers(embedding_manager).await,
        "create_collection" => handle_create_collection(request, store).await,
        "get_collection_info" => handle_get_collection_info(request, store).await,
        "insert_text" => handle_insert_text(request, store, embedding_manager, upsert_queue).await,
        "get_vector" => handle_get_vector(request, store).await,
        "update_vector" => handle_update_vector(request, store, embedding_manager).await,
        "delete_vector" => handle_delete_vectors(request, store).await,
        "multi_collection_search" => {
            handle_multi_collection_search(request, store, embedding_manager).await
        }
        "search" => handle_search_vectors(request, store, embedding_manager).await,

        // Search Operations
        "search_intelligent" => handle_intelligent_search(request, store, embedding_manager).await,
        "search_semantic" => handle_semantic_search(request, store, embedding_manager).await,
        "search_extra" => handle_search_extra(request, store, embedding_manager).await,
        "search_hybrid" => handle_hybrid_search(request, store, embedding_manager).await,

        // Discovery Operations
        "filter_collections" => handle_filter_collections(request, store).await,
        "expand_queries" => handle_expand_queries(request).await,

        // File Operations
        "get_file_content" => handle_get_file_content(request, store).await,
        "list_files" => handle_list_files_in_collection(request, store).await,
        "get_file_chunks" => handle_get_file_chunks_ordered(request, store).await,
        "get_project_outline" => handle_get_project_outline(request, store).await,
        "get_related_files" => handle_get_related_files(request, store, embedding_manager).await,

        // Graph Operations
        "graph_list_nodes" => handle_graph_list_nodes(request, store).await,
        "graph_get_neighbors" => handle_graph_get_neighbors(request, store).await,
        "graph_find_related" => handle_graph_find_related(request, store).await,
        "graph_find_path" => handle_graph_find_path(request, store).await,
        "graph_create_edge" => handle_graph_create_edge(request, store).await,
        "graph_delete_edge" => handle_graph_delete_edge(request, store).await,
        "graph_discover_edges" => handle_graph_discover_edges(request, store).await,
        "graph_discover_status" => handle_graph_discover_status(request, store).await,

        // Collection Maintenance
        "list_empty_collections" => handle_list_empty_collections(store).await,
        "cleanup_empty_collections" => handle_cleanup_empty_collections(request, store).await,
        "get_collection_stats" => handle_get_collection_stats(request, store).await,

        // phase40 §2.1: MCP tools mirroring REST-only endpoints
        // (delete_collection, embed_text, contextual_search,
        // get_database_stats) — REST-first rule: same underlying
        // VectorStore/EmbeddingManager calls the REST handlers make.
        "delete_collection" => handle_delete_collection(request, store).await,
        "embed_text" => handle_embed_text(request, embedding_manager).await,
        "contextual_search" => handle_contextual_search(request, store).await,
        "get_database_stats" => handle_get_database_stats(store, embedding_manager).await,

        // phase40 §2.2: discovery pipeline (8 ops). The handlers already
        // existed in `discovery_handlers` for REST/RPC — this wires them
        // into the MCP dispatch table.
        "discover" => handle_discover(request, store, embedding_manager).await,
        "score_collections" => handle_score_collections(request, store).await,
        "broad_discovery" => handle_broad_discovery(request, store, embedding_manager).await,
        "semantic_focus" => handle_semantic_focus(request, store, embedding_manager).await,
        "promote_readme" => handle_promote_readme(request).await,
        "compress_evidence" => handle_compress_evidence(request).await,
        "build_answer_plan" => handle_build_answer_plan(request).await,
        "render_llm_prompt" => handle_render_llm_prompt(request).await,

        // phase40 §2.2: batch operations mirroring REST /batch_* routes
        "batch_insert_texts" => {
            handle_batch_insert_texts(request, store, embedding_manager, upsert_queue).await
        }
        "batch_search" => handle_batch_search(request, store, embedding_manager).await,
        "batch_update" => handle_batch_update(request, store).await,
        "batch_delete" => handle_batch_delete(request, store).await,

        _ => Err(ErrorData::invalid_params("Unknown tool", None)),
    }
}

// =============================================
// Handler Functions
// =============================================

async fn handle_search_vectors(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    _embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    // Get the collection to access its embedding type and dimension
    let collection = store
        .get_collection(collection_name)
        .map_err(to_mcp_error)?;

    let embedding_type = collection.get_embedding_type();
    let dimension = collection.config().dimension;

    // Create embedding manager specific to this collection
    let collection_embedding_manager =
        create_embedding_manager_for_collection(&embedding_type, dimension).map_err(|e| {
            ErrorData::internal_error(format!("Failed to create embedding manager: {}", e), None)
        })?;

    // Generate embedding using the collection-specific manager
    let embedding = collection_embedding_manager
        .embed(query)
        .map_err(to_mcp_error)?;

    // Search
    let results = store
        .search(collection_name, &embedding, limit)
        .map_err(to_mcp_error)?;

    let response = json!({
        "results": results.iter().map(|r| json!({
            "id": r.id,
            "score": r.score,
            "payload": r.payload
        })).collect::<Vec<_>>(),
        "total": results.len()
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_list_collections(store: Arc<VectorStore>) -> Result<CallToolResult, ErrorData> {
    let collections = store.list_collections();
    let response = json!({
        "collections": collections,
        "total": collections.len()
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

/// phase33 (#306): mirrors the `providers` array from
/// `GET /stats` so MCP callers can discover what's registered
/// before posting `embedding_provider` on `create_collection` /
/// `model` on `embed_text`. Without this tool the only way to
/// notice an unregistered provider was to watch the call return
/// `400 unsupported_provider`.
async fn handle_list_providers(
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let default = embedding_manager
        .get_default_provider_name()
        .map(|s| s.to_string());
    let providers: Vec<serde_json::Value> = embedding_manager
        .list_providers()
        .into_iter()
        .map(|name| {
            let dimension = embedding_manager.get_provider_dimension(&name).unwrap_or(0);
            let is_default = default.as_deref() == Some(name.as_str());
            json!({
                "name": name,
                "dimension": dimension,
                "default": is_default,
            })
        })
        .collect();
    let response = json!({
        "providers": providers,
        "default_provider": default,
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_create_collection(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing name", None))?;

    let dimension =
        args.get("dimension")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ErrorData::invalid_params("Missing dimension", None))? as usize;

    let metric = args
        .get("metric")
        .and_then(|v| v.as_str())
        .unwrap_or("cosine");

    let distance_metric = match metric {
        "euclidean" => vectorizer::models::DistanceMetric::Euclidean,
        _ => vectorizer::models::DistanceMetric::Cosine,
    };

    let embedding_provider = args
        .get("embedding_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("bm25")
        .to_string();

    // Parse graph configuration if provided
    let graph_config = args.get("graph").and_then(|g| {
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

    let config = vectorizer::models::CollectionConfig {
        dimension,
        metric: distance_metric,
        quantization: vectorizer::models::QuantizationConfig::SQ { bits: 8 },
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig {
            enabled: false,
            threshold_bytes: 1024,
            algorithm: vectorizer::models::CompressionAlgorithm::Lz4,
        },
        embedding_provider,
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        graph: graph_config,
        sharding: None,
        encryption: None,
    };

    store
        .create_collection(name, config)
        .map_err(to_mcp_error)?;

    let response = json!({
        "status": "created",
        "name": name,
        "dimension": dimension,
        "metric": metric
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_get_collection_info(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing name", None))?;

    let collection = store.get_collection(name).map_err(to_mcp_error)?;

    let metadata = collection.metadata();
    let config = collection.config();

    // Build normalization info
    let normalization_info = if let Some(norm_config) = &config.normalization {
        json!({
            "enabled": norm_config.enabled,
            "level": format!("{:?}", norm_config.policy.level),
            "preserve_case": norm_config.policy.preserve_case,
            "collapse_whitespace": norm_config.policy.collapse_whitespace,
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

    let response = json!({
        "name": name,
        "vector_count": collection.vector_count(),
        "document_count": metadata.document_count,
        "dimension": config.dimension,
        "metric": format!("{:?}", config.metric),
        "quantization": {
            "type": format!("{:?}", config.quantization),
            "enabled": !matches!(config.quantization, vectorizer::models::QuantizationConfig::None)
        },
        "normalization": normalization_info,
        "created_at": metadata.created_at.to_rfc3339(),
        "updated_at": metadata.updated_at.to_rfc3339(),
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_insert_text(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    upsert_queue: Arc<vectorizer::db::UpsertQueue>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection_name", None))?;

    let text = args
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing text", None))?;

    // Issue #263: per-collection admission. Held until handler exit.
    let _ticket = match upsert_queue.try_admit(collection_name) {
        Ok((ticket, _status)) => ticket,
        Err(vectorizer::db::AdmissionError::QueueFull {
            depth,
            hard_limit,
            retry_after_seconds,
        }) => {
            return Err(ErrorData::internal_error(
                format!(
                    "queue_full: collection '{}' depth {} >= hard_limit {} (retry after {}s)",
                    collection_name, depth, hard_limit, retry_after_seconds,
                ),
                Some(json!({
                    "code": "queue_full",
                    "collection": collection_name,
                    "depth": depth,
                    "hard_limit": hard_limit,
                    "retryAfterSeconds": retry_after_seconds,
                })),
            ));
        }
    };

    let metadata = args.get("metadata").cloned();
    let public_key = args.get("public_key").and_then(|v| v.as_str());

    // Generate embedding
    let embedding = embedding_manager.embed(text).map_err(to_mcp_error)?;

    let vector_id = uuid::Uuid::new_v4().to_string();

    let payload_json = if let Some(meta) = metadata {
        meta
    } else {
        json!({})
    };

    // Encrypt payload if public_key is provided
    let payload = if let Some(key) = public_key {
        let encrypted =
            vectorizer::security::payload_encryption::encrypt_payload(&payload_json, key)
                .map_err(|e| to_mcp_error(VectorizerError::EncryptionError(e.to_string())))?;
        vectorizer::models::Payload::from_encrypted(encrypted)
    } else {
        vectorizer::models::Payload::new(payload_json)
    };

    store
        .insert(
            collection_name,
            vec![vectorizer::models::Vector::with_payload(
                vector_id.clone(),
                embedding,
                payload,
            )],
        )
        .map_err(to_mcp_error)?;

    let response = json!({
        "status": "inserted",
        "vector_id": vector_id,
        "collection": collection_name,
        "encrypted": public_key.is_some()
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_get_vector(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let vector_id = args
        .get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing vector_id", None))?;

    let coll = store.get_collection(collection).map_err(to_mcp_error)?;

    let vector = coll.get_vector(vector_id).map_err(to_mcp_error)?;

    let response = json!({
        "id": vector.id,
        "data": vector.data,
        "payload": vector.payload,
        "collection": collection
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_delete_vectors(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let vector_ids = args
        .get("vector_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing vector_ids array", None))?;

    let mut deleted_count = 0;
    for id_value in vector_ids {
        if let Some(id) = id_value.as_str() {
            if store.delete(collection, id).is_ok() {
                deleted_count += 1;
            }
        }
    }

    let response = json!({
        "status": "deleted",
        "collection": collection,
        "count": deleted_count
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_update_vector(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let vector_id = args
        .get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing vector_id", None))?;

    let text = args.get("text").and_then(|v| v.as_str());
    let metadata = args.get("metadata").cloned();
    let public_key = args.get("public_key").and_then(|v| v.as_str());

    if let Some(text) = text {
        let embedding = embedding_manager.embed(text).map_err(to_mcp_error)?;

        let payload_json = if let Some(meta) = metadata {
            meta
        } else {
            json!({})
        };

        // Encrypt payload if public_key is provided
        let payload = if let Some(key) = public_key {
            let encrypted =
                vectorizer::security::payload_encryption::encrypt_payload(&payload_json, key)
                    .map_err(|e| to_mcp_error(VectorizerError::EncryptionError(e.to_string())))?;
            vectorizer::models::Payload::from_encrypted(encrypted)
        } else {
            vectorizer::models::Payload::new(payload_json)
        };

        store
            .update(
                collection,
                vectorizer::models::Vector::with_payload(vector_id.to_string(), embedding, payload),
            )
            .map_err(to_mcp_error)?;
    }

    let response = json!({
        "status": "updated",
        "vector_id": vector_id,
        "collection": collection,
        "encrypted": public_key.is_some()
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

// Intelligent Search Handlers

async fn handle_intelligent_search(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let collections = args
        .get("collections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        });

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let domain_expansion = args
        .get("domain_expansion")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let tool = IntelligentSearchTool {
        query: query.to_string(),
        collections,
        max_results: Some(max_results),
        domain_expansion: Some(domain_expansion),
        technical_focus: Some(true),
        mmr_enabled: Some(false), // Disabled for MCP
        mmr_lambda: Some(0.7),
    };

    // Create handler with collection-specific embedding managers
    let handler = MCPToolHandler::new_with_store(store.clone());
    let response = handler.handle_intelligent_search(tool).await.map_err(|e| {
        ErrorData::internal_error(format!("Intelligent search failed: {}", e), None)
    })?;

    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;

    Ok(CallToolResult::success(vec![ContentBlock::text(
        json_response.to_string(),
    )]))
}

async fn handle_multi_collection_search(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let collections = args
        .get("collections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing collections", None))?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let max_per_collection = args
        .get("max_per_collection")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;

    let max_total_results = args
        .get("max_total_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;

    let tool = MultiCollectionSearchTool {
        query: query.to_string(),
        collections,
        max_per_collection: Some(max_per_collection),
        max_total_results: Some(max_total_results),
        cross_collection_reranking: Some(false), // Disabled for MCP
    };

    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());
    let response = handler
        .handle_multi_collection_search(tool)
        .await
        .map_err(|e| {
            ErrorData::internal_error(format!("Multi collection search failed: {}", e), None)
        })?;

    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;

    Ok(CallToolResult::success(vec![ContentBlock::text(
        json_response.to_string(),
    )]))
}

async fn handle_semantic_search(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let similarity_threshold = args
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.1) as f32;

    let tool = SemanticSearchTool {
        query: query.to_string(),
        collection: collection.to_string(),
        max_results: Some(max_results),
        semantic_reranking: Some(true),
        cross_encoder_reranking: Some(false), // Disabled for MCP
        similarity_threshold: Some(similarity_threshold),
    };

    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());
    let response = handler
        .handle_semantic_search(tool)
        .await
        .map_err(|e| ErrorData::internal_error(format!("Semantic search failed: {}", e), None))?;

    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;

    Ok(CallToolResult::success(vec![ContentBlock::text(
        json_response.to_string(),
    )]))
}

// New search_extra handler - combines multiple search strategies
async fn handle_search_extra(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let strategies = args
        .get("strategies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["basic".to_string(), "semantic".to_string()]);

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let similarity_threshold = args
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.1) as f32;

    let mut all_results: Vec<serde_json::Value> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Execute each strategy and collect results
    for strategy in &strategies {
        match strategy.as_str() {
            "basic" => {
                // Basic search
                let coll = store.get_collection(collection).map_err(to_mcp_error)?;
                let embedding_type = coll.get_embedding_type();
                let dimension = coll.config().dimension;
                let coll_emb_manager =
                    create_embedding_manager_for_collection(&embedding_type, dimension).map_err(
                        |e| {
                            ErrorData::internal_error(
                                format!("Failed to create embedding manager: {}", e),
                                None,
                            )
                        },
                    )?;
                let embedding = coll_emb_manager.embed(query).map_err(to_mcp_error)?;
                let results = store
                    .search(collection, &embedding, max_results)
                    .map_err(to_mcp_error)?;

                for result in results {
                    if !seen_ids.contains(&result.id) {
                        seen_ids.insert(result.id.clone());
                        all_results.push(json!({
                            "id": result.id,
                            "score": result.score,
                            "payload": result.payload,
                            "strategy": "basic"
                        }));
                    }
                }
            }
            "semantic" => {
                // Semantic search
                let tool = SemanticSearchTool {
                    query: query.to_string(),
                    collection: collection.to_string(),
                    max_results: Some(max_results),
                    semantic_reranking: Some(true),
                    cross_encoder_reranking: Some(false),
                    similarity_threshold: Some(similarity_threshold),
                };
                let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());
                let response = handler.handle_semantic_search(tool).await.map_err(|e| {
                    ErrorData::internal_error(format!("Semantic search failed: {}", e), None)
                })?;

                for result in response.results {
                    if !seen_ids.contains(&result.doc_id) {
                        seen_ids.insert(result.doc_id.clone());
                        all_results.push(json!({
                            "id": result.doc_id,
                            "score": result.score,
                            "content": result.content,
                            "collection": result.collection,
                            "metadata": result.metadata,
                            "strategy": "semantic"
                        }));
                    }
                }
            }
            "intelligent" => {
                // Intelligent search (single collection)
                let tool = IntelligentSearchTool {
                    query: query.to_string(),
                    collections: Some(vec![collection.to_string()]),
                    max_results: Some(max_results),
                    domain_expansion: Some(true),
                    technical_focus: Some(true),
                    mmr_enabled: Some(false),
                    mmr_lambda: Some(0.7),
                };
                let handler = MCPToolHandler::new_with_store(store.clone());
                let response = handler.handle_intelligent_search(tool).await.map_err(|e| {
                    ErrorData::internal_error(format!("Intelligent search failed: {}", e), None)
                })?;

                for result in response.results {
                    if !seen_ids.contains(&result.doc_id) {
                        seen_ids.insert(result.doc_id.clone());
                        all_results.push(json!({
                            "id": result.doc_id,
                            "score": result.score,
                            "content": result.content,
                            "collection": result.collection,
                            "metadata": result.metadata,
                            "strategy": "intelligent"
                        }));
                    }
                }
            }
            _ => continue, // Skip unknown strategies
        }
    }

    // Sort by score descending
    all_results.sort_by(|a, b| {
        let score_a = a.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let score_b = b.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit total results
    all_results.truncate(max_results * 2); // Allow more since we're combining strategies

    let response = json!({
        "query": query,
        "collection": collection,
        "strategies_used": strategies,
        "results": all_results,
        "total": all_results.len()
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

// =========================
// File Operations Handlers
// =========================

async fn handle_get_file_content(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing file_path", None))?;

    let max_size_kb = args
        .get("max_size_kb")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as usize;

    // Initialize FileOperations WITH STORE
    let file_ops = FileOperations::with_store(store);

    // Get file content
    let result = file_ops
        .get_file_content(collection, file_path, max_size_kb)
        .await
        .map_err(to_mcp_error_file_op)?;

    let response = json!({
        "file_path": result.file_path,
        "content": result.content,
        "metadata": {
            "file_type": result.metadata.file_type,
            "size_kb": result.metadata.size_kb,
            "chunk_count": result.metadata.chunk_count,
            "last_indexed": result.metadata.last_indexed,
            "language": result.metadata.language,
        },
        "chunks_available": result.chunks_available,
        "collection": result.collection,
        "from_cache": result.from_cache,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_list_files_in_collection(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    // Parse filter parameters
    let filter_by_type = args
        .get("filter_by_type")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    let min_chunks = args
        .get("min_chunks")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    let sort_by = args
        .get("sort_by")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "name" => Some(SortBy::Name),
            "size" => Some(SortBy::Size),
            "chunks" => Some(SortBy::Chunks),
            "recent" => Some(SortBy::Recent),
            _ => None,
        })
        .unwrap_or(SortBy::Name);

    let filter = FileListFilter {
        filter_by_type,
        min_chunks,
        max_results,
        sort_by,
    };

    // Initialize FileOperations WITH STORE
    let file_ops = FileOperations::with_store(store);

    // List files
    let result = file_ops
        .list_files_in_collection(collection, filter)
        .await
        .map_err(to_mcp_error_file_op)?;

    let response = json!({
        "collection": result.collection,
        "total_files": result.total_files,
        "total_chunks": result.total_chunks,
        "files": result.files.iter().map(|f| json!({
            "path": f.path,
            "file_type": f.file_type,
            "chunk_count": f.chunk_count,
            "size_estimate_kb": f.size_estimate_kb,
            "last_indexed": f.last_indexed,
            "has_summary": f.has_summary,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_hybrid_search(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    // Get collection to determine embedding type and dimension
    let collection = store
        .get_collection(collection_name)
        .map_err(to_mcp_error)?;

    let embedding_type = collection.get_embedding_type();
    let dimension = collection.config().dimension;

    // Create embedding manager for this collection
    let collection_embedding_manager =
        create_embedding_manager_for_collection(&embedding_type, dimension).map_err(|e| {
            ErrorData::internal_error(format!("Failed to create embedding manager: {}", e), None)
        })?;

    // Generate dense embedding from query text
    let query_dense = collection_embedding_manager
        .embed(query)
        .map_err(to_mcp_error)?;

    // Parse optional sparse query
    let query_sparse = if let Some(sparse_obj) = args.get("query_sparse") {
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
                            ErrorData::invalid_params(format!("Invalid sparse vector: {}", e), None)
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
    let alpha = args.get("alpha").and_then(|v| v.as_f64()).unwrap_or(0.7) as f32;
    let algorithm_str = args
        .get("algorithm")
        .and_then(|v| v.as_str())
        .unwrap_or("rrf");
    let algorithm = match algorithm_str {
        "rrf" => HybridScoringAlgorithm::ReciprocalRankFusion,
        "weighted" => HybridScoringAlgorithm::WeightedCombination,
        "alpha" => HybridScoringAlgorithm::AlphaBlending,
        _ => HybridScoringAlgorithm::ReciprocalRankFusion,
    };
    let dense_k = args.get("dense_k").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let sparse_k = args.get("sparse_k").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let final_k = args.get("final_k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let config = HybridSearchConfig {
        alpha,
        dense_k,
        sparse_k,
        final_k,
        algorithm,
    };

    // Perform hybrid search
    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .map_err(to_mcp_error)?;

    let response = json!({
        "results": results.iter().map(|r| json!({
            "id": r.id,
            "score": r.score,
            "payload": r.payload
        })).collect::<Vec<_>>(),
        "total": results.len(),
        "config": {
            "alpha": alpha,
            "algorithm": algorithm_str,
            "dense_k": dense_k,
            "sparse_k": sparse_k,
            "final_k": final_k
        }
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

// =============================================
// Collection Maintenance Handlers
// =============================================

async fn handle_list_empty_collections(
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let empty_collections = store.list_empty_collections();

    let response = json!({
        "status": "success",
        "empty_collections": empty_collections,
        "count": empty_collections.len()
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

async fn handle_cleanup_empty_collections(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref();

    let dry_run = args
        .and_then(|a| a.get("dry_run"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    match store.cleanup_empty_collections(dry_run) {
        Ok(deleted_count) => {
            let message = if dry_run {
                format!("Would delete {} empty collections", deleted_count)
            } else {
                format!("Successfully deleted {} empty collections", deleted_count)
            };

            let response = json!({
                "status": "success",
                "dry_run": dry_run,
                "deleted_count": deleted_count,
                "message": message
            });

            Ok(CallToolResult::success(vec![ContentBlock::text(
                response.to_string(),
            )]))
        }
        Err(e) => Err(to_mcp_error(e)),
    }
}

async fn handle_get_collection_stats(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection name", None))?;

    // Get collection to check if it exists
    let collection_ref = store.get_collection(collection).map_err(to_mcp_error)?;

    let vector_count = collection_ref.vector_count();
    let is_empty = vector_count == 0;

    let response = json!({
        "status": "success",
        "collection": collection,
        "vector_count": vector_count,
        "is_empty": is_empty
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

// =============================================
// phase40 §2.1 — MCP tools mirroring REST-only endpoints
// =============================================

/// Mirrors `DELETE /collections/{name}`
/// (`rest_handlers::collections::delete_collection`). The REST handler
/// also invalidates the query cache and marks the auto-save manager
/// dirty; those live on `VectorizerServer`, which `handle_mcp_tool`
/// doesn't have access to (only `store`/`embedding_manager`/
/// `cluster_manager`/`upsert_queue`) — the same constraint the existing
/// `cleanup_empty_collections` MCP tool already accepts.
async fn handle_delete_collection(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing name", None))?;

    store.delete_collection(name).map_err(to_mcp_error)?;

    let response = json!({
        "message": format!("Collection '{}' deleted successfully", name)
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

/// Mirrors `POST /embed` (`rest_handlers::vectors::embed_text`): embeds
/// `text` via the requested `model` (falls back to the server's default
/// provider when omitted). An unregistered `model` now maps to the
/// `BadRequest` MCP code instead of silently coercing to BM25.
async fn handle_embed_text(
    request: CallToolRequestParams,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let text = args
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing text", None))?;

    let requested_model = args
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let (model_name, embedding) = match requested_model {
        Some(name) => {
            if !embedding_manager.has_provider(&name) {
                return Err(to_mcp_error(VectorizerError::UnsupportedModel {
                    requested: name,
                    available: embedding_manager.list_providers(),
                }));
            }
            let emb = embedding_manager
                .embed_with_provider(&name, text)
                .map_err(to_mcp_error)?;
            (name, emb)
        }
        None => {
            let default = embedding_manager
                .get_default_provider_name()
                .unwrap_or("bm25")
                .to_string();
            let emb = embedding_manager.embed(text).map_err(to_mcp_error)?;
            (default, emb)
        }
    };
    let dimension = embedding.len();

    let response = json!({
        "embedding": embedding,
        "text": text,
        "dimension": dimension,
        "model": model_name,
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

/// Mirrors `POST /contextual_search`
/// (`rest_handlers::intelligent_search::contextual_search`) — uses the
/// same `RESTAPIHandler`/`ContextualSearchRequest` path the REST route
/// does.
async fn handle_contextual_search(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    use vectorizer::intelligent_search::rest_api::{ContextualSearchRequest, RESTAPIHandler};

    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;

    let collection = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let context_filters = args
        .get("context_filters")
        .and_then(|f| f.as_object())
        .map(|obj| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
            map
        });

    let context_weight = args
        .get("context_weight")
        .and_then(|w| w.as_f64())
        .map(|w| w as f32);

    let context_reranking = args.get("context_reranking").and_then(|r| r.as_bool());

    let max_results = args
        .get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);

    let handler = RESTAPIHandler::new_with_store(store.clone());
    let search_request = ContextualSearchRequest {
        query: query.to_string(),
        collection: collection.to_string(),
        context_filters,
        context_weight,
        context_reranking,
        max_results,
    };

    let response = handler
        .handle_contextual_search(search_request)
        .await
        .map_err(|e| {
            ErrorData::internal_error(format!("Contextual search failed: {}", e.error), None)
        })?;

    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;

    Ok(CallToolResult::success(vec![ContentBlock::text(
        json_response.to_string(),
    )]))
}

/// Mirrors `GET /stats` (`rest_handlers::meta::get_stats`) for MCP
/// callers: aggregate collection/vector counts plus the embedding
/// provider registry. Two REST-only fields are intentionally omitted:
/// `uptime_seconds` (the server start time lives on `VectorizerServer`,
/// not on the `store`/`embedding_manager` pair `handle_mcp_tool`
/// receives) and `default_quantization`/`compression_ratio` (derived by
/// private helpers in `rest_handlers::meta` not reachable from this
/// module).
async fn handle_get_database_stats(
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let collections = store.list_collections();
    let mut total_vectors: usize = 0;
    for name in &collections {
        if let Ok(coll) = store.get_collection(name) {
            total_vectors += coll.vector_count();
        }
    }

    let default_provider = embedding_manager
        .get_default_provider_name()
        .map(|s| s.to_string());
    let providers: Vec<serde_json::Value> = embedding_manager
        .list_providers()
        .into_iter()
        .map(|name| {
            let dimension = embedding_manager.get_provider_dimension(&name).unwrap_or(0);
            let is_default = default_provider.as_deref() == Some(name.as_str());
            json!({
                "name": name,
                "dimension": dimension,
                "default": is_default,
            })
        })
        .collect();

    let response = json!({
        "collections": collections.len(),
        "total_vectors": total_vectors,
        "version": env!("CARGO_PKG_VERSION"),
        "providers": providers,
        "default_provider": default_provider,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

// =============================================
// phase40 §2.2 — Batch operations (mirror REST /batch_* routes)
// =============================================

/// Mirrors `POST /batch_insert`
/// (`rest_handlers::vectors::batch_insert_texts`) using the same
/// embed-then-insert primitives the singular `insert_text` MCP tool
/// uses. Per-item failures are captured in `results` without aborting
/// the batch, matching the REST response shape (`{collection, inserted,
/// failed, count, results}`).
async fn handle_batch_insert_texts(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    upsert_queue: Arc<vectorizer::db::UpsertQueue>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection_name", None))?;

    let texts = args
        .get("texts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing texts array", None))?;

    if texts.is_empty() {
        return Err(ErrorData::invalid_params(
            "texts array must contain at least one entry",
            None,
        ));
    }

    // Issue #263: per-collection admission, mirrors the REST batch path.
    // Held until the batch completes.
    let _ticket = match upsert_queue.try_admit(collection_name) {
        Ok((ticket, _status)) => ticket,
        Err(vectorizer::db::AdmissionError::QueueFull {
            depth,
            hard_limit,
            retry_after_seconds,
        }) => {
            return Err(ErrorData::internal_error(
                format!(
                    "queue_full: collection '{}' depth {} >= hard_limit {} (retry after {}s)",
                    collection_name, depth, hard_limit, retry_after_seconds,
                ),
                Some(json!({
                    "code": "queue_full",
                    "collection": collection_name,
                    "depth": depth,
                    "hard_limit": hard_limit,
                    "retryAfterSeconds": retry_after_seconds,
                })),
            ));
        }
    };

    let mut inserted: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(texts.len());

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
        let metadata = entry.get("metadata").cloned();
        let public_key = entry.get("public_key").and_then(|k| k.as_str());

        let embedding = match embedding_manager.embed(text) {
            Ok(e) => e,
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "client_id": client_id,
                    "status": "error",
                    "error": e.to_string(),
                    "error_type": e.code(),
                }));
                continue;
            }
        };

        let vector_id = client_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let payload_json = metadata.unwrap_or_else(|| json!({}));

        let payload = if let Some(key) = public_key {
            match vectorizer::security::payload_encryption::encrypt_payload(&payload_json, key) {
                Ok(encrypted) => vectorizer::models::Payload::from_encrypted(encrypted),
                Err(e) => {
                    failed += 1;
                    results.push(json!({
                        "index": idx,
                        "client_id": client_id,
                        "status": "error",
                        "error": format!("Encryption failed: {}", e),
                    }));
                    continue;
                }
            }
        } else {
            vectorizer::models::Payload::new(payload_json)
        };

        match store.insert(
            collection_name,
            vec![vectorizer::models::Vector::with_payload(
                vector_id.clone(),
                embedding,
                payload,
            )],
        ) {
            Ok(()) => {
                inserted += 1;
                results.push(json!({
                    "index": idx,
                    "client_id": client_id,
                    "status": "ok",
                    "vector_id": vector_id,
                }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "client_id": client_id,
                    "status": "error",
                    "error": e.to_string(),
                    "error_type": e.code(),
                }));
            }
        }
    }

    let response = json!({
        "collection": collection_name,
        "inserted": inserted,
        "failed": failed,
        "count": texts.len(),
        "results": results,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

/// Mirrors `POST /batch_search`
/// (`rest_handlers::search::batch_search_vectors`): each entry may
/// carry a text `query` (embedded server-side) or a raw `vector`.
/// `limit` is clamped to the same 100-result ceiling the `search`
/// tool's schema declares. Per-query failures are captured without
/// aborting the batch.
async fn handle_batch_search(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    const MAX_BATCH_SEARCH_LIMIT: usize = 100;

    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let queries = args
        .get("queries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing queries array", None))?;

    if queries.is_empty() {
        return Err(ErrorData::invalid_params(
            "queries array must contain at least one entry",
            None,
        ));
    }

    let mut succeeded: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(queries.len());

    for (idx, entry) in queries.iter().enumerate() {
        let limit = (entry.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize)
            .min(MAX_BATCH_SEARCH_LIMIT);

        let embedding = if let Some(vec_arr) = entry.get("vector").and_then(|v| v.as_array()) {
            let mut query_vector = Vec::with_capacity(vec_arr.len());
            let mut bad_entry = false;
            for v in vec_arr {
                match v.as_f64() {
                    Some(f) => query_vector.push(f as f32),
                    None => {
                        bad_entry = true;
                        break;
                    }
                }
            }
            if bad_entry {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "status": "error",
                    "error": "vector entries must be numbers",
                }));
                continue;
            }
            query_vector
        } else if let Some(query) = entry.get("query").and_then(|q| q.as_str()) {
            match embedding_manager.embed(query) {
                Ok(e) => e,
                Err(e) => {
                    failed += 1;
                    results.push(json!({
                        "index": idx,
                        "status": "error",
                        "error": e.to_string(),
                        "error_type": e.code(),
                    }));
                    continue;
                }
            }
        } else {
            failed += 1;
            results.push(json!({
                "index": idx,
                "status": "error",
                "error": format!("entry[{}] missing both `query` and `vector`", idx),
            }));
            continue;
        };

        match store.search(collection_name, &embedding, limit) {
            Ok(hits) => {
                succeeded += 1;
                results.push(json!({
                    "index": idx,
                    "status": "ok",
                    "query": entry.get("query").cloned().unwrap_or(serde_json::Value::Null),
                    "total_results": hits.len(),
                    "results": hits.iter().map(|r| json!({
                        "id": r.id,
                        "score": r.score,
                        "payload": r.payload,
                    })).collect::<Vec<_>>(),
                }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "status": "error",
                    "error": e.to_string(),
                    "error_type": e.code(),
                    "query": entry.get("query").cloned().unwrap_or(serde_json::Value::Null),
                }));
            }
        }
    }

    let response = json!({
        "collection": collection_name,
        "count": queries.len(),
        "succeeded": succeeded,
        "failed": failed,
        "results": results,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

/// Mirrors `POST /batch_update`
/// (`rest_handlers::search::batch_update_vectors`): replaces a
/// vector's dense data and/or payload in bulk. Per-entry failures are
/// captured without aborting the batch.
async fn handle_batch_update(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let updates = args
        .get("updates")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing updates array", None))?;

    if updates.is_empty() {
        return Err(ErrorData::invalid_params(
            "updates array must contain at least one entry",
            None,
        ));
    }

    let collection_dim = store
        .get_collection(collection_name)
        .map_err(to_mcp_error)?
        .config()
        .dimension;

    let mut updated: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(updates.len());

    for (idx, entry) in updates.iter().enumerate() {
        let Some(id) = entry.get("id").and_then(|i| i.as_str()) else {
            failed += 1;
            results.push(json!({
                "index": idx,
                "status": "error",
                "error": "missing `id` field",
            }));
            continue;
        };

        let existing = match store
            .get_collection(collection_name)
            .and_then(|c| c.get_vector(id))
        {
            Ok(v) => v,
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "error",
                    "error": e.to_string(),
                }));
                continue;
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
            Some(p) if !p.is_null() => Some(vectorizer::models::Payload::new(p.clone())),
            Some(_) => None,
            None => existing.payload.clone(),
        };

        let updated_vector = vectorizer::models::Vector {
            id: id.to_string(),
            data: new_data,
            sparse: existing.sparse.clone(),
            payload: new_payload,
            document_id: existing.document_id.clone(),
        };

        match store.update(collection_name, updated_vector) {
            Ok(()) => {
                updated += 1;
                results.push(json!({ "index": idx, "id": id, "status": "ok" }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "error",
                    "error": e.to_string(),
                }));
            }
        }
    }

    let response = json!({
        "collection": collection_name,
        "count": updates.len(),
        "updated": updated,
        "failed": failed,
        "results": results,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}

/// Mirrors `POST /batch_delete`
/// (`rest_handlers::search::batch_delete_vectors`): deletes a list of
/// vector ids from a single collection. Per-id failures are captured
/// without aborting the batch.
async fn handle_batch_delete(
    request: CallToolRequestParams,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;

    let ids = args
        .get("ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing ids array", None))?;

    if ids.is_empty() {
        return Err(ErrorData::invalid_params(
            "ids array must contain at least one entry",
            None,
        ));
    }

    let mut deleted: usize = 0;
    let mut failed: usize = 0;
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(ids.len());

    for (idx, entry) in ids.iter().enumerate() {
        let Some(id) = entry.as_str() else {
            failed += 1;
            results.push(json!({
                "index": idx,
                "status": "error",
                "error": "id must be a string",
            }));
            continue;
        };

        match store.delete(collection_name, id) {
            Ok(()) => {
                deleted += 1;
                results.push(json!({ "index": idx, "id": id, "status": "ok" }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "index": idx,
                    "id": id,
                    "status": "error",
                    "error": e.to_string(),
                }));
            }
        }
    }

    let response = json!({
        "collection": collection_name,
        "count": ids.len(),
        "deleted": deleted,
        "failed": failed,
        "results": results,
    });

    Ok(CallToolResult::success(vec![ContentBlock::text(
        response.to_string(),
    )]))
}
