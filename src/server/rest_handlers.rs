//! REST API handlers

use axum::{
    extract::{Path, State},
    response::Json,
    http::StatusCode,
};
use serde_json::{json, Value};

use super::VectorizerServer;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn list_collections(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    Json(json!({
        "collections": collections,
        "total": collections.len()
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
        .ok_or(StatusCode::BAD_REQUEST)? as usize;

    let metric = payload.get("metric")
        .and_then(|m| m.as_str())
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

    match state.store.create_collection(name, config) {
        Ok(_) => Ok(Json(json!({
            "status": "created",
            "name": name,
            "dimension": dimension,
            "metric": metric
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_collection(
    State(state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.store.get_collection(&name) {
        Ok(collection) => Ok(Json(json!({
            "name": name,
            "vector_count": collection.vector_count(),
            "dimension": collection.config().dimension,
            "metric": collection.config().metric
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_collection(
    State(state): State<VectorizerServer>,
    Path(name): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match state.store.delete_collection(&name) {
        Ok(_) => Ok(Json(json!({
            "status": "deleted",
            "name": name
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn search_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection_name")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let limit = payload.get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(10) as usize;

    // Generate embedding
    let embedding = match state.embedding_manager.embed(query) {
        Ok(emb) => emb,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Search
    match state.store.search(collection_name, &embedding, limit) {
        Ok(results) => {
            let search_results: Vec<Value> = results
                .iter()
                .map(|result| json!({
                    "id": result.id,
                    "score": result.score,
                    "payload": result.payload
                }))
                .collect();

            Ok(Json(json!({
                "results": search_results,
                "query": query,
                "collection": collection_name,
                "total_results": search_results.len()
            })))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn insert_text(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection_name")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let text = payload.get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Generate embedding
    let embedding = match state.embedding_manager.embed(text) {
        Ok(emb) => emb,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let vector_id = uuid::Uuid::new_v4().to_string();
    let metadata = payload.get("metadata").cloned().unwrap_or(json!({}));
    
    match state.store.insert(collection_name, vec![crate::models::Vector::with_payload(
        vector_id.clone(),
        embedding,
        crate::models::Payload::from_value(metadata).unwrap()
    )]) {
        Ok(_) => Ok(Json(json!({
            "status": "inserted",
            "vector_id": vector_id,
            "collection": collection_name
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn batch_insert_texts(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection_name")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let texts = payload.get("texts")
        .and_then(|t| t.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut vectors = Vec::new();
    
    for text_value in texts {
        if let Some(text) = text_value.as_str() {
            let embedding = match state.embedding_manager.embed(text) {
                Ok(emb) => emb,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            let vector_id = uuid::Uuid::new_v4().to_string();
            vectors.push(crate::models::Vector::new(vector_id, embedding));
        }
    }

    match state.store.insert(collection_name, vectors.clone()) {
        Ok(_) => Ok(Json(json!({
            "status": "batch_inserted",
            "collection": collection_name,
            "count": vectors.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_vector(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection_name")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let vector_id = payload.get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let text = payload.get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Generate embedding
    let embedding = match state.embedding_manager.embed(text) {
        Ok(emb) => emb,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let metadata = payload.get("metadata").cloned().unwrap_or(json!({}));
    
    match state.store.update(collection_name, crate::models::Vector::with_payload(
        vector_id.to_string(),
        embedding,
        crate::models::Payload::from_value(metadata).unwrap()
    )) {
        Ok(_) => Ok(Json(json!({
            "status": "updated",
            "vector_id": vector_id,
            "collection": collection_name
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_vector(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection_name")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let vector_id = payload.get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    match state.store.delete(collection_name, vector_id) {
        Ok(_) => Ok(Json(json!({
            "status": "deleted",
            "vector_id": vector_id,
            "collection": collection_name
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_stats(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    let total_vectors: usize = collections.iter()
        .filter_map(|name| state.store.get_collection(name).ok())
        .map(|col| col.vector_count())
        .sum();

    Json(json!({
        "total_collections": collections.len(),
        "total_vectors": total_vectors,
        "collections": collections,
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn insert_texts(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let texts = payload.get("texts")
        .and_then(|t| t.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut vectors = Vec::new();
    
    for text_obj in texts {
        if let Some(obj) = text_obj.as_object() {
            let id = obj.get("id").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let text = obj.get("text").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            
            let embedding = match state.embedding_manager.embed(text) {
                Ok(emb) => emb,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            let metadata = obj.get("metadata").cloned().unwrap_or(json!({}));
            vectors.push(crate::models::Vector::with_payload(
                id.to_string(),
                embedding,
                crate::models::Payload::from_value(metadata).unwrap()
            ));
        }
    }

    match state.store.insert(collection_name, vectors.clone()) {
        Ok(_) => Ok(Json(json!({
            "status": "inserted",
            "collection": collection_name,
            "count": vectors.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn batch_search_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let queries = payload.get("queries")
        .and_then(|q| q.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut all_results = Vec::new();
    
    for query_obj in queries {
        if let Some(obj) = query_obj.as_object() {
            let query = obj.get("query").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let limit = obj.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            
            let embedding = match state.embedding_manager.embed(query) {
                Ok(emb) => emb,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };
            
            match state.store.search(collection_name, &embedding, limit) {
                Ok(results) => {
                    all_results.push(json!({
                        "query": query,
                        "results": results.iter().map(|r| json!({
                            "id": r.id,
                            "score": r.score,
                            "payload": r.payload
                        })).collect::<Vec<_>>()
                    }));
                },
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }

    Ok(Json(json!({
        "collection": collection_name,
        "searches": all_results,
        "total_searches": all_results.len()
    })))
}

pub async fn batch_update_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let updates = payload.get("updates")
        .and_then(|u| u.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut updated_count = 0;
    
    for update_obj in updates {
        if let Some(obj) = update_obj.as_object() {
            let vector_id = obj.get("vector_id").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                let embedding = match state.embedding_manager.embed(text) {
                    Ok(emb) => emb,
                    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                };

                let metadata = obj.get("metadata").cloned().unwrap_or(json!({}));
                
                if state.store.update(collection_name, crate::models::Vector::with_payload(
                    vector_id.to_string(),
                    embedding,
                    crate::models::Payload::from_value(metadata).unwrap()
                )).is_ok() {
                    updated_count += 1;
                }
            }
        }
    }

    Ok(Json(json!({
        "status": "updated",
        "collection": collection_name,
        "count": updated_count
    })))
}

pub async fn batch_delete_vectors(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let vector_ids = payload.get("vector_ids")
        .and_then(|v| v.as_array())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut deleted_count = 0;
    for id_value in vector_ids {
        if let Some(id) = id_value.as_str() {
            if state.store.delete(collection_name, id).is_ok() {
                deleted_count += 1;
            }
        }
    }

    Ok(Json(json!({
        "status": "deleted",
        "collection": collection_name,
        "count": deleted_count
    })))
}

pub async fn get_indexing_progress() -> Json<Value> {
    Json(json!({
        "status": "no_indexing_in_progress",
        "message": "No active indexing operations",
        "collections": []
    }))
}

pub async fn embed_text(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let text = payload.get("text")
        .and_then(|t| t.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    match state.embedding_manager.embed(text) {
        Ok(embedding) => Ok(Json(json!({
            "embedding": embedding,
            "dimension": embedding.len(),
            "provider": "bm25"
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_vector(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let collection_name = payload.get("collection")
        .and_then(|c| c.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let vector_id = payload.get("vector_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    match state.store.get_collection(collection_name) {
        Ok(coll) => {
            match coll.get_vector(vector_id) {
                Ok(vector) => Ok(Json(json!({
                    "id": vector.id,
                    "data": vector.data,
                    "payload": vector.payload,
                    "collection": collection_name
                }))),
                Err(_) => Err(StatusCode::NOT_FOUND),
            }
        },
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
