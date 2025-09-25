//! HTTP request handlers for the Vectorizer API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, Sse},
};
use futures_util::Stream;
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::{
    VectorStore,
    embedding::{EmbeddingManager, Bm25Embedding},
    models::{CollectionConfig, Payload, Vector},
};
use std::sync::Mutex;

use super::types::*;
use std::collections::HashMap;

/// Workspace collection definition
#[derive(Clone, Debug)]
pub struct WorkspaceCollection {
    pub name: String,
    pub description: String,
    pub dimension: u64,
    pub metric: String,
    pub model: String,
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Vector store instance
    pub store: Arc<VectorStore>,
    /// Embedding manager for consistent embedding generation
    pub embedding_manager: Arc<Mutex<EmbeddingManager>>,
    /// Server start time for uptime calculation
    pub start_time: Instant,
    /// Indexing progress tracking
    pub indexing_progress: Arc<Mutex<HashMap<String, IndexingStatus>>>,
    /// Workspace collections (all defined collections, even if not yet indexed)
    pub workspace_collections: Vec<WorkspaceCollection>,
}

impl AppState {
    /// Create new application state
    pub fn new(store: Arc<VectorStore>, mut embedding_manager: EmbeddingManager) -> Self {
        // Check if BM25 vocabulary is empty and needs rebuilding
        // Note: Vocabulary is built during document loading, so empty here is expected

        // Initialize indexing progress for existing collections
        let mut indexing_progress = HashMap::new();
        for collection_name in store.list_collections() {
            indexing_progress.insert(collection_name, IndexingStatus {
                status: "completed".to_string(),
                progress: 100.0,
                total_documents: 0, // Will be updated when we have stats
                processed_documents: 0,
                estimated_time_remaining: None,
                last_updated: Utc::now().to_rfc3339(),
            });
        }

        // Load workspace collections
        let workspace_collections = Self::load_workspace_collections();

        // Initialize indexing progress for all workspace collections
        for collection in &workspace_collections {
            if !indexing_progress.contains_key(&collection.name) {
                indexing_progress.insert(collection.name.clone(), IndexingStatus {
                    status: "pending".to_string(),
                    progress: 0.0,
                    total_documents: 0,
                    processed_documents: 0,
                    estimated_time_remaining: None,
                    last_updated: Utc::now().to_rfc3339(),
                });
            }
        }

        Self {
            store,
            embedding_manager: Arc::new(Mutex::new(embedding_manager)),
            start_time: Instant::now(),
            indexing_progress: Arc::new(Mutex::new(indexing_progress)),
            workspace_collections,
        }
    }

    /// Load workspace collections from environment variable
    pub fn load_workspace_collections() -> Vec<WorkspaceCollection> {
        if let Ok(workspace_path) = std::env::var("VECTORIZER_WORKSPACE_INFO") {
            if let Ok(content) = std::fs::read_to_string(&workspace_path) {
                if let Ok(workspace_json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(projects) = workspace_json.get("projects").and_then(|p| p.as_array()) {
                        let mut collections = Vec::new();
                        for project in projects {
                            if let Some(project_collections) = project.get("collections").and_then(|c| c.as_array()) {
                                for collection in project_collections {
                                    if let (Some(name), Some(description)) = (
                                        collection.get("name").and_then(|n| n.as_str()),
                                        collection.get("description").and_then(|d| d.as_str())
                                    ) {
                                        let dimension = collection.get("dimension").and_then(|d| d.as_u64()).unwrap_or(512);
                                        let metric = collection.get("metric").and_then(|m| m.as_str()).unwrap_or("cosine").to_string();
                                        let model = collection.get("embedding").and_then(|e| e.get("model")).and_then(|m| m.as_str()).unwrap_or("bm25").to_string();

                                        collections.push(WorkspaceCollection {
                                            name: name.to_string(),
                                            description: description.to_string(),
                                            dimension,
                                            metric,
                                            model,
                                        });
                                    }
                                }
                            }
                        }
                        return collections;
                    }
                }
            }
        }
        Vec::new()
    }

    /// Create new application state with shared indexing progress
    pub fn new_with_progress(
        store: Arc<VectorStore>,
        embedding_manager: EmbeddingManager,
        indexing_progress: Arc<Mutex<HashMap<String, IndexingStatus>>>
    ) -> Self {
        // Load workspace collections
        let workspace_collections = Self::load_workspace_collections();

        // Initialize indexing progress for all workspace collections
        {
            let mut progress = indexing_progress.lock().unwrap();
            for collection in &workspace_collections {
                if !progress.contains_key(&collection.name) {
                    progress.insert(collection.name.clone(), IndexingStatus {
                        status: "pending".to_string(),
                        progress: 0.0,
                        total_documents: 0,
                        processed_documents: 0,
                        estimated_time_remaining: None,
                        last_updated: Utc::now().to_rfc3339(),
                    });
                }
            }
        }

        Self {
            store,
            embedding_manager: Arc::new(Mutex::new(embedding_manager)),
            start_time: Instant::now(),
            indexing_progress,
            workspace_collections,
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
        timestamp: Utc::now().to_rfc3339(),
        uptime: state.start_time.elapsed().as_secs(),
        collections: collections.len(),
        total_vectors,
    })
}

