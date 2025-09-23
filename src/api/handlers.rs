//! HTTP request handlers for the Vectorizer API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};

use crate::{
    models::{CollectionConfig, Payload, Vector},
    VectorStore,
};

use super::types::*;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Vector store instance
    pub store: Arc<VectorStore>,
    /// Server start time for uptime calculation
    pub start_time: Instant,
}

impl AppState {
    /// Create new application state
    pub fn new(store: VectorStore) -> Self {
        Self {
            store: Arc::new(store),
            start_time: Instant::now(),
        }
    }
}

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    debug!("Health check requested");
    
    let collections = state.store.list_collections();
    let total_vectors = collections
        .iter()
        .map(|name| {
            state
                .store
                .get_collection_metadata(name)
                .map(|meta| meta.vector_count)
                .unwrap_or(0)
        })
        .sum();

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: crate::VERSION.to_string(),
        uptime: state.start_time.elapsed().as_secs(),
        collections: collections.len(),
        total_vectors,
    })
}

/// List all collections
pub async fn list_collections(State(state): State<AppState>) -> Json<Vec<CollectionInfo>> {
    debug!("Listing collections");
    
    let collections = state.store.list_collections();
    let mut collection_infos = Vec::new();

    for name in collections {
        if let Ok(metadata) = state.store.get_collection_metadata(&name) {
            collection_infos.push(CollectionInfo {
                name: metadata.name,
                dimension: metadata.config.dimension,
                metric: metadata.config.metric.into(),
                vector_count: metadata.vector_count,
                created_at: metadata.created_at.to_rfc3339(),
                updated_at: metadata.updated_at.to_rfc3339(),
            });
        }
    }

    Json(collection_infos)
}

/// Create a new collection
pub async fn create_collection(
    State(state): State<AppState>,
    Json(request): Json<CreateCollectionRequest>,
) -> Result<Json<CreateCollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating collection: {}", request.name);

    // Validate collection name
    if request.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Collection name cannot be empty".to_string(),
                code: "INVALID_COLLECTION_NAME".to_string(),
                details: None,
            }),
        ));
    }

    // Validate dimension
    if request.dimension == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Dimension must be greater than 0".to_string(),
                code: "INVALID_DIMENSION".to_string(),
                details: None,
            }),
        ));
    }

    // Create collection configuration
    let config = CollectionConfig {
        dimension: request.dimension,
        metric: request.metric.into(),
        hnsw_config: request.hnsw_config.map(Into::into).unwrap_or_default(),
        quantization: None,
        compression: Default::default(),
    };

    // Create the collection
    match state.store.create_collection(&request.name, config) {
        Ok(_) => {
            info!("Collection '{}' created successfully", request.name);
            Ok(Json(CreateCollectionResponse {
                message: "Collection created successfully".to_string(),
                collection: request.name,
            }))
        }
        Err(e) => {
            error!("Failed to create collection '{}': {}", request.name, e);
            Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: format!("Failed to create collection: {}", e),
                    code: "COLLECTION_CREATION_FAILED".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

/// Get collection information
pub async fn get_collection(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
) -> Result<Json<CollectionInfo>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting collection info: {}", collection_name);

    match state.store.get_collection_metadata(&collection_name) {
        Ok(metadata) => Ok(Json(CollectionInfo {
            name: metadata.name,
            dimension: metadata.config.dimension,
            metric: metadata.config.metric.into(),
            vector_count: metadata.vector_count,
            created_at: metadata.created_at.to_rfc3339(),
            updated_at: metadata.updated_at.to_rfc3339(),
        })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Collection '{}' not found", collection_name),
                code: "COLLECTION_NOT_FOUND".to_string(),
                details: None,
            }),
        )),
    }
}

