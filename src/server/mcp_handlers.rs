//! MCP Tool handlers

use std::sync::Arc;
use rmcp::model::{CallToolRequestParam, CallToolResult, Content, ErrorData};
use serde_json::json;
use crate::{VectorStore, embedding::EmbeddingManager};
use crate::intelligent_search::mcp_tools::*;
use crate::discovery::{Discovery, DiscoveryConfig, filter_collections, expand_queries_baseline, ExpansionConfig, CollectionRef};
use crate::file_operations::{FileOperations, FileListFilter, SummaryType, SortBy};
use super::discovery_handlers::*;
use super::file_operations_handlers::*;

/// Helper function to create an embedding manager for a specific collection
fn create_embedding_manager_for_collection(embedding_type: &str, dimension: usize) -> Result<EmbeddingManager, String> {
    let mut manager = EmbeddingManager::new();
    
    match embedding_type {
        "bm25" => {
            let bm25 = crate::embedding::Bm25Embedding::new(dimension);
            manager.register_provider("bm25".to_string(), Box::new(bm25));
            manager.set_default_provider("bm25").map_err(|e| format!("Failed to set BM25 provider: {}", e))?;
        },
        "tfidf" => {
            let tfidf = crate::embedding::TfIdfEmbedding::new(dimension);
            manager.register_provider("tfidf".to_string(), Box::new(tfidf));
            manager.set_default_provider("tfidf").map_err(|e| format!("Failed to set TF-IDF provider: {}", e))?;
        },
        "svd" => {
            let svd = crate::embedding::SvdEmbedding::new(dimension, dimension);
            manager.register_provider("svd".to_string(), Box::new(svd));
            manager.set_default_provider("svd").map_err(|e| format!("Failed to set SVD provider: {}", e))?;
        },
        "bert" => {
            let bert = crate::embedding::BertEmbedding::new(dimension);
            manager.register_provider("bert".to_string(), Box::new(bert));
            manager.set_default_provider("bert").map_err(|e| format!("Failed to set BERT provider: {}", e))?;
        },
        "minilm" => {
            let minilm = crate::embedding::MiniLmEmbedding::new(dimension);
            manager.register_provider("minilm".to_string(), Box::new(minilm));
            manager.set_default_provider("minilm").map_err(|e| format!("Failed to set MiniLM provider: {}", e))?;
        },
        "bagofwords" => {
            let bow = crate::embedding::BagOfWordsEmbedding::new(dimension);
            manager.register_provider("bagofwords".to_string(), Box::new(bow));
            manager.set_default_provider("bagofwords").map_err(|e| format!("Failed to set BagOfWords provider: {}", e))?;
        },
        "charngram" => {
            let char_ngram = crate::embedding::CharNGramEmbedding::new(dimension, 3);
            manager.register_provider("charngram".to_string(), Box::new(char_ngram));
            manager.set_default_provider("charngram").map_err(|e| format!("Failed to set CharNGram provider: {}", e))?;
        },
        _ => {
            // Default to BM25 if unknown type
            let bm25 = crate::embedding::Bm25Embedding::new(dimension);
            manager.register_provider("bm25".to_string(), Box::new(bm25));
            manager.set_default_provider("bm25").map_err(|e| format!("Failed to set default BM25 provider: {}", e))?;
        }
    }
    
    Ok(manager)
}