/// List all collections
pub async fn list_collections(State(state): State<AppState>) -> Json<ListCollectionsResponse> {
    debug!("Listing collections");

    let existing_collections = state.store.list_collections();
    let mut collection_infos = Vec::new();

    let indexing_progress = state.indexing_progress.lock().unwrap();
    info!("📊 API: Indexing progress map has {} entries", indexing_progress.len());

    // First, add all workspace-defined collections
    for workspace_collection in &state.workspace_collections {
        let name = &workspace_collection.name;

        let (metadata, indexing_status) = if existing_collections.contains(name) {
            // Collection exists in vector store
            if let Ok(metadata) = state.store.get_collection_metadata(name) {
                let status = indexing_progress.get(name)
                    .cloned()
                    .unwrap_or_else(|| IndexingStatus {
                        status: "completed".to_string(),
                        progress: 100.0,
                        total_documents: 0,
                        processed_documents: 0,
                        estimated_time_remaining: None,
                        last_updated: Utc::now().to_rfc3339(),
                    });

                (Some(metadata), status)
            } else {
                // Collection exists but can't get metadata - show as error
                (None, IndexingStatus {
                    status: "error".to_string(),
                    progress: 0.0,
                    total_documents: 0,
                    processed_documents: 0,
                    estimated_time_remaining: None,
                    last_updated: Utc::now().to_rfc3339(),
                })
            }
        } else {
            // Collection defined in workspace but not yet indexed
            (None, indexing_progress.get(name)
                .cloned()
                .unwrap_or_else(|| IndexingStatus {
                    status: "pending".to_string(),
                    progress: 0.0,
                    total_documents: 0,
                    processed_documents: 0,
                    estimated_time_remaining: None,
                    last_updated: Utc::now().to_rfc3339(),
                }))
        };

        if let Some(metadata) = metadata {
            // Collection exists
            collection_infos.push(CollectionInfo {
                name: metadata.name,
                dimension: metadata.config.dimension,
                metric: metadata.config.metric.into(),
                vector_count: metadata.vector_count,
                created_at: metadata.created_at.to_rfc3339(),
                updated_at: metadata.updated_at.to_rfc3339(),
                indexing_status,
            });
        } else {
            // Collection defined but not yet created
            collection_infos.push(CollectionInfo {
                name: workspace_collection.name.clone(),
                dimension: workspace_collection.dimension as usize,
                metric: match workspace_collection.metric.as_str() {
                    "cosine" => crate::api::types::DistanceMetric::Cosine,
                    "euclidean" => crate::api::types::DistanceMetric::Euclidean,
                    "dot_product" => crate::api::types::DistanceMetric::DotProduct,
                    _ => crate::api::types::DistanceMetric::Cosine,
                },
                vector_count: 0,
                created_at: Utc::now().to_rfc3339(), // Placeholder
                updated_at: Utc::now().to_rfc3339(), // Placeholder
                indexing_status,
            });
        }
    }

    // Also add any collections that exist in vector store but are not in workspace (legacy)
    for name in existing_collections {
        if !state.workspace_collections.iter().any(|wc| wc.name == name) {
            if let Ok(metadata) = state.store.get_collection_metadata(&name) {
                let indexing_status = indexing_progress.get(&name)
                    .cloned()
                    .unwrap_or_else(|| IndexingStatus {
                        status: "completed".to_string(),
                        progress: 100.0,
                        total_documents: 0,
                        processed_documents: 0,
                        estimated_time_remaining: None,
                        last_updated: Utc::now().to_rfc3339(),
                    });

                collection_infos.push(CollectionInfo {
                    name: metadata.name,
                    dimension: metadata.config.dimension,
                    metric: metadata.config.metric.into(),
                    vector_count: metadata.vector_count,
                    created_at: metadata.created_at.to_rfc3339(),
                    updated_at: metadata.updated_at.to_rfc3339(),
                    indexing_status,
                });
            }
        }
    }

    Json(ListCollectionsResponse {
        collections: collection_infos,
    })
}