/// Delete a collection
pub async fn delete_collection(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting collection: {}", collection_name);

    match state.store.delete_collection(&collection_name) {
        Ok(_) => {
            info!("Collection '{}' deleted successfully", collection_name);
            Ok(Json(serde_json::json!({
                "message": "Collection deleted successfully",
                "collection": collection_name
            })))
        }
        Err(e) => {
            error!("Failed to delete collection '{}': {}", collection_name, e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Failed to delete collection: {}", e),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

/// Insert vectors into a collection
pub async fn insert_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<InsertVectorsRequest>,
) -> Result<Json<InsertVectorsResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Inserting {} vectors into collection: {}",
        request.vectors.len(),
        collection_name
    );

    if request.vectors.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No vectors provided".to_string(),
                code: "NO_VECTORS".to_string(),
                details: None,
            }),
        ));
    }

    // Convert API vectors to internal format
    let mut vectors = Vec::new();
    for vector_data in request.vectors {
        let payload = vector_data.payload.map(|p| Payload::new(p));
        vectors.push(Vector {
            id: vector_data.id,
            data: vector_data.vector,
            payload,
        });
    }

    let vector_count = vectors.len();

    match state.store.insert(&collection_name, vectors) {
        Ok(_) => {
            info!(
                "Successfully inserted {} vectors into collection '{}'",
                vector_count, collection_name
            );
            Ok(Json(InsertVectorsResponse {
                message: "Vectors inserted successfully".to_string(),
                inserted: vector_count,
            }))
        }
        Err(e) => {
            error!(
                "Failed to insert vectors into collection '{}': {}",
                collection_name, e
            );
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Failed to insert vectors: {}", e),
                    code: "VECTOR_INSERTION_FAILED".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

/// Search for similar vectors
pub async fn search_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        "Searching in collection '{}' with limit: {:?}",
        collection_name, request.limit
    );

    let start_time = Instant::now();
    let limit = request.limit.unwrap_or(10).min(100); // Cap at 100 results

    match state.store.search(&collection_name, &request.vector, limit) {
        Ok(results) => {
            let query_time = start_time.elapsed().as_secs_f64() * 1000.0;
            
            let mut search_results = Vec::new();
            for result in results {
                // Apply score threshold if specified
                if let Some(threshold) = request.score_threshold {
                    if result.score < threshold {
                        continue;
                    }
                }

                search_results.push(super::types::SearchResult {
                    id: result.id,
                    score: result.score,
                    vector: result.vector.unwrap_or_default(),
                    payload: result.payload.map(|p| p.data),
                });
            }

            debug!(
                "Search completed in {:.2}ms, found {} results",
                query_time,
                search_results.len()
            );

            Ok(Json(SearchResponse {
                results: search_results,
                query_time_ms: query_time,
            }))
        }
        Err(e) => {
            error!("Search failed in collection '{}': {}", collection_name, e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Search failed: {}", e),
                    code: "SEARCH_FAILED".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

/// Get a specific vector by ID
pub async fn get_vector(
    State(state): State<AppState>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<VectorData>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Getting vector '{}' from collection '{}'", vector_id, collection_name);

    match state.store.get_vector(&collection_name, &vector_id) {
        Ok(vector) => Ok(Json(VectorData {
            id: vector.id,
            vector: vector.data,
            payload: vector.payload.map(|p| p.data),
        })),
        Err(e) => {
            error!(
                "Failed to get vector '{}' from collection '{}': {}",
                vector_id, collection_name, e
            );
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Vector not found: {}", e),
                    code: "VECTOR_NOT_FOUND".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

/// Delete a vector by ID
pub async fn delete_vector(
    State(state): State<AppState>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting vector '{}' from collection '{}'", vector_id, collection_name);

    match state.store.delete(&collection_name, &vector_id) {
        Ok(_) => {
            info!(
                "Vector '{}' deleted successfully from collection '{}'",
                vector_id, collection_name
            );
            Ok(Json(serde_json::json!({
                "message": "Vector deleted successfully",
                "collection": collection_name,
                "vector_id": vector_id
            })))
        }
        Err(e) => {
            error!(
                "Failed to delete vector '{}' from collection '{}': {}",
                vector_id, collection_name, e
            );
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Failed to delete vector: {}", e),
                    code: "VECTOR_NOT_FOUND".to_string(),
                    details: None,
                }),
            ))
        }
    }
}
