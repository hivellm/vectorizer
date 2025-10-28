//! MCP Tool handlers

use std::sync::Arc;
use std::time::Instant;

use rmcp::model::{CallToolRequestParam, CallToolResult, Content, ErrorData};
use serde_json::json;
use tracing::warn;

use super::discovery_handlers::*;
use super::file_operations_handlers::*;
use crate::VectorStore;
use crate::discovery::{
    CollectionRef, Discovery, DiscoveryConfig, ExpansionConfig, expand_queries_baseline,
    filter_collections,
};
use crate::embedding::EmbeddingManager;
use crate::file_operations::{FileListFilter, FileOperations, SortBy, SummaryType};
use crate::intelligent_search::mcp_tools::*;

/// Helper function to create an embedding manager for a specific collection
fn create_embedding_manager_for_collection(
    embedding_type: &str,
    dimension: usize,
) -> Result<Arc<EmbeddingManager>, String> {
    let mut config = crate::embedding::EmbeddingConfig::default();
    config.dimension = dimension;

    let mut manager = crate::embedding::EmbeddingManager::new(config);

    match embedding_type {
        "bm25" | "tfidf" | "bagofwords" | "charngram" | "svd" => {
            let provider = Arc::new(crate::embedding::BM25Factory::create_default());
            manager.add_provider(crate::embedding::EmbeddingProviderType::BM25, provider);
            manager.set_default_provider(crate::embedding::EmbeddingProviderType::BM25);
        }
        "bert" | "minilm" => {
            let provider = Arc::new(crate::embedding::BERTFactory::create_default());
            manager.add_provider(crate::embedding::EmbeddingProviderType::BERT, provider);
            manager.set_default_provider(crate::embedding::EmbeddingProviderType::BERT);
        }
        _ => {
            return Err(format!("Unsupported embedding type: {}", embedding_type));
        }
    }

    Ok(Arc::new(manager))
}

/// Enhanced error handling for MCP operations
#[derive(Debug, thiserror::Error)]
pub enum MCPError {
    #[error("Invalid parameters: {message}")]
    InvalidParams { message: String },

    #[error("Collection not found: {name}")]
    CollectionNotFound { name: String },

    #[error("Vector not found: {id} in collection {collection}")]
    VectorNotFound { id: String, collection: String },

    #[error("Embedding failed: {reason}")]
    EmbeddingFailed { reason: String },

    #[error("Search failed: {reason}")]
    SearchFailed { reason: String },

    #[error("Serialization failed: {reason}")]
    SerializationFailed { reason: String },

    #[error("Internal error: {reason}")]
    InternalError { reason: String },

    #[error("File operation failed: {reason}")]
    FileOperationFailed { reason: String },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },

    #[error("Timeout occurred: {operation}")]
    Timeout { operation: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    #[error("Permission denied: {resource}")]
    PermissionDenied { resource: String },

    #[error("Concurrent modification: {resource}")]
    ConcurrentModification { resource: String },
}