/// Get indexing progress
pub async fn get_indexing_progress(State(state): State<AppState>) -> Json<IndexingProgressResponse> {
    debug!("Getting indexing progress");

    let indexing_progress = state.indexing_progress.lock().unwrap();
    let collections: Vec<IndexingStatus> = indexing_progress.values().cloned().collect();

    // Determine overall status
    let overall_status = if collections.is_empty() {
        "idle".to_string()
    } else if collections.iter().all(|c| c.status == "completed") {
        "completed".to_string()
    } else if collections.iter().any(|c| c.status == "indexing") {
        "indexing".to_string()
    } else {
        "partial".to_string()
    };

    // Calculate estimated completion time based on active collections
    let estimated_completion = if overall_status == "indexing" {
        let active_collections: Vec<_> = collections.iter()
            .filter(|c| c.status == "indexing" && c.estimated_time_remaining.is_some())
            .collect();

        if !active_collections.is_empty() {
            let max_remaining = active_collections.iter()
                .map(|c| c.estimated_time_remaining.unwrap())
                .max()
                .unwrap_or(0);

            let completion_time = Utc::now() + chrono::Duration::seconds(max_remaining as i64);
            Some(completion_time.to_rfc3339())
        } else {
            None
        }
    } else {
        None
    };

    Json(IndexingProgressResponse {
        overall_status,
        collections,
        started_at: Utc::now().to_rfc3339(), // This should be tracked from when indexing started
        estimated_completion,
    })
}

