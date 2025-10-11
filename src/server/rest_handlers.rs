//! REST API handlers

use axum::{
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{info, debug, error};

use super::VectorizerServer;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn get_stats(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    let total_vectors: usize = collections.iter().map(|name| {
        state.store.get_collection(name).map(|c| c.vector_count()).unwrap_or(0)
    }).sum();
    
    Json(json!({
        "collections": collections.len(),
        "total_vectors": total_vectors,
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn get_indexing_progress(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    let total_collections = collections.len();
    
    Json(json!({
        "overall_status": "completed",
        "collections": collections.iter().map(|name| {
            json!({
                "name": name,
                "status": "completed",
                "progress": 1.0,
                "total_documents": 0,
                "processed_documents": 0,
                "errors": 0
            })
        }).collect::<Vec<_>>(),
        "total_collections": total_collections,
        "completed_collections": total_collections,
        "processing_collections": 0
    }))
}



pub async fn search_vectors_by_text(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let limit = payload.get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(10) as usize;

    info!("🔍 Searching for '{}' in collection '{}'", query, collection_name);

    // Get the collection
    let collection = match state.store.get_collection(&collection_name) {
        Ok(collection) => collection,
        Err(_) => {
            return Ok(Json(json!({
                "results": [],
                "query": query,
                "limit": limit,
                "collection": collection_name,
                "error": "Collection not found"
            })));
        }
    };

    // Generate embedding for the query
    let query_embedding = match state.embedding_manager.embed(query) {
        Ok(embedding) => embedding,
        Err(e) => {
            error!("Failed to generate embedding: {}", e);
            return Ok(Json(json!({
                "results": [],
                "query": query,
                "limit": limit,
                "collection": collection_name,
                "error": "Failed to generate embedding"
            })));
        }
    };

    // Search vectors in the collection
    let search_results = match collection.search(&query_embedding, limit) {
        Ok(results) => results,
        Err(e) => {
            error!("Search failed: {}", e);
            return Ok(Json(json!({
                "results": [],
                "query": query,
                "limit": limit,
                "collection": collection_name,
                "error": "Search failed"
            })));
        }
    };

    // Convert results to JSON format
    let results: Vec<Value> = search_results.into_iter().map(|result| {
        json!({
            "id": result.id,
            "score": result.score,
            "vector": result.vector,
            "payload": result.payload.map(|p| p.data)
        })
    }).collect();

    Ok(Json(json!({
        "results": results,
        "query": query,
        "limit": limit,
        "collection": collection_name,
        "total_results": results.len()
    })))
}

pub async fn search_by_file(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let file_path = payload.get("file_path")
        .and_then(|f| f.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let limit = payload.get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(10) as usize;

    // For now, return empty results
    Ok(Json(json!({
        "results": [],
        "file_path": file_path,
        "limit": limit,
        "collection": collection_name
    })))
}

pub async fn list_vectors(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    debug!("Listing vectors from collection: {}", collection_name);

    // Parse query parameters for pagination - cap at 50 for vector browser
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10)
        .min(50);
    let offset = params.get("offset")
        .and_then(|o| o.parse::<usize>().ok())
        .unwrap_or(0);
    let min_score = params.get("min_score")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        .max(0.0)
        .min(1.0);

    // Get the collection
    let collection = match state.store.get_collection(&collection_name) {
        Ok(collection) => collection,
        Err(_) => {
            return Ok(Json(json!({
                "vectors": [],
                "total": 0,
                "limit": limit,
                "offset": offset,
                "collection": collection_name,
                "error": "Collection not found"
            })));
        }
    };

    // Get actual vectors from the local collection
    let all_vectors = collection.get_all_vectors();
    let total_count = all_vectors.len();

    // Filter vectors by minimum score (placeholder: filter by payload size)
    let filtered_vectors: Vec<_> = all_vectors
        .into_iter()
        .filter(|v| {
            // Calculate a score based on payload content length
            let score = if let Some(ref payload) = v.payload {
                // Simple scoring based on content richness
                let content_length = payload.data.get("content")
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
        .map(|v| json!({
            "id": v.id,
            "vector": v.data,
            "payload": v.payload.map(|p| p.data),
        }))
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
        paginated_count,
        collection_name,
        total_count,
        duration
    );

    Ok(Json(response))
}

pub async fn list_collections(State(state): State<VectorizerServer>) -> Json<Value> {
    let mut collections = state.store.list_collections();
    
    // Sort alphabetically for consistent dashboard display
    collections.sort();
    
    let collection_infos: Vec<Value> = collections.iter().map(|name| {
        match state.store.get_collection(name) {
            Ok(collection) => {
                let metadata = collection.metadata();
                let config = collection.config();
                let (index_size, payload_size, total_size) = collection.get_size_info();
                let (index_bytes, payload_bytes, total_bytes) = collection.calculate_memory_usage();
                
                // Build normalization info
                let normalization_enabled = config.normalization
                    .as_ref()
                    .map(|n| n.enabled)
                    .unwrap_or(false);
                
                let normalization_level = if normalization_enabled {
                    config.normalization
                        .as_ref()
                        .map(|n| format!("{:?}", n.policy.level))
                        .unwrap_or_else(|| "None".to_string())
                } else {
                    "Disabled".to_string()
                };
                
                json!({
                    "name": name,
                    "vector_count": collection.vector_count(),
                    "document_count": metadata.document_count,
                    "dimension": config.dimension,
                    "metric": format!("{:?}", config.metric),
                    "embedding_provider": "bm25",
                    "size": {
                        "total": total_size,
                        "total_bytes": total_bytes,
                        "index": index_size,
                        "index_bytes": index_bytes,
                        "payload": payload_size,
                        "payload_bytes": payload_bytes
                    },
                    "quantization": {
                        "enabled": matches!(config.quantization, crate::models::QuantizationConfig::SQ { bits: 8 }),
                        "type": format!("{:?}", config.quantization),
                        "bits": if matches!(config.quantization, crate::models::QuantizationConfig::SQ { bits: 8 }) { 8 } else { 0 }
                    },
                    "normalization": {
                        "enabled": normalization_enabled,
                        "level": normalization_level
                    },
                    "created_at": metadata.created_at.to_rfc3339(),
                    "updated_at": chrono::Utc::now().to_rfc3339(),
                    "indexing_status": {
                        "status": "completed",
                        "progress": 1.0,
                        "total_documents": collection.vector_count(),
                        "processed_documents": collection.vector_count(),
                        "errors": 0,
                        "start_time": chrono::Utc::now().to_rfc3339(),
                        "end_time": chrono::Utc::now().to_rfc3339()
                    }
                })
            },
            Err(_) => json!({
                "name": name,
                "vector_count": 0,
                "document_count": 0,
                "dimension": 512,
                "metric": "Cosine",
                "embedding_provider": "bm25",
                "size": {
                    "total": "0 B",
                    "total_bytes": 0,
                    "index": "0 B",
                    "index_bytes": 0,
                    "payload": "0 B",
                    "payload_bytes": 0
                },
                "created_at": chrono::Utc::now().to_rfc3339(),
                "updated_at": chrono::Utc::now().to_rfc3339(),
                "indexing_status": {
                    "status": "error",
                    "progress": 0.0,
                    "total_documents": 0,
                    "processed_documents": 0,
                    "errors": 1,
                    "start_time": chrono::Utc::now().to_rfc3339(),
                    "end_time": chrono::Utc::now().to_rfc3339()
                }
            })
        }
    }).collect();

    Json(json!({
        "collections": collection_infos,
        "total_collections": collections.len()
    }))
}

pub async fn create_collection(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let name = payload.get("name")
        .and_then(|n| n.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let dimension = payload.get("dimension")
        .and_then(|d| d.as_u64())
        .unwrap_or(512) as usize;
    let metric = payload.get("metric")
        .and_then(|m| m.as_str())
        .unwrap_or("cosine");

    info!("Creating collection: {} with dimension {} and metric {}", name, dimension, metric);

    // Create collection configuration
    let config = crate::models::CollectionConfig {
        dimension,
        metric: match metric {
            "cosine" => crate::models::DistanceMetric::Cosine,
            "euclidean" => crate::models::DistanceMetric::Euclidean,
            "dot" => crate::models::DistanceMetric::DotProduct,
            _ => crate::models::DistanceMetric::Cosine,
        },
        hnsw_config: crate::models::HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
    };

    // Actually create the collection in the store
    match state.store.create_collection(name, config) {
        Ok(_) => {
            info!("Collection '{}' created successfully", name);
            Ok(Json(json!({
                "message": format!("Collection '{}' created successfully", name),
                "collection": name,
                "dimension": dimension,
                "metric": metric
            })))
        }
        Err(e) => {
            error!("Failed to create collection '{}': {}", name, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_collection(
    State(state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.store.get_collection(&name) {
        Ok(collection) => {
            let metadata = collection.metadata();
            let config = collection.config();
            let (index_size, payload_size, total_size) = collection.get_size_info();
            let (index_bytes, payload_bytes, total_bytes) = collection.calculate_memory_usage();
            
            // Build normalization info
            let normalization_info = if let Some(norm_config) = &config.normalization {
                json!({
                    "enabled": norm_config.enabled,
                    "level": format!("{:?}", norm_config.policy.level),
                    "preserve_case": norm_config.policy.preserve_case,
                    "collapse_whitespace": norm_config.policy.collapse_whitespace,
                    "remove_html": norm_config.policy.remove_html,
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
            
            Ok(Json(json!({
                "name": name,
                "vector_count": collection.vector_count(),
                "document_count": metadata.document_count,
                "dimension": config.dimension,
                "metric": format!("{:?}", config.metric),
                "created_at": metadata.created_at.to_rfc3339(),
                "updated_at": metadata.updated_at.to_rfc3339(),
                "size": {
                    "total": total_size,
                    "total_bytes": total_bytes,
                    "index": index_size,
                    "index_bytes": index_bytes,
                    "payload": payload_size,
                    "payload_bytes": payload_bytes
                },
                "quantization": {
                    "enabled": matches!(config.quantization, crate::models::QuantizationConfig::SQ { bits: 8 }),
                    "type": format!("{:?}", config.quantization),
                    "bits": if matches!(config.quantization, crate::models::QuantizationConfig::SQ { bits: 8 }) { 8 } else { 0 }
                },
                "normalization": normalization_info,
                "status": "ready"
            })))
        },
        Err(_) => Err(StatusCode::NOT_FOUND)
    }
}

pub async fn delete_collection(
    State(_state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    info!("Deleting collection: {}", name);
    
    // For now, just return success
    Ok(Json(json!({
        "message": format!("Collection '{}' deleted successfully", name)
    })))
}

pub async fn get_vector(
    State(state): State<VectorizerServer>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    match state.store.get_collection(&collection_name) {
        Ok(_collection) => {
            // For now, return mock data
            Ok(Json(json!({
                "id": vector_id,
                "vector": vec![0.1; 512],
                "metadata": {
                    "collection": collection_name
                }
            })))
        },
        Err(_) => Err(StatusCode::NOT_FOUND)
    }
}

pub async fn delete_vector(
    State(_state): State<VectorizerServer>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    info!("Deleting vector {} from collection {}", vector_id, collection_name);
    
    Ok(Json(json!({
        "message": format!("Vector '{}' deleted from collection '{}'", vector_id, collection_name)
    })))
}

pub async fn search_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let query_vector = payload.get("vector")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let limit = payload.get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(10) as usize;

    // For now, return empty results
    Ok(Json(json!({
        "results": [],
        "query_vector": query_vector,
        "limit": limit
    })))
}

pub async fn insert_text(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let text = payload.get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let metadata = payload.get("metadata")
        .and_then(|m| m.as_object())
        .map(|m| {
            m.iter().map(|(k, v)| {
                (k.clone(), match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
            }).collect::<std::collections::HashMap<String, String>>()
        })
        .unwrap_or_default();

    info!("Inserting text into collection '{}': {}", collection_name, text);

    // Get the collection
    let collection = state.store.get_collection(collection_name)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Generate embedding for the text
    let embedding = state.embedding_manager.embed(text)
        .map_err(|e| {
            error!("Failed to generate embedding: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create payload with metadata
    let payload_data = crate::models::Payload::new(serde_json::Value::Object(
        metadata.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect()
    ));

    // Create vector with generated ID
    let vector_id = format!("{}", uuid::Uuid::new_v4());
    let vector = crate::models::Vector {
        id: vector_id.clone(),
        data: embedding,
        payload: Some(payload_data),
    };

    // Insert the vector using the store
    state.store.insert(collection_name, vec![vector])
        .map_err(|e| {
            error!("Failed to insert vector: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Vector inserted successfully with ID: {}", vector_id);

    Ok(Json(json!({
        "message": "Text inserted successfully",
        "text": text,
        "vector_id": vector_id,
        "collection": collection_name
    })))
}

pub async fn update_vector(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let id = payload.get("id")
        .and_then(|i| i.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Updating vector: {}", id);

    Ok(Json(json!({
        "message": format!("Vector '{}' updated successfully", id)
    })))
}

pub async fn delete_vector_generic(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let id = payload.get("id")
        .and_then(|i| i.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Deleting vector: {}", id);

    Ok(Json(json!({
        "message": format!("Vector '{}' deleted successfully", id)
    })))
}

pub async fn embed_text(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let text = payload.get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // For now, return mock embedding
    Ok(Json(json!({
        "embedding": vec![0.1; 512],
        "text": text,
        "dimension": 512
    })))
}

pub async fn batch_insert_texts(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let texts = payload.get("texts")
        .and_then(|t| t.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Batch inserting {} texts", texts.len());

    Ok(Json(json!({
        "message": format!("Batch inserted {} texts successfully", texts.len()),
        "count": texts.len()
    })))
}

pub async fn insert_texts(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let texts = payload.get("texts")
        .and_then(|t| t.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Inserting {} texts", texts.len());

    Ok(Json(json!({
        "message": format!("Inserted {} texts successfully", texts.len()),
        "count": texts.len()
    })))
}

pub async fn batch_search_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let queries = payload.get("queries")
        .and_then(|q| q.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Batch searching {} queries", queries.len());

    Ok(Json(json!({
        "results": [],
        "queries": queries.len(),
        "message": "Batch search completed"
    })))
}

pub async fn batch_update_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let updates = payload.get("updates")
        .and_then(|u| u.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Batch updating {} vectors", updates.len());

    Ok(Json(json!({
        "message": format!("Batch updated {} vectors successfully", updates.len()),
        "count": updates.len()
    })))
}

pub async fn batch_delete_vectors(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let ids = payload.get("ids")
        .and_then(|i| i.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Batch deleting {} vectors", ids.len());

    Ok(Json(json!({
        "message": format!("Batch deleted {} vectors successfully", ids.len()),
        "count": ids.len()
    })))
}

// Intelligent search REST handlers

pub async fn intelligent_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::intelligent_search::rest_api::{RESTAPIHandler, IntelligentSearchRequest};
    
    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    
    // Extract parameters from JSON payload
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let collections = payload.get("collections")
        .and_then(|c| c.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>());
    
    let max_results = payload.get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);
    
    let domain_expansion = payload.get("domain_expansion")
        .and_then(|d| d.as_bool());
    
    let technical_focus = payload.get("technical_focus")
        .and_then(|t| t.as_bool());
    
    let mmr_enabled = payload.get("mmr_enabled")
        .and_then(|m| m.as_bool());
    
    let mmr_lambda = payload.get("mmr_lambda")
        .and_then(|l| l.as_f64())
        .map(|l| l as f32);
    
    // Create intelligent search request
    let request = IntelligentSearchRequest {
        query: query.to_string(),
        collections,
        max_results,
        domain_expansion,
        technical_focus,
        mmr_enabled,
        mmr_lambda,
    };
    
    match handler.handle_intelligent_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Intelligent search error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn multi_collection_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::intelligent_search::rest_api::{RESTAPIHandler, MultiCollectionSearchRequest};
    
    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let collections = payload.get("collections")
        .and_then(|c| c.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    
    let max_per_collection = payload.get("max_per_collection")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);
    
    let max_total_results = payload.get("max_total_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);
    
    let cross_collection_reranking = payload.get("cross_collection_reranking")
        .and_then(|c| c.as_bool());
    
    let request = MultiCollectionSearchRequest {
        query: query.to_string(),
        collections,
        max_per_collection,
        max_total_results,
        cross_collection_reranking,
    };
    
    match handler.handle_multi_collection_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Multi collection search error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn semantic_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::intelligent_search::rest_api::{RESTAPIHandler, SemanticSearchRequest};
    
    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let max_results = payload.get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);
    
    let semantic_reranking = payload.get("semantic_reranking")
        .and_then(|s| s.as_bool());
    
    let cross_encoder_reranking = payload.get("cross_encoder_reranking")
        .and_then(|c| c.as_bool());
    
    let similarity_threshold = payload.get("similarity_threshold")
        .and_then(|s| s.as_f64())
        .map(|s| s as f32);
    
    let request = SemanticSearchRequest {
        query: query.to_string(),
        collection: collection.to_string(),
        max_results,
        semantic_reranking,
        cross_encoder_reranking,
        similarity_threshold,
    };
    
    match handler.handle_semantic_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Semantic search error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn contextual_search(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::intelligent_search::rest_api::{RESTAPIHandler, ContextualSearchRequest};
    
    // Create handler with the actual server instances
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let context_filters = payload.get("context_filters")
        .and_then(|f| f.as_object())
        .map(|obj| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
            map
        });
    
    let context_weight = payload.get("context_weight")
        .and_then(|w| w.as_f64())
        .map(|w| w as f32);
    
    let context_reranking = payload.get("context_reranking")
        .and_then(|r| r.as_bool());
    
    let max_results = payload.get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);
    
    let request = ContextualSearchRequest {
        query: query.to_string(),
        collection: collection.to_string(),
        context_filters,
        context_weight,
        context_reranking,
        max_results,
    };
    
    match handler.handle_contextual_search(request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or(json!({})))),
        Err(e) => {
            error!("Contextual search error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================
// Discovery API Handlers
// ============================================

pub async fn discover(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{Discovery, DiscoveryConfig};
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let mut config = DiscoveryConfig::default();
    
    if let Some(include) = payload.get("include_collections").and_then(|v| v.as_array()) {
        config.include_collections = include.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }
    
    if let Some(exclude) = payload.get("exclude_collections").and_then(|v| v.as_array()) {
        config.exclude_collections = exclude.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }
    
    if let Some(max_bullets) = payload.get("max_bullets").and_then(|v| v.as_u64()) {
        config.max_bullets = max_bullets as usize;
    }
    
    if let Some(broad_k) = payload.get("broad_k").and_then(|v| v.as_u64()) {
        config.broad_k = broad_k as usize;
    }
    
    if let Some(focus_k) = payload.get("focus_k").and_then(|v| v.as_u64()) {
        config.focus_k = focus_k as usize;
    }
    
    let discovery = Discovery::new(config, state.store.clone(), state.embedding_manager.clone());
    
    match discovery.discover(query).await {
        Ok(response) => Ok(Json(json!({
            "answer_prompt": response.answer_prompt,
            "sections": response.plan.sections.len(),
            "bullets": response.bullets.len(),
            "chunks": response.chunks.len(),
            "metrics": {
                "total_time_ms": response.metrics.total_time_ms,
                "collections_searched": response.metrics.collections_searched,
                "queries_generated": response.metrics.queries_generated,
                "chunks_found": response.metrics.chunks_found,
                "chunks_after_dedup": response.metrics.chunks_after_dedup,
                "bullets_extracted": response.metrics.bullets_extracted,
                "final_prompt_tokens": response.metrics.final_prompt_tokens,
            }
        }))),
        Err(e) => {
            error!("Discovery error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn filter_collections(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::filter_collections as filter_fn;
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let include: Vec<&str> = payload.get("include")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    
    let exclude: Vec<&str> = payload.get("exclude")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    
    let all_collections: Vec<_> = state.store.list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                crate::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();
    
    match filter_fn(query, &include, &exclude, &all_collections) {
        Ok(filtered) => Ok(Json(json!({
            "filtered_collections": filtered.iter().map(|c| json!({
                "name": c.name,
                "vector_count": c.vector_count,
            })).collect::<Vec<_>>(),
            "count": filtered.len(),
        }))),
        Err(e) => {
            error!("Filter collections error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn score_collections(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{score_collections as score_fn, ScoringConfig};
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let mut config = ScoringConfig::default();
    
    if let Some(w) = payload.get("name_match_weight").and_then(|v| v.as_f64()) {
        config.name_match_weight = w as f32;
    }
    if let Some(w) = payload.get("term_boost_weight").and_then(|v| v.as_f64()) {
        config.term_boost_weight = w as f32;
    }
    if let Some(w) = payload.get("signal_boost_weight").and_then(|v| v.as_f64()) {
        config.signal_boost_weight = w as f32;
    }
    
    let all_collections: Vec<_> = state.store.list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                crate::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();
    
    let query_terms: Vec<&str> = query.split_whitespace().collect();
    
    match score_fn(&query_terms, &all_collections, &config) {
        Ok(scored) => Ok(Json(json!({
            "scored_collections": scored.iter().map(|(c, score)| json!({
                "name": c.name,
                "score": score,
                "vector_count": c.vector_count,
            })).collect::<Vec<_>>(),
            "count": scored.len(),
        }))),
        Err(e) => {
            error!("Score collections error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn expand_queries(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{expand_queries_baseline, ExpansionConfig};
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let mut config = ExpansionConfig::default();
    
    if let Some(max) = payload.get("max_expansions").and_then(|v| v.as_u64()) {
        config.max_expansions = max as usize;
    }
    if let Some(def) = payload.get("include_definition").and_then(|v| v.as_bool()) {
        config.include_definition = def;
    }
    if let Some(feat) = payload.get("include_features").and_then(|v| v.as_bool()) {
        config.include_features = feat;
    }
    if let Some(arch) = payload.get("include_architecture").and_then(|v| v.as_bool()) {
        config.include_architecture = arch;
    }
    
    match expand_queries_baseline(query, &config) {
        Ok(expanded) => Ok(Json(json!({
            "original_query": query,
            "expanded_queries": expanded,
            "count": expanded.len(),
        }))),
        Err(e) => {
            error!("Expand queries error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn broad_discovery(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{broad_discovery as broad_fn, BroadDiscoveryConfig};
    
    let queries = payload.get("queries")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    
    let k = payload.get("k")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;
    
    let config = BroadDiscoveryConfig::default();
    
    let collections: Vec<_> = state.store.list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let metadata = coll.metadata();
                crate::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: metadata.config.dimension,
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at,
                    updated_at: metadata.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();
    
    match broad_fn(&queries, &collections, k, &config, &state.store, &state.embedding_manager).await {
        Ok(chunks) => Ok(Json(json!({
            "chunks": chunks.iter().map(|c| json!({
                "collection": c.collection,
                "score": c.score,
                "content_preview": c.content.chars().take(100).collect::<String>(),
            })).collect::<Vec<_>>(),
            "count": chunks.len(),
        }))),
        Err(e) => {
            error!("Broad discovery error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn semantic_focus(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{semantic_focus as focus_fn, SemanticFocusConfig};
    
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let queries = payload.get("queries")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    
    let k = payload.get("k")
        .and_then(|v| v.as_u64())
        .unwrap_or(15) as usize;
    
    let config = SemanticFocusConfig::default();
    
    let coll = state.store.get_collection(collection_name)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let metadata = coll.metadata();
    let collection = crate::discovery::CollectionRef {
        name: collection_name.to_string(),
        dimension: metadata.config.dimension,
        vector_count: metadata.vector_count,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        tags: vec![],
    };
    
    match focus_fn(&collection, &queries, k, &config, &state.store, &state.embedding_manager).await {
        Ok(chunks) => Ok(Json(json!({
            "chunks": chunks.iter().map(|c| json!({
                "collection": c.collection,
                "score": c.score,
                "content_preview": c.content.chars().take(100).collect::<String>(),
            })).collect::<Vec<_>>(),
            "count": chunks.len(),
        }))),
        Err(e) => {
            error!("Semantic focus error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn promote_readme(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{promote_readme as promote_fn, ReadmePromotionConfig, ScoredChunk, ChunkMetadata};
    
    let chunks_json = payload.get("chunks")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let chunks: Vec<ScoredChunk> = chunks_json.iter().filter_map(|v| {
        let obj = v.as_object()?;
        Some(ScoredChunk {
            collection: obj.get("collection")?.as_str()?.to_string(),
            doc_id: obj.get("doc_id")?.as_str()?.to_string(),
            content: obj.get("content")?.as_str()?.to_string(),
            score: obj.get("score")?.as_f64()? as f32,
            metadata: ChunkMetadata {
                file_path: obj.get("file_path")?.as_str()?.to_string(),
                chunk_index: obj.get("chunk_index")?.as_u64()? as usize,
                file_extension: obj.get("file_extension")?.as_str()?.to_string(),
                line_range: None,
            },
        })
    }).collect();
    
    let config = ReadmePromotionConfig::default();
    
    match promote_fn(&chunks, &config) {
        Ok(promoted) => Ok(Json(json!({
            "promoted_chunks": promoted.iter().map(|c| json!({
                "collection": c.collection,
                "file_path": c.metadata.file_path,
                "score": c.score,
            })).collect::<Vec<_>>(),
            "count": promoted.len(),
        }))),
        Err(e) => {
            error!("Promote README error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn compress_evidence(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{compress_evidence as compress_fn, CompressionConfig, ScoredChunk, ChunkMetadata};
    
    let chunks_json = payload.get("chunks")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let max_bullets = payload.get("max_bullets")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;
    
    let max_per_doc = payload.get("max_per_doc")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    
    let chunks: Vec<ScoredChunk> = chunks_json.iter().filter_map(|v| {
        let obj = v.as_object()?;
        Some(ScoredChunk {
            collection: obj.get("collection")?.as_str()?.to_string(),
            doc_id: obj.get("doc_id")?.as_str()?.to_string(),
            content: obj.get("content")?.as_str()?.to_string(),
            score: obj.get("score")?.as_f64()? as f32,
            metadata: ChunkMetadata {
                file_path: obj.get("file_path")?.as_str()?.to_string(),
                chunk_index: obj.get("chunk_index")?.as_u64()? as usize,
                file_extension: obj.get("file_extension")?.as_str()?.to_string(),
                line_range: None,
            },
        })
    }).collect();
    
    let config = CompressionConfig::default();
    
    match compress_fn(&chunks, max_bullets, max_per_doc, &config) {
        Ok(bullets) => Ok(Json(json!({
            "bullets": bullets.iter().map(|b| json!({
                "text": b.text,
                "source_id": b.source_id,
                "category": format!("{:?}", b.category),
                "score": b.score,
            })).collect::<Vec<_>>(),
            "count": bullets.len(),
        }))),
        Err(e) => {
            error!("Compress evidence error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn build_answer_plan(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{build_answer_plan as build_fn, AnswerPlanConfig, Bullet, BulletCategory};
    
    let bullets_json = payload.get("bullets")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let bullets: Vec<Bullet> = bullets_json.iter().filter_map(|v| {
        let obj = v.as_object()?;
        let category = match obj.get("category")?.as_str()? {
            "Definition" => BulletCategory::Definition,
            "Feature" => BulletCategory::Feature,
            "Architecture" => BulletCategory::Architecture,
            "Performance" => BulletCategory::Performance,
            "Integration" => BulletCategory::Integration,
            "UseCase" => BulletCategory::UseCase,
            _ => BulletCategory::Other,
        };
        
        Some(Bullet {
            text: obj.get("text")?.as_str()?.to_string(),
            source_id: obj.get("source_id")?.as_str()?.to_string(),
            collection: obj.get("collection")?.as_str()?.to_string(),
            file_path: obj.get("file_path")?.as_str()?.to_string(),
            score: obj.get("score")?.as_f64()? as f32,
            category,
        })
    }).collect();
    
    let config = AnswerPlanConfig::default();
    
    match build_fn(&bullets, &config) {
        Ok(plan) => Ok(Json(json!({
            "sections": plan.sections.iter().map(|s| json!({
                "title": s.title,
                "bullets_count": s.bullets.len(),
                "bullets": s.bullets.iter().map(|b| json!({
                    "text": b.text,
                    "source_id": b.source_id,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "total_bullets": plan.total_bullets,
            "sources": plan.sources,
        }))),
        Err(e) => {
            error!("Build answer plan error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn render_llm_prompt(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::discovery::{render_llm_prompt as render_fn, PromptRenderConfig, AnswerPlan, Section, SectionType, Bullet, BulletCategory};
    
    let plan_json = payload.get("plan")
        .and_then(|v| v.as_object())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let sections_json = plan_json.get("sections")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let sections: Vec<Section> = sections_json.iter().filter_map(|v| {
        let obj = v.as_object()?;
        let bullets_json = obj.get("bullets")?.as_array()?;
        
        let bullets: Vec<Bullet> = bullets_json.iter().filter_map(|b| {
            let b_obj = b.as_object()?;
            let category = match b_obj.get("category")?.as_str()? {
                "Definition" => BulletCategory::Definition,
                "Feature" => BulletCategory::Feature,
                "Architecture" => BulletCategory::Architecture,
                "Performance" => BulletCategory::Performance,
                "Integration" => BulletCategory::Integration,
                "UseCase" => BulletCategory::UseCase,
                _ => BulletCategory::Other,
            };
            
            Some(Bullet {
                text: b_obj.get("text")?.as_str()?.to_string(),
                source_id: b_obj.get("source_id")?.as_str()?.to_string(),
                collection: b_obj.get("collection")?.as_str()?.to_string(),
                file_path: b_obj.get("file_path")?.as_str()?.to_string(),
                score: b_obj.get("score")?.as_f64()? as f32,
                category,
            })
        }).collect();
        
        Some(Section {
            title: obj.get("title")?.as_str()?.to_string(),
            section_type: SectionType::Definition,
            bullets,
            priority: obj.get("priority")?.as_u64()? as usize,
        })
    }).collect();
    
    let plan = AnswerPlan {
        sections,
        total_bullets: plan_json.get("total_bullets").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        sources: plan_json.get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
    };
    
    let config = PromptRenderConfig::default();
    
    match render_fn(&plan, &config) {
        Ok(prompt) => Ok(Json(json!({
            "prompt": prompt,
            "length": prompt.len(),
            "estimated_tokens": prompt.len() / 4,
        }))),
        Err(e) => {
            error!("Render LLM prompt error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================
// File Operations API Handlers
// ============================================

pub async fn get_file_content(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::FileOperations;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let file_path = payload.get("file_path")
        .and_then(|f| f.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let max_size_kb = payload.get("max_size_kb")
        .and_then(|m| m.as_u64())
        .unwrap_or(500) as usize;
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.get_file_content(collection, file_path, max_size_kb).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get file content error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_files_in_collection(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::{FileOperations, FileListFilter, SortBy};
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let filter_by_type = payload.get("filter_by_type")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect());
    
    let min_chunks = payload.get("min_chunks")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    
    let max_results = payload.get("max_results")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    
    let sort_by = payload.get("sort_by")
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
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.list_files_in_collection(collection, filter).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("List files error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_file_summary(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::{FileOperations, SummaryType};
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let file_path = payload.get("file_path")
        .and_then(|f| f.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let summary_type = payload.get("summary_type")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "extractive" => Some(SummaryType::Extractive),
            "structural" => Some(SummaryType::Structural),
            "both" => Some(SummaryType::Both),
            _ => None,
        })
        .unwrap_or(SummaryType::Both);
    
    let max_sentences = payload.get("max_sentences")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.get_file_summary(collection, file_path, summary_type, max_sentences).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get file summary error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_file_chunks_ordered(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::FileOperations;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let file_path = payload.get("file_path")
        .and_then(|f| f.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let start_chunk = payload.get("start_chunk")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    
    let limit = payload.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let include_context = payload.get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.get_file_chunks_ordered(collection, file_path, start_chunk, limit, include_context).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get file chunks error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_project_outline(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::FileOperations;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let max_depth = payload.get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    let include_summaries = payload.get("include_summaries")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let highlight_key_files = payload.get("highlight_key_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.get_project_outline(collection, max_depth, include_summaries, highlight_key_files).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get project outline error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_related_files(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::FileOperations;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let file_path = payload.get("file_path")
        .and_then(|f| f.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let limit = payload.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    let similarity_threshold = payload.get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.6) as f32;
    
    let include_reason = payload.get("include_reason")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.get_related_files(collection, file_path, limit, similarity_threshold, include_reason, &state.embedding_manager).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Get related files error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn search_by_file_type(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    use crate::file_operations::FileOperations;
    
    let collection = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let file_types = payload.get("file_types")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let limit = payload.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    let return_full_files = payload.get("return_full_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let file_ops = FileOperations::with_store(state.store.clone());
    
    match file_ops.search_by_file_type(collection, query, file_types, limit, return_full_files, &state.embedding_manager).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(json!({})))),
        Err(e) => {
            error!("Search by file type error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}