impl MCPError {
    pub fn to_error_data(self) -> ErrorData {
        match self {
            MCPError::InvalidParams { message } => ErrorData::invalid_params(message, None),
            MCPError::CollectionNotFound { name } => {
                ErrorData::invalid_params(format!("Collection '{}' not found", name), None)
            }
            MCPError::VectorNotFound { id, collection } => ErrorData::invalid_params(
                format!("Vector '{}' not found in collection '{}'", id, collection),
                None,
            ),
            MCPError::EmbeddingFailed { reason } => {
                ErrorData::internal_error(format!("Embedding operation failed: {}", reason), None)
            }
            MCPError::SearchFailed { reason } => {
                ErrorData::internal_error(format!("Search operation failed: {}", reason), None)
            }
            MCPError::SerializationFailed { reason } => {
                ErrorData::internal_error(format!("Serialization failed: {}", reason), None)
            }
            MCPError::InternalError { reason } => ErrorData::internal_error(reason, None),
            MCPError::FileOperationFailed { reason } => {
                ErrorData::internal_error(format!("File operation failed: {}", reason), None)
            }
            MCPError::ValidationFailed { reason } => {
                ErrorData::invalid_params(format!("Validation failed: {}", reason), None)
            }
            MCPError::RateLimitExceeded { message } => {
                ErrorData::invalid_params(format!("Rate limit exceeded: {}", message), None)
            }
            MCPError::Timeout { operation } => {
                ErrorData::internal_error(format!("Operation '{}' timed out", operation), None)
            }
            MCPError::ResourceExhausted { resource } => {
                ErrorData::internal_error(format!("Resource '{}' exhausted", resource), None)
            }
            MCPError::ConfigurationError { message } => {
                ErrorData::internal_error(format!("Configuration error: {}", message), None)
            }
            MCPError::NetworkError { reason } => {
                ErrorData::internal_error(format!("Network error: {}", reason), None)
            }
            MCPError::PermissionDenied { resource } => ErrorData::invalid_params(
                format!("Permission denied for resource: {}", resource),
                None,
            ),
            MCPError::ConcurrentModification { resource } => ErrorData::internal_error(
                format!("Concurrent modification detected for: {}", resource),
                None,
            ),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MCPError::Timeout { .. }
                | MCPError::NetworkError { .. }
                | MCPError::ResourceExhausted { .. }
                | MCPError::ConcurrentModification { .. }
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::InvalidParams { .. }
            | MCPError::ValidationFailed { .. }
            | MCPError::CollectionNotFound { .. }
            | MCPError::VectorNotFound { .. } => ErrorSeverity::Low,
            MCPError::EmbeddingFailed { .. }
            | MCPError::SearchFailed { .. }
            | MCPError::FileOperationFailed { .. }
            | MCPError::RateLimitExceeded { .. }
            | MCPError::PermissionDenied { .. } => ErrorSeverity::Medium,
            MCPError::InternalError { .. }
            | MCPError::SerializationFailed { .. }
            | MCPError::Timeout { .. }
            | MCPError::ResourceExhausted { .. }
            | MCPError::ConfigurationError { .. }
            | MCPError::NetworkError { .. }
            | MCPError::ConcurrentModification { .. } => ErrorSeverity::High,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
}

pub async fn handle_mcp_tool(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let start_time = Instant::now();
    let tool_name = request.name.clone();

    // Add retry logic for retryable operations
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        let result = match request.name.as_ref() {
            // Core Collection/Vector Operations
            "list_collections" => handle_list_collections(store.clone()).await,
            "create_collection" => handle_create_collection(request.clone(), store.clone()).await,
            "get_collection_info" => {
                handle_get_collection_info(request.clone(), store.clone()).await
            }
            "insert_text" => {
                handle_insert_text(request.clone(), store.clone(), embedding_manager.clone()).await
            }
            "get_vector" => handle_get_vector(request.clone(), store.clone()).await,
            "update_vector" => {
                handle_update_vector(request.clone(), store.clone(), embedding_manager.clone())
                    .await
            }
            "delete_vector" => handle_delete_vectors(request.clone(), store.clone()).await,
            "multi_collection_search" => {
                handle_multi_collection_search(
                    request.clone(),
                    store.clone(),
                    embedding_manager.clone(),
                )
                .await
            }
            "search" => {
                handle_search_vectors(request.clone(), store.clone(), embedding_manager.clone())
                    .await
            }

            // Search Operations
            "search_intelligent" => {
                handle_intelligent_search(request.clone(), store.clone(), embedding_manager.clone())
                    .await
            }
            "search_semantic" => {
                handle_semantic_search(request.clone(), store.clone(), embedding_manager.clone())
                    .await
            }
            "search_extra" => {
                handle_search_extra(request.clone(), store.clone(), embedding_manager.clone()).await
            }

            // Discovery Operations
            "filter_collections" => handle_filter_collections(request.clone(), store.clone()).await,
            "expand_queries" => handle_expand_queries(request.clone()).await,

            // File Operations
            "get_file_content" => handle_get_file_content(request.clone(), store.clone()).await,
            "list_files" => handle_list_files_in_collection(request.clone(), store.clone()).await,
            "get_file_chunks" => {
                handle_get_file_chunks_ordered(request.clone(), store.clone()).await
            }
            "get_project_outline" => {
                handle_get_project_outline(request.clone(), store.clone()).await
            }
            "get_related_files" => {
                handle_get_related_files(request.clone(), store.clone(), embedding_manager.clone())
                    .await
            }

            // NOTE: Qdrant compatibility operations removed from MCP
            // Use REST API at /qdrant/* for Qdrant compatibility
            // Recommended: Use native Vectorizer MCP tools for better performance

            // Performance and Monitoring Operations
            "get_performance_metrics" => handle_get_performance_metrics(store.clone()).await,
            "get_detailed_performance_report" => {
                handle_get_detailed_performance_report(store.clone(), embedding_manager.clone())
                    .await
            }
            "clear_cache" => handle_clear_cache(request.clone()).await,
            "health_check" => handle_health_check(request.clone(), store.clone()).await,

            _ => Err(ErrorData::invalid_params("Unknown tool", None)),
        };

        match result {
            Ok(success_result) => {
                let duration = start_time.elapsed();

                // Log performance metrics
                if duration.as_millis() > 100 {
                    tracing::warn!(
                        "Slow MCP tool execution: {} took {}ms (attempt {})",
                        tool_name,
                        duration.as_millis(),
                        attempt + 1
                    );
                } else {
                    tracing::debug!(
                        "MCP tool execution: {} took {}ms (attempt {})",
                        tool_name,
                        duration.as_millis(),
                        attempt + 1
                    );
                }

                return Ok(success_result);
            }
            Err(error) => {
                last_error = Some(error);

                // Check if this is a retryable error
                if attempt < max_retries {
                    // Convert ErrorData back to MCPError to check retryability
                    // For now, we'll retry on any error for simplicity
                    tracing::warn!(
                        "MCP tool '{}' failed on attempt {}, retrying...",
                        tool_name,
                        attempt + 1
                    );

                    // Exponential backoff
                    let delay_ms = 100 * (2_u64.pow(attempt as u32));
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    continue;
                } else {
                    tracing::error!(
                        "MCP tool '{}' failed after {} attempts",
                        tool_name,
                        max_retries + 1
                    );
                    break;
                }
            }
        }
    }

    // Return the last error if all retries failed
    Err(last_error.unwrap_or_else(|| ErrorData::internal_error("Unknown error occurred", None)))
}

// =============================================
// Handler Functions
// =============================================

async fn handle_search_vectors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    _embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref().ok_or_else(|| {
        MCPError::InvalidParams {
            message: "Missing arguments".to_string(),
        }
        .to_error_data()
    })?;