/// Create a new collection
pub async fn create_collection(
    State(state): State<AppState>,
    Json(request): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<CreateCollectionResponse>), (StatusCode, Json<ErrorResponse>)> {
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
            Ok((
                StatusCode::CREATED,
                Json(CreateCollectionResponse {
                    message: "Collection created successfully".to_string(),
                    collection: request.name,
                }),
            ))
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
        Ok(metadata) => {
            let indexing_status = state.indexing_progress.lock().unwrap()
                .get(&collection_name)
                .cloned()
                .unwrap_or_else(|| IndexingStatus {
                    status: "completed".to_string(),
                    progress: 100.0,
                    total_documents: metadata.vector_count,
                    processed_documents: metadata.vector_count,
                    estimated_time_remaining: None,
                    last_updated: Utc::now().to_rfc3339(),
                });

            Ok(Json(CollectionInfo {
                name: metadata.name,
                dimension: metadata.config.dimension,
                metric: metadata.config.metric.into(),
                vector_count: metadata.vector_count,
                created_at: metadata.created_at.to_rfc3339(),
                updated_at: metadata.updated_at.to_rfc3339(),
                indexing_status,
            }))
        },
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
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting collection: {}", collection_name);

    match state.store.delete_collection(&collection_name) {
        Ok(_) => {
            info!("Collection '{}' deleted successfully", collection_name);
            Ok(StatusCode::NO_CONTENT)
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
) -> Result<(StatusCode, Json<InsertVectorsResponse>), (StatusCode, Json<ErrorResponse>)> {
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
        // Validate embedding - reject zero vectors
        let non_zero_count = vector_data.vector.iter().filter(|&&x| x != 0.0).count();
        if non_zero_count == 0 {
            warn!("Skipping vector {} with zero embedding", vector_data.id);
            continue; // Skip zero vectors
        }
        
        let payload = vector_data.payload.map(Payload::new);
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
            Ok((
                StatusCode::CREATED,
                Json(InsertVectorsResponse {
                    message: "Vectors inserted successfully".to_string(),
                    inserted: vector_count,
                    inserted_count: vector_count,
                }),
            ))
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
    Json(request): Json<SearchUnifiedRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = Instant::now();

    // Determine input type and prepare vector + optional threshold and limit
    let (query_vector, limit, score_threshold) = match request {
        SearchUnifiedRequest::Vector(req) => {
            debug!(
                "Searching (vector) in collection '{}' with limit: {:?}",
                collection_name, req.limit
            );
            (
                req.vector,
                req.limit.unwrap_or(10).min(100),
                req.score_threshold,
            )
        }
        SearchUnifiedRequest::Text(req) => {
            debug!(
                "Searching (text) in collection '{}' with limit: {:?}",
                collection_name, req.limit
            );
            // Get collection info to determine embedding dimension
            let collection_info = match state.store.get_collection_metadata(&collection_name) {
                Ok(metadata) => metadata,
                Err(_) => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse {
                            error: format!("Collection '{}' not found", collection_name),
                            code: "COLLECTION_NOT_FOUND".to_string(),
                            details: None,
                        }),
                    ));
                }
            };

            let embedding_dimension = collection_info.config.dimension;
            let vector = match create_text_embedding(&req.query, embedding_dimension) {
                Ok(v) => v,
                Err(e) => {
                    error!(
                        "Failed to create embedding for query '{}': {}",
                        req.query, e
                    );
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to create embedding: {}", e),
                            code: "EMBEDDING_ERROR".to_string(),
                            details: None,
                        }),
                    ));
                }
            };
            (
                vector,
                req.limit.unwrap_or(10).min(100),
                req.score_threshold,
            )
        }
    };
    
    // Set default minimum score threshold to filter out poor results
    let min_score = score_threshold.unwrap_or(0.1); // Default minimum score of 0.1

    match state.store.search(&collection_name, &query_vector, limit) {
        Ok(results) => {
            let query_time = start_time.elapsed().as_secs_f64() * 1000.0;

            let mut search_results = Vec::new();
            for result in results {
                // Apply score threshold (use default if not specified)
                if result.score < min_score {
                    continue;
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

/// Search for similar vectors using text query (automatically embedded)
pub async fn search_vectors_by_text(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<super::types::SearchTextRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        "Searching collection '{}' with text query: '{}'",
        collection_name, request.query
    );

    // Validate request
    if request.query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Query text cannot be empty".to_string(),
                code: "INVALID_QUERY".to_string(),
                details: None,
            }),
        ));
    }

    // Get collection info to determine embedding dimension
    let collection_info = match state.store.get_collection_metadata(&collection_name) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Collection '{}' not found", collection_name),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    details: None,
                }),
            ));
        }
    };

    // Get the collection to determine the embedding type used
    let collection = match state.store.get_collection(&collection_name) {
        Ok(collection) => collection,
        Err(e) => {
            error!("Collection '{}' not found: {}", collection_name, e);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Collection '{}' not found", collection_name),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    details: None,
                }),
            ));
        }
    };

    // Get the embedding type used for this collection
    let collection_embedding_type = collection.get_embedding_type();
    debug!("Using embedding type '{}' for collection '{}'", collection_embedding_type, collection_name);

    // Create embedding for the query text using the same embedding type as the collection
    let query_vector = {
        let manager = state.embedding_manager.lock().unwrap();
        match manager.embed_with_provider(&collection_embedding_type, &request.query) {
            Ok(vector) => {
                // Validate embedding - reject zero vectors
                let non_zero_count = vector.iter().filter(|&&x| x != 0.0).count();
                if non_zero_count == 0 {
                    error!("Query embedding is zero for '{}' with provider '{}'", request.query, collection_embedding_type);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Query '{}' produced zero embedding with provider '{}'. Try a different query or check vocabulary.", request.query, collection_embedding_type),
                            code: "ZERO_EMBEDDING".to_string(),
                            details: None,
                        }),
                    ));
                }
                vector
            },
            Err(e) => {
                error!(
                    "Failed to create embedding for query '{}' with provider '{}': {}",
                    request.query, collection_embedding_type, e
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to create embedding with provider '{}': {}", collection_embedding_type, e),
                        code: "EMBEDDING_ERROR".to_string(),
                        details: None,
                    }),
                ));
            }
        }
    };

    let start_time = Instant::now();
    let limit = request.limit.unwrap_or(10).min(100); // Cap at 100 results
    
    // Set default minimum score threshold to filter out poor results
    let min_score = request.score_threshold.unwrap_or(0.1); // Default minimum score of 0.1

    match state.store.search(&collection_name, &query_vector, limit) {
        Ok(results) => {
            let query_time = start_time.elapsed().as_secs_f64() * 1000.0;

            let mut search_results = Vec::new();
            for result in results {
                // Apply score threshold (use default if not specified)
                if result.score < min_score {
                    continue;
                }

                // Apply file filter if specified
                if let Some(file_filter) = &request.file_filter {
                    if let Some(payload) = &result.payload {
                        if let Some(file_path) = payload.data.get("file_path") {
                            if let Some(file_path_str) = file_path.as_str() {
                                if !file_path_str.contains(file_filter) {
                                    continue;
                                }
                            }
                        }
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
                "Text search completed in {:.2}ms, found {} results",
                query_time,
                search_results.len()
            );

            Ok(Json(SearchResponse {
                results: search_results,
                query_time_ms: query_time,
            }))
        }
        Err(e) => {
            error!(
                "Text search failed in collection '{}': {}",
                collection_name, e
            );
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

/// Create embedding vector from text using TF-IDF approach with better word weighting
fn create_text_embedding(query: &str, dimension: usize) -> anyhow::Result<Vec<f32>> {
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher, DefaultHasher};
    
    // Tokenize and clean the query (same as TfIdfEmbedding)
    let words: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect();
    
    if words.is_empty() {
        return Ok(vec![0.0f32; dimension]);
    }
    
    // Compute TF (same as TfIdfEmbedding)
    let total_words = words.len() as f32;
    let mut word_counts: HashMap<String, usize> = HashMap::new();
    for word in words {
        *word_counts.entry(word).or_insert(0) += 1;
    }
    
    let tf_values: HashMap<String, f32> = word_counts
        .into_iter()
        .map(|(word, count)| (word, count as f32 / total_words))
        .collect();
    
    let mut embedding = vec![0.0f32; dimension];
    
    // Create embedding using consistent hashing (similar to TfIdfEmbedding approach)
    for (word, tf_value) in tf_values {
        // Use consistent hashing to map words to dimensions
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        let word_hash = hasher.finish();
        
        // Map word to multiple dimensions for better coverage
        for i in 0..dimension {
            let dim_seed = (word_hash.wrapping_add(i as u64 * 7919)) % dimension as u64;
            let dim_index = dim_seed as usize;
            
            // Use TF value with some variation based on word characteristics
            let idf_approx = 1.0 + (word.len() as f32).ln(); // Simple IDF approximation
            let weight = tf_value * idf_approx;
            
            // Add weight to the dimension
            embedding[dim_index] += weight;
        }
    }
    
    // L2 normalize the embedding (same as TfIdfEmbedding)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut embedding {
            *value /= norm;
        }
    }
    
    // Debug: Check if embedding is all zeros
    let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
    if non_zero_count == 0 {
        // Fallback: create a simple hash-based embedding for zero embeddings
        let mut fallback_embedding = vec![0.0f32; dimension];
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let query_hash = hasher.finish();
        
        for i in 0..dimension {
            let seed = (query_hash.wrapping_add(i as u64 * 7919)) % dimension as u64;
            fallback_embedding[i] = ((seed % 1000) as f32 / 1000.0) - 0.5;
        }
        
        // Normalize fallback
        let norm: f32 = fallback_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut fallback_embedding {
                *value /= norm;
            }
        }
        
        return Ok(fallback_embedding);
    }
    
    Ok(embedding)
}

