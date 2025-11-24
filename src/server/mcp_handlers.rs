//! MCP Tool handlers

use std::sync::Arc;

use rmcp::model::{CallToolRequestParam, CallToolResult, Content, ErrorData};
use serde_json::json;

use super::discovery_handlers::*;
use super::file_operations_handlers::*;
use super::graph_handlers::*;
use crate::VectorStore;
use crate::db::graph::RelationshipType;
use crate::db::{HybridScoringAlgorithm, HybridSearchConfig};
use crate::discovery::{
    CollectionRef, Discovery, DiscoveryConfig, ExpansionConfig, expand_queries_baseline,
    filter_collections,
};
use crate::embedding::EmbeddingManager;
use crate::file_operations::{FileListFilter, FileOperations, SortBy, SummaryType};
use crate::intelligent_search::mcp_tools::*;
use crate::models::SparseVector;

/// Helper function to create an embedding manager for a specific collection
fn create_embedding_manager_for_collection(
    embedding_type: &str,
    dimension: usize,
) -> Result<EmbeddingManager, String> {
    let mut manager = EmbeddingManager::new();

    match embedding_type {
        "bm25" => {
            let bm25 = crate::embedding::Bm25Embedding::new(dimension);
            manager.register_provider("bm25".to_string(), Box::new(bm25));
            manager
                .set_default_provider("bm25")
                .map_err(|e| format!("Failed to set BM25 provider: {}", e))?;
        }
        "tfidf" => {
            let tfidf = crate::embedding::TfIdfEmbedding::new(dimension);
            manager.register_provider("tfidf".to_string(), Box::new(tfidf));
            manager
                .set_default_provider("tfidf")
                .map_err(|e| format!("Failed to set TF-IDF provider: {}", e))?;
        }
        "svd" => {
            let svd = crate::embedding::SvdEmbedding::new(dimension, dimension);
            manager.register_provider("svd".to_string(), Box::new(svd));
            manager
                .set_default_provider("svd")
                .map_err(|e| format!("Failed to set SVD provider: {}", e))?;
        }
        "bert" => {
            let bert = crate::embedding::BertEmbedding::new(dimension);
            manager.register_provider("bert".to_string(), Box::new(bert));
            manager
                .set_default_provider("bert")
                .map_err(|e| format!("Failed to set BERT provider: {}", e))?;
        }
        "minilm" => {
            let minilm = crate::embedding::MiniLmEmbedding::new(dimension);
            manager.register_provider("minilm".to_string(), Box::new(minilm));
            manager
                .set_default_provider("minilm")
                .map_err(|e| format!("Failed to set MiniLM provider: {}", e))?;
        }
        "bagofwords" => {
            let bow = crate::embedding::BagOfWordsEmbedding::new(dimension);
            manager.register_provider("bagofwords".to_string(), Box::new(bow));
            manager
                .set_default_provider("bagofwords")
                .map_err(|e| format!("Failed to set BagOfWords provider: {}", e))?;
        }
        "charngram" => {
            let char_ngram = crate::embedding::CharNGramEmbedding::new(dimension, 3);
            manager.register_provider("charngram".to_string(), Box::new(char_ngram));
            manager
                .set_default_provider("charngram")
                .map_err(|e| format!("Failed to set CharNGram provider: {}", e))?;
        }
        _ => {
            // Default to BM25 if unknown type
            let bm25 = crate::embedding::Bm25Embedding::new(dimension);
            manager.register_provider("bm25".to_string(), Box::new(bm25));
            manager
                .set_default_provider("bm25")
                .map_err(|e| format!("Failed to set default BM25 provider: {}", e))?;
        }
    }

    Ok(manager)
}

pub async fn handle_mcp_tool(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    match request.name.as_ref() {
        // Core Collection/Vector Operations
        "list_collections" => handle_list_collections(store).await,
        "create_collection" => handle_create_collection(request, store).await,
        "get_collection_info" => handle_get_collection_info(request, store).await,
        "insert_text" => handle_insert_text(request, store, embedding_manager).await,
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

        // Cluster Operations
        "cluster_list_nodes" => handle_cluster_list_nodes(store, cluster_manager).await,
        "cluster_get_shard_distribution" => {
            handle_cluster_get_shard_distribution(store, cluster_manager).await
        }
        "cluster_rebalance" => handle_cluster_rebalance(request, store, cluster_manager).await,
        "cluster_add_node" => handle_cluster_add_node(request, store, cluster_manager).await,
        "cluster_remove_node" => handle_cluster_remove_node(request, store, cluster_manager).await,
        "cluster_get_node_info" => {
            handle_cluster_get_node_info(request, store, cluster_manager).await
        }

        _ => Err(ErrorData::invalid_params("Unknown tool", None)),
    }
}

// =============================================
// Handler Functions
// =============================================

