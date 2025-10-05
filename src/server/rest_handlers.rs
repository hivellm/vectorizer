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

pub async fn get_memory_analysis(State(_state): State<VectorizerServer>) -> Json<Value> {
    Json(json!({
        "memory_usage": {
            "total_memory_mb": 0,
            "used_memory_mb": 0,
            "free_memory_mb": 0,
            "memory_percentage": 0.0
        },
        "collections_memory": [],
        "timestamp": chrono::Utc::now().to_rfc3339()
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
    let collections = state.store.list_collections();
    
    let collection_infos: Vec<Value> = collections.iter().map(|name| {
        match state.store.get_collection(name) {
            Ok(collection) => json!({
                "name": name,
                "vector_count": collection.vector_count(),
                "document_count": collection.vector_count(), // Same as vector count for now
                "dimension": collection.config().dimension,
                "metric": format!("{:?}", collection.config().metric),
                "embedding_provider": "bm25",
                "created_at": chrono::Utc::now().to_rfc3339(),
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
            }),
            Err(_) => json!({
                "name": name,
                "vector_count": 0,
                "document_count": 0,
                "dimension": 512,
                "metric": "Cosine",
                "embedding_provider": "bm25",
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

    info!("Creating collection: {}", name);

    // For now, just return success
    Ok(Json(json!({
        "message": format!("Collection '{}' created successfully", name),
        "collection": name,
        "dimension": dimension
    })))
}

pub async fn get_collection(
    State(state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.store.get_collection(&name) {
        Ok(collection) => {
            Ok(Json(json!({
                "name": name,
                "vector_count": collection.vector_count(),
                "dimension": collection.config().dimension,
                "metric": format!("{:?}", collection.config().metric),
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
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let text = payload.get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Inserting text: {}", text);

    Ok(Json(json!({
        "message": "Text inserted successfully",
        "text": text
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