/// List available embedding providers
pub async fn list_embedding_providers(State(state): State<AppState>) -> Json<ListEmbeddingProvidersResponse> {
    let manager = state.embedding_manager.lock().unwrap();
    let providers = manager.list_providers();
    let default_provider = manager.get_default_provider().ok().map(|_| "default".to_string());
    
    Json(ListEmbeddingProvidersResponse {
        providers,
        default_provider,
    })
}

/// Set the default embedding provider
pub async fn set_embedding_provider(
    State(state): State<AppState>,
    Json(request): Json<SetEmbeddingProviderRequest>,
) -> Result<Json<SetEmbeddingProviderResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut manager = state.embedding_manager.lock().unwrap();
    
    if !manager.has_provider(&request.provider_name) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Provider '{}' not found", request.provider_name),
                code: "PROVIDER_NOT_FOUND".to_string(),
                details: None,
            }),
        ));
    }

    match manager.set_default_provider(&request.provider_name) {
        Ok(_) => {
            // Update embedding type for all existing collections
            let collections = state.store.list_collections();
            for collection_name in collections {
                if let Ok(collection) = state.store.get_collection(&collection_name) {
                    collection.set_embedding_type(request.provider_name.clone());
                    info!("Updated embedding type to '{}' for collection '{}'", request.provider_name, collection_name);
                }
            }
            
            Ok(Json(SetEmbeddingProviderResponse {
                success: true,
                message: format!("Default provider set to '{}' and updated all collections", request.provider_name),
                provider_name: request.provider_name,
            }))
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to set provider: {}", e),
                code: "PROVIDER_SET_ERROR".to_string(),
                details: None,
            }),
        )),
    }
}