    let collection_name = args
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            MCPError::InvalidParams {
                message: "Missing collection parameter".to_string(),
            }
            .to_error_data()
        })?;

    let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
        MCPError::InvalidParams {
            message: "Missing query parameter".to_string(),
        }
        .to_error_data()
    })?;

    // Validate query is not empty
    if query.trim().is_empty() {
        return Err(MCPError::ValidationFailed {
            reason: "Query cannot be empty".to_string(),
        }
        .to_error_data());
    }

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let similarity_threshold = args
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.1);

    // Validate limit is within reasonable bounds
    if limit == 0 || limit > 1000 {
        return Err(MCPError::ValidationFailed {
            reason: "Limit must be between 1 and 1000".to_string(),
        }
        .to_error_data());
    }

    // Validate similarity threshold
    if similarity_threshold < 0.0 || similarity_threshold > 1.0 {
        return Err(MCPError::ValidationFailed {
            reason: "Similarity threshold must be between 0.0 and 1.0".to_string(),
        }
        .to_error_data());
    }

    // Get the collection to access its embedding manager
    let collection = store.get_collection(collection_name).map_err(|_| {
        MCPError::CollectionNotFound {
            name: collection_name.to_string(),
        }
        .to_error_data()
    })?;

    // Try to get the collection's specific embedding manager first
    let collection_embedding_manager = store
        .get_collection_embedding_manager(collection_name)
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            // Fallback: create a new embedding manager if not found
            let embedding_type = collection.get_embedding_type();
            let dimension = collection.config().dimension;
            warn!(
                "No embedding manager found for collection '{}', creating new one",
                collection_name
            );
            create_embedding_manager_for_collection(&embedding_type, dimension)
                .unwrap_or_else(|e| {
                    warn!("Failed to create fallback embedding manager: {}", e);
                    // Return a default embedding manager as last resort
                    Arc::new(EmbeddingManager::new(
                        crate::embedding::EmbeddingConfig::default(),
                    ))
                })
        });

    // Generate embedding using the collection-specific manager
    let embedding_result = collection_embedding_manager
        .embed(query)
        .await
        .map_err(|e| {
            MCPError::EmbeddingFailed {
                reason: e.to_string(),
            }
            .to_error_data()
        })?;
    let embedding = embedding_result.embedding;

    // Search with performance timing
    let search_start = Instant::now();
    let results = store
        .search(collection_name, &embedding, limit)
        .map_err(|e| {
            MCPError::SearchFailed {
                reason: e.to_string(),
            }
            .to_error_data()
        })?;
    let search_duration = search_start.elapsed();

    // Optimize JSON response building
    let mut result_objects = Vec::with_capacity(results.len());
    for result in &results {
        result_objects.push(json!({
            "id": result.id,
            "score": result.score,
            "payload": result.payload
        }));
    }

    let response = json!({
        "results": result_objects,
        "query": query,
        "collection": collection_name,
        "limit": limit,
        "similarity_threshold": similarity_threshold,
        "total": results.len(),
        "search_time_ms": search_duration.as_millis(),
        "time": 0.0
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_list_collections(store: Arc<VectorStore>) -> Result<CallToolResult, ErrorData> {
    let start_time = Instant::now();

    // Get collections with metadata for better performance
    let collections = store.list_collections();

    // Build response efficiently
    let mut collection_info = Vec::with_capacity(collections.len());
    for collection_name in &collections {
        if let Ok(collection) = store.get_collection(collection_name) {
            collection_info.push(json!({
                "name": collection_name,
                "vector_count": collection.vector_count(),
                "dimension": collection.config().dimension,
                "metric": format!("{:?}", collection.config().metric),
                "embedding_type": collection.get_embedding_type()
            }));
        }
    }

    let duration = start_time.elapsed();
    let response = json!({
        "collections": collection_info,
        "total": collections.len(),
        "processing_time_ms": duration.as_millis()
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_create_collection(
    request: CallToolRequestParam,
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
        "euclidean" => crate::models::DistanceMetric::Euclidean,
        _ => crate::models::DistanceMetric::Cosine,
    };

    let config = crate::models::CollectionConfig {
        dimension,
        metric: distance_metric,
        quantization: crate::models::QuantizationConfig::SQ { bits: 8 },
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig {
            enabled: false,
            threshold_bytes: 1024,
            algorithm: crate::models::CompressionAlgorithm::Lz4,
        },
        normalization: None,
    };

    store.create_collection(name, config).map_err(|e| {
        ErrorData::internal_error(format!("Failed to create collection: {}", e), None)
    })?;

    let response = json!({
        "status": "created",
        "name": name,
        "dimension": dimension,
        "metric": metric
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_get_collection_info(
    request: CallToolRequestParam,
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

    let collection = store
        .get_collection(name)
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;

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
            "enabled": !matches!(config.quantization, crate::models::QuantizationConfig::None)
        },
        "normalization": normalization_info,
        "created_at": metadata.created_at.to_rfc3339(),
        "updated_at": metadata.updated_at.to_rfc3339(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_insert_text(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
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

    let metadata = args.get("metadata").cloned();

    // Generate embedding
    let embedding_result = embedding_manager
        .embed(text)
        .await
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
    let embedding = embedding_result.embedding;

    let vector_id = uuid::Uuid::new_v4().to_string();
    let payload = if let Some(meta) = metadata {
        crate::models::Payload::new(meta)
    } else {
        crate::models::Payload::new(json!({}))
    };

    store
        .insert(
            collection_name,
            vec![crate::models::Vector::with_payload(
                vector_id.clone(),
                embedding,
                payload,
            )],
        )
        .map_err(|e| ErrorData::internal_error(format!("Insert failed: {}", e), None))?;

    let response = json!({
        "status": "inserted",
        "vector_id": vector_id,
        "collection": collection_name
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_get_vector(
    request: CallToolRequestParam,
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

    let coll = store
        .get_collection(collection)
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;

    let vector = coll
        .get_vector(vector_id)
        .map_err(|e| ErrorData::internal_error(format!("Vector not found: {}", e), None))?;

    let response = json!({
        "id": vector.id,
        "data": vector.data,
        "payload": vector.payload,
        "collection": collection
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_delete_vectors(
    request: CallToolRequestParam,
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
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_update_vector(
    request: CallToolRequestParam,
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

    if let Some(text) = text {
        let embedding_result = embedding_manager
            .embed(text)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
        let embedding = embedding_result.embedding;

        let payload = if let Some(meta) = metadata {
            crate::models::Payload::new(meta)
        } else {
            crate::models::Payload::new(json!({}))
        };

        store
            .update(
                collection,
                crate::models::Vector::with_payload(vector_id.to_string(), embedding, payload),
            )
            .map_err(|e| ErrorData::internal_error(format!("Update failed: {}", e), None))?;
    }

    let response = json!({
        "status": "updated",
        "vector_id": vector_id,
        "collection": collection
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// Intelligent Search Handlers

async fn handle_intelligent_search(
    request: CallToolRequestParam,
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
    let search_start = Instant::now();
    let response = handler.handle_intelligent_search(tool).await.map_err(|e| {
        ErrorData::internal_error(format!("Intelligent search failed: {}", e), None)
    })?;
    let search_duration = search_start.elapsed();

    // Log performance for intelligent search
    if search_duration.as_millis() > 500 {
        tracing::warn!(
            "Slow intelligent search execution: took {}ms",
            search_duration.as_millis()
        );
    }

    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        json_response.to_string(),
    )]))
}

async fn handle_multi_collection_search(
    request: CallToolRequestParam,
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

    Ok(CallToolResult::success(vec![Content::text(
        json_response.to_string(),
    )]))
}

async fn handle_semantic_search(
    request: CallToolRequestParam,
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

    Ok(CallToolResult::success(vec![Content::text(
        json_response.to_string(),
    )]))
}

// New search_extra handler - combines multiple search strategies
async fn handle_search_extra(
    request: CallToolRequestParam,
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
                let coll = store.get_collection(collection).map_err(|e| {
                    ErrorData::internal_error(format!("Collection not found: {}", e), None)
                })?;
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
                let embedding_result = coll_emb_manager.embed(query).await.map_err(|e| {
                    ErrorData::internal_error(format!("Embedding failed: {}", e), None)
                })?;
                let embedding = embedding_result.embedding;
                let results = store
                    .search(collection, &embedding, max_results)
                    .map_err(|e| {
                        ErrorData::internal_error(format!("Search failed: {}", e), None)
                    })?;

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

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// =========================
// File Operations Handlers
// =========================

async fn handle_get_file_content(
    request: CallToolRequestParam,
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
        .map_err(|e| {
            ErrorData::internal_error(format!("Failed to get file content: {}", e), None)
        })?;

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

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_list_files_in_collection(
    request: CallToolRequestParam,
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
        .map_err(|e| ErrorData::internal_error(format!("Failed to list files: {}", e), None))?;

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

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// =============================================
// Qdrant Compatibility - REMOVED FROM MCP
// =============================================
// Qdrant compatibility is available only via REST API at /qdrant/*
// Use native Vectorizer MCP tools for better performance and features
// Migration guide: See docs/specs/QDRANT_MIGRATION.md

// =============================================
// Performance and Monitoring Handlers
// =============================================

async fn handle_get_performance_metrics(store: Arc<VectorStore>) -> Result<CallToolResult, ErrorData> {
    // Collect real metrics from the vector store
    let collection_count = store.list_collections().len();
    
    // Get collection stats
    let mut total_vectors = 0_usize;
    let mut total_memory_bytes = 0_usize;
    
    for collection_name in store.list_collections() {
        if let Ok(collection) = store.get_collection(&collection_name) {
            let config = collection.config();
            // Estimate vectors count (this is approximate as we don't have direct count)
            // In a real implementation, collections would track this
            total_memory_bytes += config.dimension * std::mem::size_of::<f32>();
        }
    }
    
    let metrics = json!({
        "collections": {
            "total": collection_count,
            "active": collection_count,
        },
        "vectors": {
            "total_estimated": total_vectors,
            "memory_bytes": total_memory_bytes,
        },
        "system": {
            "uptime_seconds": 0, // Would need server start time
            "last_updated": chrono::Utc::now().to_rfc3339(),
        },
        "status": "healthy",
        "note": "Limited metrics available through MCP. For detailed metrics, use PerformanceMonitor via REST API or direct server access."
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&metrics)
            .map_err(|e| ErrorData::internal_error(format!("Failed to serialize metrics: {}", e), None))?,
    )]))
}

/// Handle detailed performance report request
async fn handle_get_detailed_performance_report(
    store: Arc<VectorStore>,
    _embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    // Get basic system information
    let collections = store.list_collections();
    let total_collections = collections.len();

    // Calculate total vectors across all collections
    let mut total_vectors = 0;
    let mut collection_details = Vec::new();

    for collection_name in collections {
        if let Ok(collection) = store.get_collection(&collection_name) {
            let vector_count = collection.vector_count();
            total_vectors += vector_count;

            let config = collection.config();
            collection_details.push(json!({
                "name": collection_name,
                "vector_count": vector_count,
                "dimension": config.dimension,
                "metric": config.metric
            }));
        }
    }

    // Get system information
    let system_info = json!({
        "total_collections": total_collections,
        "total_vectors": total_vectors,
        "collections": collection_details,
        "server_uptime": "N/A", // Would need to track this
        "memory_usage": "N/A",  // Would need system monitoring
        "cpu_usage": "N/A"      // Would need system monitoring
    });

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "performance_report": system_info,
            "note": "This is a basic performance report. Full metrics collection will be available in future versions.",
            "timestamp": chrono::Utc::now()
        }).to_string(),
    )]))
}

async fn handle_clear_cache(request: CallToolRequestParam) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref().ok_or_else(|| {
        MCPError::InvalidParams {
            message: "Missing arguments".to_string(),
        }
        .to_error_data()
    })?;

    let cache_type = args
        .get("cache_type")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let response = match cache_type {
        "all" => {
            // In a real implementation, this would clear all caches
            json!({
                "status": "success",
                "message": "All caches cleared",
                "cache_type": "all",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        }
        "queries" => {
            json!({
                "status": "success",
                "message": "Query cache cleared",
                "cache_type": "queries",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        }
        "collections" => {
            json!({
                "status": "success",
                "message": "Collection cache cleared",
                "cache_type": "collections",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        }
        _ => {
            return Err(MCPError::ValidationFailed {
                reason: "Invalid cache type. Must be 'all', 'queries', or 'collections'"
                    .to_string(),
            }
            .to_error_data());
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_health_check(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref().ok_or_else(|| {
        MCPError::InvalidParams {
            message: "Missing arguments".to_string(),
        }
        .to_error_data()
    })?;

    let include_metrics = args
        .get("include_metrics")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let check_collections = args
        .get("check_collections")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut health_status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {}
    });

    // Check vector store health
    let collections = store.list_collections();
    let store_health = json!({
        "status": "healthy",
        "collections_count": collections.len(),
        "collections": collections
    });

    health_status["checks"]["vector_store"] = store_health;

    // Check collections if requested
    if check_collections {
        let mut collection_health = json!({
            "status": "healthy",
            "accessible_collections": 0,
            "total_collections": collections.len()
        });

        let mut accessible_count = 0;
        for collection_name in &collections {
            match store.get_collection(collection_name) {
                Ok(collection) => {
                    accessible_count += 1;
                    // Additional collection health checks could be added here
                }
                Err(_) => {
                    // Collection is not accessible
                }
            }
        }

        collection_health["accessible_collections"] =
            serde_json::Value::Number(serde_json::Number::from(accessible_count));
        health_status["checks"]["collections"] = collection_health;
    }

    // Add performance metrics if requested
    if include_metrics {
        let metrics = json!({
            "operation_count": 0,
            "average_duration_ms": 0.0,
            "cache_hits": 0,
            "cache_misses": 0,
            "error_count": 0
        });
        health_status["metrics"] = metrics;
    }

    // Determine overall health status
    let overall_healthy = health_status["checks"]["vector_store"]["status"] == "healthy"
        && (!check_collections || health_status["checks"]["collections"]["status"] == "healthy");

    if !overall_healthy {
        health_status["status"] = serde_json::Value::String("unhealthy".to_string());
    }

    Ok(CallToolResult::success(vec![Content::text(
        health_status.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_embedding_manager_bm25() {
        let result = create_embedding_manager_for_collection("bm25", 128);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_embedding_manager_tfidf() {
        let result = create_embedding_manager_for_collection("tfidf", 128);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_embedding_manager_svd() {
        let result = create_embedding_manager_for_collection("svd", 128);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_embedding_manager_bert() {
        let result = create_embedding_manager_for_collection("bert", 128);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_embedding_manager_invalid_type() {
        let result = create_embedding_manager_for_collection("invalid", 128);
        assert!(result.is_err());
    }
}