pub async fn handle_mcp_tool(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    match request.name.as_ref() {
        "search_vectors" => handle_search_vectors(request, store, embedding_manager).await,
        "list_collections" => handle_list_collections(store).await,
        "create_collection" => handle_create_collection(request, store).await,
        "get_collection_info" => handle_get_collection_info(request, store).await,
        "delete_collection" => handle_delete_collection(request, store).await,
        "insert_text" => handle_insert_text(request, store, embedding_manager).await,
        "batch_insert_texts" => handle_batch_insert_texts(request, store, embedding_manager).await,
        "embed_text" => handle_embed_text(request, embedding_manager).await,
        "get_vector" => handle_get_vector(request, store).await,
        "delete_vectors" => handle_delete_vectors(request, store).await,
        "update_vector" => handle_update_vector(request, store, embedding_manager).await,
        "health_check" => handle_health_check().await,
        "insert_texts" => handle_insert_texts(request, store, embedding_manager).await,
        "batch_search_vectors" => handle_batch_search_vectors(request, store, embedding_manager).await,
        "batch_update_vectors" => handle_batch_update_vectors(request, store, embedding_manager).await,
        "batch_delete_vectors" => handle_batch_delete_vectors(request, store).await,
        "get_indexing_progress" => handle_get_indexing_progress().await,
        // Intelligent Search Tools
        "intelligent_search" => handle_intelligent_search(request, store, embedding_manager).await,
        "multi_collection_search" => handle_multi_collection_search(request, store, embedding_manager).await,
        "semantic_search" => handle_semantic_search(request, store, embedding_manager).await,
        "contextual_search" => handle_contextual_search(request, store, embedding_manager).await,
        // Discovery Tools (9 functions + 1 full pipeline)
        "discover" => handle_discover(request, store, embedding_manager).await,
        "filter_collections" => handle_filter_collections(request, store).await,
        "score_collections" => handle_score_collections(request, store).await,
        "expand_queries" => handle_expand_queries(request).await,
        "broad_discovery" => handle_broad_discovery(request, store, embedding_manager).await,
        "semantic_focus" => handle_semantic_focus(request, store, embedding_manager).await,
        "promote_readme" => handle_promote_readme(request).await,
        "compress_evidence" => handle_compress_evidence(request).await,
        "build_answer_plan" => handle_build_answer_plan(request).await,
        "render_llm_prompt" => handle_render_llm_prompt(request).await,
        // File Operations Tools
        "get_file_content" => handle_get_file_content(request, store).await,
        "list_files_in_collection" => handle_list_files_in_collection(request, store).await,
        "get_file_summary" => handle_get_file_summary(request, store).await,
        "get_file_chunks_ordered" => handle_get_file_chunks_ordered(request, store).await,
        "get_project_outline" => handle_get_project_outline(request, store).await,
        "get_related_files" => handle_get_related_files(request, store, embedding_manager).await,
        "search_by_file_type" => handle_search_by_file_type(request, store, embedding_manager).await,
        _ => Err(ErrorData::invalid_params("Unknown tool", None)),
    }
}