/// Get a specific vector by ID
pub async fn get_vector(
    State(state): State<AppState>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<VectorData>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        "Getting vector '{}' from collection '{}'",
        vector_id, collection_name
    );

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
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Deleting vector '{}' from collection '{}'",
        vector_id, collection_name
    );

    match state.store.delete(&collection_name, &vector_id) {
        Ok(_) => {
            info!(
                "Vector '{}' deleted successfully from collection '{}'",
                vector_id, collection_name
            );
            Ok(StatusCode::NO_CONTENT)
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

/// List vectors from a collection with pagination
pub async fn list_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
) -> Result<Json<ListVectorsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = Instant::now();
    
    info!("Listing vectors from collection: {}", collection_name);
    
    // Parse query parameters for pagination
    let limit = 10; // Default limit
    let offset = 0; // Default offset
    
    match state.store.get_collection(&collection_name) {
        Ok(collection) => {
            // For now, we'll return a mock response since we don't have a direct list_vectors method
            // In a real implementation, you'd need to add this method to VectorStore
            let vectors = vec![
                Vector {
                    id: "vector_1".to_string(),
                    data: vec![0.1, 0.2, 0.3], // Mock data
                    payload: Some(Payload {
                        data: serde_json::json!({
                            "content": "Sample document content 1",
                            "file_path": "sample1.md",
                            "chunk_index": 0
                        }),
                    }),
                },
                Vector {
                    id: "vector_2".to_string(),
                    data: vec![0.4, 0.5, 0.6], // Mock data
                    payload: Some(Payload {
                        data: serde_json::json!({
                            "content": "Sample document content 2",
                            "file_path": "sample2.md",
                            "chunk_index": 1
                        }),
                    }),
                },
            ];
            
            let response = ListVectorsResponse {
                vectors: vectors.into_iter().map(|v| VectorResponse {
                    id: v.id,
                    payload: v.payload.map(|p| p.data),
                }).collect(),
                total: collection.metadata().vector_count,
                limit,
                offset,
            };
            
            let duration = start_time.elapsed();
            info!("Listed {} vectors from collection '{}' in {:?}", 
                  response.vectors.len(), collection_name, duration);
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get collection '{}': {}", collection_name, e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Collection not found".to_string(),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

/// Search for vectors by file path
pub async fn search_by_file(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<super::types::SearchByFileRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        "Searching collection '{}' for file: '{}'",
        collection_name, request.file_path
    );

    // Validate request
    if request.file_path.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "File path cannot be empty".to_string(),
                code: "INVALID_FILE_PATH".to_string(),
                details: None,
            }),
        ));
    }

    // Check if collection exists
    if state.store.get_collection_metadata(&collection_name).is_err() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Collection '{}' not found", collection_name),
                code: "COLLECTION_NOT_FOUND".to_string(),
                details: None,
            }),
        ));
    }

    let start_time = Instant::now();
    let limit = request.limit.unwrap_or(100).min(1000); // Cap at 1000 results for file search

    // Get all vectors and filter by file path
    let collection = match state.store.get_collection(&collection_name) {
        Ok(collection) => collection,
        Err(e) => {
            error!("Failed to get collection '{}': {}", collection_name, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to search vectors".to_string(),
                    code: "SEARCH_ERROR".to_string(),
                    details: None,
                }),
            ));
        }
    };
    
    let all_vectors = collection.get_all_vectors();

    let mut search_results = Vec::new();
    for vector in all_vectors {
        if let Some(payload) = &vector.payload {
            if let Some(file_path) = payload.data.get("file_path") {
                if let Some(file_path_str) = file_path.as_str() {
                    if file_path_str.contains(&request.file_path) {
                        // Create a dummy vector for the result (we're filtering by metadata, not similarity)
                        let dummy_vector = vec![0.0; 512]; // Default dimension
                        
                        search_results.push(super::types::SearchResult {
                            id: vector.id,
                            score: 1.0, // Perfect match for file path
                            vector: dummy_vector,
                            payload: Some(payload.data.clone()),
                        });

                        if search_results.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }
    }

    let query_time = start_time.elapsed().as_secs_f64() * 1000.0;

    debug!(
        "File search completed in {:.2}ms, found {} results for file '{}'",
        query_time,
        search_results.len(),
        request.file_path
    );

    Ok(Json(SearchResponse {
        results: search_results,
        query_time_ms: query_time,
    }))
}

/// List all files in a collection
pub async fn list_files(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<super::types::ListFilesRequest>,
) -> Result<Json<super::types::ListFilesResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Listing files in collection '{}'", collection_name);

    // Check if collection exists
    if state.store.get_collection_metadata(&collection_name).is_err() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Collection '{}' not found", collection_name),
                code: "COLLECTION_NOT_FOUND".to_string(),
                details: None,
            }),
        ));
    }

    let limit = request.limit.unwrap_or(50).min(500);
    let offset = request.offset.unwrap_or(0);

    // Get all vectors to extract file information
    let collection = match state.store.get_collection(&collection_name) {
        Ok(collection) => collection,
        Err(e) => {
            error!("Failed to get collection '{}': {}", collection_name, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to list files".to_string(),
                    code: "LIST_ERROR".to_string(),
                    details: None,
                }),
            ));
        }
    };
    
    let all_vectors = collection.get_all_vectors();

    // Group vectors by file path
    let mut file_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut file_extensions: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();

    for vector in all_vectors {
        if let Some(payload) = &vector.payload {
            if let Some(file_path) = payload.data.get("file_path") {
                if let Some(file_path_str) = file_path.as_str() {
                    *file_map.entry(file_path_str.to_string()).or_insert(0) += 1;
                    
                    // Extract file extension
                    if let Some(extension) = std::path::Path::new(file_path_str).extension() {
                        file_extensions.insert(file_path_str.to_string(), Some(extension.to_string_lossy().to_string()));
                    } else {
                        file_extensions.insert(file_path_str.to_string(), None);
                    }
                }
            }
        }
    }

    // Convert to FileInfo and apply filters
    let mut files: Vec<super::types::FileInfo> = file_map
        .into_iter()
        .map(|(file_path, chunk_count)| {
            let extension = file_extensions.get(&file_path).cloned().flatten();
            super::types::FileInfo {
                file_path,
                chunk_count,
                extension,
            }
        })
        .collect();

    // Apply extension filter if specified
    if let Some(extension_filter) = &request.extension_filter {
        files.retain(|file| {
            file.extension.as_ref().map_or(false, |ext| ext == extension_filter)
        });
    }

    // Sort by file path for consistent ordering
    files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    let total = files.len();
    let files: Vec<super::types::FileInfo> = files
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    debug!(
        "Listed {} files from collection '{}' (total: {})",
        files.len(),
        collection_name,
        total
    );

    Ok(Json(super::types::ListFilesResponse {
        files,
        total,
        limit,
        offset,
    }))
}