async fn handle_search_vectors(
    request: CallToolRequestParam,
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
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;

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
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;

    // Search
    let results = store
        .search(collection_name, &embedding, limit)
        .map_err(|e| ErrorData::internal_error(format!("Search failed: {}", e), None))?;

    let response = json!({
        "results": results.iter().map(|r| json!({
            "id": r.id,
            "score": r.score,
            "payload": r.payload
        })).collect::<Vec<_>>(),
        "total": results.len()
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_list_collections(store: Arc<VectorStore>) -> Result<CallToolResult, ErrorData> {
    let collections = store.list_collections();
    let response = json!({
        "collections": collections,
        "total": collections.len()
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

    // Parse graph configuration if provided
    let graph_config = args.get("graph").and_then(|g| {
        if let Some(enabled) = g.get("enabled").and_then(|e| e.as_bool()) {
            if enabled {
                Some(crate::models::GraphConfig {
                    enabled: true,
                    auto_relationship: crate::models::AutoRelationshipConfig::default(),
                })
            } else {
                None
            }
        } else {
            None
        }
    });

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
        storage_type: Some(crate::models::StorageType::Memory),
        graph: graph_config,
        sharding: None,
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
    let embedding = embedding_manager
        .embed(text)
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;

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
        let embedding = embedding_manager
            .embed(text)
            .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;

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
    let response = handler.handle_intelligent_search(tool).await.map_err(|e| {
        ErrorData::internal_error(format!("Intelligent search failed: {}", e), None)
    })?;

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
                let embedding = coll_emb_manager.embed(query).map_err(|e| {
                    ErrorData::internal_error(format!("Embedding failed: {}", e), None)
                })?;
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

async fn handle_hybrid_search(
    request: CallToolRequestParam,
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
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;

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
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;

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
        .map_err(|e| ErrorData::internal_error(format!("Hybrid search failed: {}", e), None))?;

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

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// Cluster Management Handlers
async fn handle_cluster_list_nodes(
    _store: Arc<VectorStore>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    let manager =
        cluster_manager.ok_or_else(|| ErrorData::invalid_params("Cluster not enabled", None))?;

    let nodes = manager.get_nodes();
    let response = json!({
        "nodes": nodes.iter().map(|n| json!({
            "id": n.id.as_str(),
            "address": n.address,
            "grpc_port": n.grpc_port,
            "status": format!("{:?}", n.status),
            "shards": n.shards.iter().map(|s| s.as_u32()).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_cluster_get_shard_distribution(
    _store: Arc<VectorStore>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    let manager =
        cluster_manager.ok_or_else(|| ErrorData::invalid_params("Cluster not enabled", None))?;

    let router = manager.shard_router();
    let nodes = manager.get_nodes();
    let mut distribution = std::collections::HashMap::new();

    for node in nodes {
        let shards = router.get_shards_for_node(&node.id);
        distribution.insert(node.id.as_str().to_string(), shards.len());
    }

    let response = json!({
        "distribution": distribution,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

async fn handle_cluster_rebalance(
    _request: CallToolRequestParam,
    _store: Arc<VectorStore>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    let manager =
        cluster_manager.ok_or_else(|| ErrorData::invalid_params("Cluster not enabled", None))?;

    let router = manager.shard_router();
    let all_nodes: Vec<_> = manager.get_nodes().iter().map(|n| n.id.clone()).collect();
    let all_shards: Vec<_> = all_nodes
        .iter()
        .flat_map(|node_id| router.get_shards_for_node(node_id))
        .collect();
    if !all_nodes.is_empty() && !all_shards.is_empty() {
        router.rebalance(&all_shards, &all_nodes);
    }

    Ok(CallToolResult::success(vec![Content::text(
        json!({"message": "Rebalancing triggered"}).to_string(),
    )]))
}

async fn handle_cluster_add_node(
    request: CallToolRequestParam,
    _store: Arc<VectorStore>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    let manager =
        cluster_manager.ok_or_else(|| ErrorData::invalid_params("Cluster not enabled", None))?;

    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let node_id = args
        .get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing node_id", None))?;
    let address = args
        .get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing address", None))?;
    let grpc_port = args
        .get("grpc_port")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ErrorData::invalid_params("Missing grpc_port", None))?;

    let mut node = crate::cluster::ClusterNode::new(
        crate::cluster::NodeId::new(node_id.to_string()),
        address.to_string(),
        grpc_port as u16,
    );
    node.mark_active();
    manager.add_node(node);

    Ok(CallToolResult::success(vec![Content::text(
        json!({"message": "Node added"}).to_string(),
    )]))
}

async fn handle_cluster_remove_node(
    request: CallToolRequestParam,
    _store: Arc<VectorStore>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    let manager =
        cluster_manager.ok_or_else(|| ErrorData::invalid_params("Cluster not enabled", None))?;

    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let node_id = args
        .get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing node_id", None))?;

    manager.remove_node(&crate::cluster::NodeId::new(node_id.to_string()));

    Ok(CallToolResult::success(vec![Content::text(
        json!({"message": "Node removed"}).to_string(),
    )]))
}

async fn handle_cluster_get_node_info(
    request: CallToolRequestParam,
    _store: Arc<VectorStore>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
) -> Result<CallToolResult, ErrorData> {
    let manager =
        cluster_manager.ok_or_else(|| ErrorData::invalid_params("Cluster not enabled", None))?;

    let args = request
        .arguments
        .as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

    let node_id = args
        .get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing node_id", None))?;

    let node = manager
        .get_node(&crate::cluster::NodeId::new(node_id.to_string()))
        .ok_or_else(|| ErrorData::invalid_params("Node not found", None))?;

    let response = json!({
        "id": node.id.as_str(),
        "address": node.address,
        "grpc_port": node.grpc_port,
        "status": format!("{:?}", node.status),
        "shards": node.shards.iter().map(|s| s.as_u32()).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