async fn handle_search_vectors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    _embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection_name = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;
    
    let limit = args.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    // Get the collection to access its embedding type and dimension
    let collection = store.get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;
    
    let embedding_type = collection.get_embedding_type();
    let dimension = collection.config().dimension;
    
    // Create embedding manager specific to this collection
    let collection_embedding_manager = create_embedding_manager_for_collection(&embedding_type, dimension)
        .map_err(|e| ErrorData::internal_error(format!("Failed to create embedding manager: {}", e), None))?;
    
    // Generate embedding using the collection-specific manager
    let embedding = collection_embedding_manager.embed(query)
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
    
    // Search
    let results = store.search(collection_name, &embedding, limit)
        .map_err(|e| ErrorData::internal_error(format!("Search failed: {}", e), None))?;
    
    let response = json!({
        "results": results.iter().map(|r| json!({
            "id": r.id,
            "score": r.score,
            "payload": r.payload
        })).collect::<Vec<_>>(),
        "total": results.len()
    });
    
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_list_collections(store: Arc<VectorStore>) -> Result<CallToolResult, ErrorData> {
    let collections = store.list_collections();
    let response = json!({
        "collections": collections,
        "total": collections.len()
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_create_collection(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing name", None))?;
    
    let dimension = args.get("dimension")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ErrorData::invalid_params("Missing dimension", None))? as usize;
    
    let metric = args.get("metric")
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
    };
    
    store.create_collection(name, config)
        .map_err(|e| ErrorData::internal_error(format!("Failed to create collection: {}", e), None))?;
    
    let response = json!({
        "status": "created",
        "name": name,
        "dimension": dimension,
        "metric": metric
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_get_collection_info(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing name", None))?;
    
    let collection = store.get_collection(name)
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;
    
    let response = json!({
        "name": name,
        "vector_count": collection.vector_count(),
        "dimension": collection.config().dimension,
        "metric": format!("{:?}", collection.config().metric)
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_delete_collection(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing name", None))?;
    
    store.delete_collection(name)
        .map_err(|e| ErrorData::internal_error(format!("Failed to delete collection: {}", e), None))?;
    
    let response = json!({"status": "deleted", "name": name});
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_insert_text(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection_name = args.get("collection_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection_name", None))?;
    
    let text = args.get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing text", None))?;
    
    let metadata = args.get("metadata").cloned();
    
    // Generate embedding
    let embedding = embedding_manager.embed(text)
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
    
    let vector_id = uuid::Uuid::new_v4().to_string();
    let payload = if let Some(meta) = metadata {
        crate::models::Payload::new(meta)
    } else {
        crate::models::Payload::new(json!({}))
    };
    
    store.insert(collection_name, vec![crate::models::Vector::with_payload(
        vector_id.clone(),
        embedding,
        payload
    )]).map_err(|e| ErrorData::internal_error(format!("Insert failed: {}", e), None))?;
    
    let response = json!({
        "status": "inserted",
        "vector_id": vector_id,
        "collection": collection_name
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_batch_insert_texts(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection_name = args.get("collection_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection_name", None))?;
    
    let texts = args.get("texts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing texts array", None))?;
    
    let mut vectors = Vec::new();
    for text_value in texts {
        if let Some(text) = text_value.as_str() {
            let embedding = embedding_manager.embed(text)
                .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
            
            let vector_id = uuid::Uuid::new_v4().to_string();
            vectors.push(crate::models::Vector::new(vector_id, embedding));
        }
    }
    
    store.insert(collection_name, vectors.clone())
        .map_err(|e| ErrorData::internal_error(format!("Batch insert failed: {}", e), None))?;
    
    let response = json!({
        "status": "batch_inserted",
        "collection": collection_name,
        "count": vectors.len()
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_embed_text(
    request: CallToolRequestParam,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let text = args.get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing text", None))?;
    
    let embedding = embedding_manager.embed(text)
        .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
    
    let response = json!({
        "embedding": embedding,
        "dimension": embedding.len(),
        "provider": "bm25"
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_get_vector(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let vector_id = args.get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing vector_id", None))?;
    
    let coll = store.get_collection(collection)
        .map_err(|e| ErrorData::internal_error(format!("Collection not found: {}", e), None))?;
    
    let vector = coll.get_vector(vector_id)
        .map_err(|e| ErrorData::internal_error(format!("Vector not found: {}", e), None))?;
    
    let response = json!({
        "id": vector.id,
        "data": vector.data,
        "payload": vector.payload,
        "collection": collection
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_delete_vectors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let vector_ids = args.get("vector_ids")
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
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_update_vector(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let vector_id = args.get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing vector_id", None))?;
    
    let text = args.get("text").and_then(|v| v.as_str());
    let metadata = args.get("metadata").cloned();
    
    if let Some(text) = text {
        let embedding = embedding_manager.embed(text)
            .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
        
        let payload = if let Some(meta) = metadata {
            crate::models::Payload::new(meta)
        } else {
            crate::models::Payload::new(json!({}))
        };
        
        store.update(collection, crate::models::Vector::with_payload(
            vector_id.to_string(),
            embedding,
            payload
        )).map_err(|e| ErrorData::internal_error(format!("Update failed: {}", e), None))?;
    }
    
    let response = json!({
        "status": "updated",
        "vector_id": vector_id,
        "collection": collection
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_health_check() -> Result<CallToolResult, ErrorData> {
    let response = json!({
        "status": "healthy",
        "service": "vectorizer",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_insert_texts(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let texts = args.get("texts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing texts array", None))?;
    
    let mut vectors = Vec::new();
    for text_obj in texts {
        if let Some(obj) = text_obj.as_object() {
            let id = obj.get("id").and_then(|v| v.as_str())
                .ok_or_else(|| ErrorData::invalid_params("Missing id in text object", None))?;
            let text = obj.get("text").and_then(|v| v.as_str())
                .ok_or_else(|| ErrorData::invalid_params("Missing text in text object", None))?;
            let metadata = obj.get("metadata").cloned();
            
            let embedding = embedding_manager.embed(text)
                .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
            
            let payload = if let Some(meta) = metadata {
                crate::models::Payload::new(meta)
            } else {
                crate::models::Payload::new(json!({}))
            };
            
            vectors.push(crate::models::Vector::with_payload(id.to_string(), embedding, payload));
        }
    }
    
    store.insert(collection, vectors.clone())
        .map_err(|e| ErrorData::internal_error(format!("Insert failed: {}", e), None))?;
    
    let response = json!({
        "status": "inserted",
        "collection": collection,
        "count": vectors.len()
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_batch_search_vectors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let queries = args.get("queries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing queries array", None))?;
    
    let mut all_results = Vec::new();
    
    for query_obj in queries {
        if let Some(obj) = query_obj.as_object() {
            let query = obj.get("query").and_then(|v| v.as_str())
                .ok_or_else(|| ErrorData::invalid_params("Missing query in query object", None))?;
            let limit = obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            
            let embedding = embedding_manager.embed(query)
                .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
            
            let results = store.search(collection, &embedding, limit)
                .map_err(|e| ErrorData::internal_error(format!("Search failed: {}", e), None))?;
            
            all_results.push(json!({
                "query": query,
                "results": results.iter().map(|r| json!({
                    "id": r.id,
                    "score": r.score,
                    "payload": r.payload
                })).collect::<Vec<_>>()
            }));
        }
    }
    
    let response = json!({
        "collection": collection,
        "searches": all_results,
        "total_searches": all_results.len()
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_batch_update_vectors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let updates = args.get("updates")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing updates array", None))?;
    
    let mut updated_count = 0;
    
    for update_obj in updates {
        if let Some(obj) = update_obj.as_object() {
            let vector_id = obj.get("vector_id").and_then(|v| v.as_str())
                .ok_or_else(|| ErrorData::invalid_params("Missing vector_id in update object", None))?;
            
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                let embedding = embedding_manager.embed(text)
                    .map_err(|e| ErrorData::internal_error(format!("Embedding failed: {}", e), None))?;
                
                let metadata = obj.get("metadata").cloned();
                let payload = if let Some(meta) = metadata {
                    crate::models::Payload::new(meta)
                } else {
                    crate::models::Payload::new(json!({}))
                };
                
                if store.update(collection, crate::models::Vector::with_payload(
                    vector_id.to_string(),
                    embedding,
                    payload
                )).is_ok() {
                    updated_count += 1;
                }
            }
        }
    }
    
    let response = json!({
        "status": "updated",
        "collection": collection,
        "count": updated_count
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_batch_delete_vectors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let vector_ids = args.get("vector_ids")
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
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_get_indexing_progress() -> Result<CallToolResult, ErrorData> {
    let response = json!({
        "status": "no_indexing_in_progress",
        "message": "No active indexing operations",
        "collections": []
    });
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

// Intelligent Search Handlers

async fn handle_intelligent_search(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;
    
    let collections = args.get("collections")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>());
    
    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let domain_expansion = args.get("domain_expansion")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let technical_focus = args.get("technical_focus")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let mmr_enabled = args.get("mmr_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let mmr_lambda = args.get("mmr_lambda")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;
    
    let tool = IntelligentSearchTool {
        query: query.to_string(),
        collections,
        max_results: Some(max_results),
        domain_expansion: Some(domain_expansion),
        technical_focus: Some(technical_focus),
        mmr_enabled: Some(mmr_enabled),
        mmr_lambda: Some(mmr_lambda),
    };
    
    // Create handler with collection-specific embedding managers
    let handler = MCPToolHandler::new_with_store(store.clone());
    let response = handler.handle_intelligent_search(tool).await
        .map_err(|e| ErrorData::internal_error(format!("Intelligent search failed: {}", e), None))?;
    
    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;
    
    Ok(CallToolResult::success(vec![Content::text(json_response.to_string())]))
}

async fn handle_multi_collection_search(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;
    
    let collections = args.get("collections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ErrorData::invalid_params("Missing collections", None))?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let max_per_collection = args.get("max_per_collection")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    let max_total_results = args.get("max_total_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;
    
    let cross_collection_reranking = args.get("cross_collection_reranking")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let tool = MultiCollectionSearchTool {
        query: query.to_string(),
        collections,
        max_per_collection: Some(max_per_collection),
        max_total_results: Some(max_total_results),
        cross_collection_reranking: Some(cross_collection_reranking),
    };
    
    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());
    let response = handler.handle_multi_collection_search(tool).await
        .map_err(|e| ErrorData::internal_error(format!("Multi collection search failed: {}", e), None))?;
    
    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;
    
    Ok(CallToolResult::success(vec![Content::text(json_response.to_string())]))
}

async fn handle_semantic_search(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let semantic_reranking = args.get("semantic_reranking")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let cross_encoder_reranking = args.get("cross_encoder_reranking")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let similarity_threshold = args.get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5) as f32;
    
    let tool = SemanticSearchTool {
        query: query.to_string(),
        collection: collection.to_string(),
        max_results: Some(max_results),
        semantic_reranking: Some(semantic_reranking),
        cross_encoder_reranking: Some(cross_encoder_reranking),
        similarity_threshold: Some(similarity_threshold),
    };
    
    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());
    let response = handler.handle_semantic_search(tool).await
        .map_err(|e| ErrorData::internal_error(format!("Semantic search failed: {}", e), None))?;
    
    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;
    
    Ok(CallToolResult::success(vec![Content::text(json_response.to_string())]))
}

async fn handle_contextual_search(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing query", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let context_filters = args.get("context_filters")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<std::collections::HashMap<String, serde_json::Value>>()
        });
    
    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let context_reranking = args.get("context_reranking")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let context_weight = args.get("context_weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.3) as f32;
    
    let tool = ContextualSearchTool {
        query: query.to_string(),
        collection: collection.to_string(),
        context_filters,
        max_results: Some(max_results),
        context_reranking: Some(context_reranking),
        context_weight: Some(context_weight),
    };
    
    let handler = MCPToolHandler::new(store.clone(), embedding_manager.clone());
    let response = handler.handle_contextual_search(tool).await
        .map_err(|e| ErrorData::internal_error(format!("Contextual search failed: {}", e), None))?;
    
    let json_response = serde_json::to_value(response)
        .map_err(|e| ErrorData::internal_error(format!("Serialization failed: {}", e), None))?;
    
    Ok(CallToolResult::success(vec![Content::text(json_response.to_string())]))
}

// =========================
// File Operations Handlers
// =========================

async fn handle_get_file_content(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let file_path = args.get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing file_path", None))?;
    
    let max_size_kb = args.get("max_size_kb")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as usize;
    
    // Initialize FileOperations WITH STORE
    let file_ops = FileOperations::with_store(store);
    
    // Get file content
    let result = file_ops.get_file_content(collection, file_path, max_size_kb)
        .await
        .map_err(|e| ErrorData::internal_error(format!("Failed to get file content: {}", e), None))?;
    
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
    
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_list_files_in_collection(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    // Parse filter parameters
    let filter_by_type = args.get("filter_by_type")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect());
    
    let min_chunks = args.get("min_chunks")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    
    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    
    let sort_by = args.get("sort_by")
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
    let result = file_ops.list_files_in_collection(collection, filter)
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
    
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

async fn handle_get_file_summary(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args = request.arguments.as_ref()
        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
    
    let collection = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing collection", None))?;
    
    let file_path = args.get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing file_path", None))?;
    
    let summary_type = args.get("summary_type")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "extractive" => Some(SummaryType::Extractive),
            "structural" => Some(SummaryType::Structural),
            "both" => Some(SummaryType::Both),
            _ => None,
        })
        .unwrap_or(SummaryType::Both);
    
    let max_sentences = args.get("max_sentences")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    // Initialize FileOperations WITH STORE
    let file_ops = FileOperations::with_store(store);
    
    // Get summary
    let result = file_ops.get_file_summary(collection, file_path, summary_type, max_sentences)
        .await
        .map_err(|e| ErrorData::internal_error(format!("Failed to get file summary: {}", e), None))?;
    
    let mut response = json!({
        "file_path": result.file_path,
        "metadata": {
            "chunk_count": result.metadata.chunk_count,
            "file_type": result.metadata.file_type,
            "summary_method": result.metadata.summary_method,
        },
        "generated_at": result.generated_at,
    });
    
    if let Some(extractive) = result.extractive_summary {
        response["extractive_summary"] = json!(extractive);
    }
    
    if let Some(structural) = result.structural_summary {
        response["structural_summary"] = json!({
            "outline": structural.outline,
            "key_sections": structural.key_sections,
            "key_points": structural.key_points,
        });
    }
    
    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
}