// ============================================================================
// MCP (Model Context Protocol) ENDPOINTS - REST API Implementation
// ============================================================================

use serde::{Deserialize, Serialize};

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCapabilities {
    pub tools: Option<McpToolCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCapabilities {
    pub supported: bool,
}

/// MCP Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

/// MCP Initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeRequest {
    pub protocol_version: String,
    pub capabilities: serde_json::Value,
    pub client_info: serde_json::Value,
}

/// MCP Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeResult {
    pub protocol_version: String,
    pub capabilities: McpServerCapabilities,
    pub server_info: McpServerInfo,
}

/// List MCP tools
pub async fn mcp_tools_list(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    debug!("MCP tools/list requested");

    // Define available MCP tools based on existing API
    let tools = vec![
        serde_json::json!({
            "name": "search_vectors",
            "description": "Search for similar vectors in a collection",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query text"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "default": 10
                    }
                },
                "required": ["collection", "query"]
            }
        }),
        serde_json::json!({
            "name": "list_collections",
            "description": "List all available collections",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        serde_json::json!({
            "name": "embed_text",
            "description": "Generate embeddings for text",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to embed"
                    }
                },
                "required": ["text"]
            }
        }),
    ];

    let response = serde_json::json!({
        "tools": tools
    });

    debug!("MCP tools/list response: {} tools available", tools.len());
    Ok(Json(response))
}

/// MCP tool call handler
pub async fn mcp_tools_call(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    debug!("MCP tools/call requested: {:?}", request);

    let request_id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);
    
    let params = request.get("params")
        .and_then(|p| p.as_object())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request",
                    "data": "Missing params"
                }
            }))
        ))?;

    let tool_name = params.get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request",
                    "data": "Missing tool name"
                }
            }))
        ))?;

    let empty_map = serde_json::Map::new();
    let arguments = params.get("arguments")
        .and_then(|a| a.as_object())
        .unwrap_or(&empty_map);

    let result = match tool_name {
        "search_vectors" => {
            let collection = arguments.get("collection")
                .and_then(|c| c.as_str())
                .ok_or_else(|| (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "error": {
                            "code": -32602,
                            "message": "Invalid params",
                            "data": "Missing collection parameter"
                        }
                    }))
                ))?;

            let query = arguments.get("query")
                .and_then(|q| q.as_str())
                .ok_or_else(|| (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "error": {
                            "code": -32602,
                            "message": "Invalid params",
                            "data": "Missing query parameter"
                        }
                    }))
                ))?;

            let limit = arguments.get("limit")
                .and_then(|l| l.as_u64())
                .unwrap_or(10) as usize;

            // Use existing search handler directly
            match search_vectors_by_text(
                State(state.clone()),
                Path(collection.to_string()),
                Json(super::types::SearchTextRequest {
                    query: query.to_string(),
                    limit: Some(limit),
                    score_threshold: Some(0.1),
                    file_filter: None,
                })
            ).await {
                Ok(Json(response)) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
                            }
                        ]
                    }
                }),
                Err((status, Json(error_response))) => {
                    return Err((
                        status,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {
                                "code": -32603,
                                "message": "Internal error",
                                "data": error_response.error
                            }
                        }))
                    ));
                }
            }
        },

        "list_collections" => {
            // Use existing collections handler directly
            let result = list_collections(State(state.clone())).await;
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string(&result.0).unwrap_or_else(|_| "{}".to_string())
                        }
                    ]
                }
            })
        },

        "embed_text" => {
            let text = arguments.get("text")
                .and_then(|t| t.as_str())
                .ok_or_else(|| (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "error": {
                            "code": -32602,
                            "message": "Invalid params",
                            "data": "Missing text parameter"
                        }
                    }))
                ))?;

            // Generate embedding directly using the embedding manager
            match state.embedding_manager.lock().unwrap().embed_with_provider("bm25", text) {
                Ok(embedding) => {
                    let response = serde_json::json!({
                        "embedding": embedding,
                        "dimension": embedding.len(),
                        "provider": "bm25"
                    });
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
                                }
                            ]
                        }
                    })
                },
                Err(e) => return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "error": {
                            "code": -32603,
                            "message": "Internal error",
                            "data": format!("Embedding failed: {}", e)
                        }
                    }))
                )),
            }
        },

        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found",
                        "data": format!("Unknown tool: {}", tool_name)
                    }
                }))
            ));
        }
    };

    debug!("MCP tools/call completed for tool: {}", tool_name);
    Ok(Json(result))
}

/// MCP initialize handler
pub async fn mcp_initialize(
    State(_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    debug!("MCP initialize requested: {:?}", request);

    let request_id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);

    let result = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "supported": true }
            },
            "serverInfo": {
                "name": "Vectorizer MCP Server",
                "version": "1.0.0"
            }
        }
    });

    debug!("MCP initialize completed");
    Ok(Json(result))
}

/// MCP ping handler
pub async fn mcp_ping() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    debug!("MCP ping requested");
    Ok(Json(serde_json::json!({"status": "pong"})))
}

/// MCP SSE endpoint for real-time communication
pub async fn mcp_sse(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    use futures_util::stream;
    use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    debug!("MCP SSE connection established");

    // Create a channel for sending messages
    let (tx, rx) = mpsc::unbounded_channel();
    let tx = Arc::new(tx);

    // Spawn a task to handle incoming requests
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // Send initial server capabilities
        let init_message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "server/initialized",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "supported": true }
                },
                "serverInfo": {
                    "name": "Vectorizer MCP Server",
                    "version": "1.0.0"
                }
            }
        });

        let _ = tx_clone.send(axum::response::sse::Event::default().data(init_message.to_string()));
        
        // Send tools list
        let tools_message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "params": {
                "tools": [
                    {
                        "name": "search_vectors",
                        "description": "Search for similar vectors in a collection",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "collection": {
                                    "type": "string",
                                    "description": "Collection name"
                                },
                                "query": {
                                    "type": "string",
                                    "description": "Search query text"
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of results",
                                    "default": 10
                                }
                            },
                            "required": ["collection", "query"]
                        }
                    },
                    {
                        "name": "list_collections",
                        "description": "List all available collections",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "embed_text",
                        "description": "Generate embeddings for text",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": {
                                    "type": "string",
                                    "description": "Text to embed"
                                }
                            },
                            "required": ["text"]
                        }
                    }
                ]
            }
        });

        let _ = tx_clone.send(axum::response::sse::Event::default().data(tools_message.to_string()));
    });

    // Create stream from receiver
    let stream = UnboundedReceiverStream::new(rx)
        .map(|event| Ok(event));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("keepalive")
    )
}

/// MCP HTTP endpoint for tool calls (fallback for SSE)
pub async fn mcp_http_tools_call(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    debug!("MCP HTTP tools/call requested: {:?}", request);

    // Extract the method and params from the request
    let method = request["method"].as_str()
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing method"}))
        ))?;

    let params = request["params"].as_object()
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing params"}))
        ))?;

    let result = match method {
        "tools/call" => {
            let tool_name = params["name"].as_str()
                .ok_or_else(|| (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": -32602, "message": "Missing tool name"}}))
                ))?;

            let arguments = params["arguments"].as_object()
                .ok_or_else(|| (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": -32602, "message": "Missing arguments"}}))
                ))?;

            // Reuse existing tool call logic
            match mcp_tools_call(State(state), Json(request.clone())).await {
                Ok(Json(result)) => serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "result": result}),
                Err((status, Json(error))) => {
                    return Err((status, Json(serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": error}))));
                }
            }
        },
        "tools/list" => {
            match mcp_tools_list(State(state)).await {
                Ok(Json(result)) => serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "result": result}),
                Err((status, Json(error))) => {
                    return Err((status, Json(serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": error}))));
                }
            }
        },
        "initialize" => {
            // Create a mock initialize request
            let init_request = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            });

            match mcp_initialize(State(state), Json(init_request)).await {
                Ok(Json(result)) => result,
                Err((status, Json(error))) => {
                    return Err((status, Json(error)));
                }
            }
        },
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": -32601, "message": format!("Unknown method: {}", method)}}))
            ));
        }
    };

    Ok(Json(result))
}
