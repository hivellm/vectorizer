//! HTTP request handlers for the Vectorizer API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Sse},
};
use chrono::{Duration, Utc};
use futures_util::{Stream, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tracing::{debug, error, info, warn};

use crate::{
    VectorStore,
    embedding::{Bm25Embedding, EmbeddingManager},
    models::{CollectionConfig, Payload, Vector},
    grpc::client::VectorizerGrpcClient,
    config::GrpcConfig,
    batch::{BatchProcessor, BatchConfig, BatchOperation, BatchProcessorBuilder},
};
use std::sync::Mutex;

use super::types::*;

/// Helper function to convert BatchConfigRequest to BatchConfig
fn convert_batch_config(config: Option<BatchConfigRequest>) -> BatchConfig {
    if let Some(config) = config {
        BatchConfig {
            max_batch_size: config.max_batch_size.unwrap_or(1000),
            max_memory_usage_mb: config.max_memory_usage_mb.unwrap_or(512),
            parallel_workers: config.parallel_workers.unwrap_or(4),
            chunk_size: config.chunk_size.unwrap_or(100),
            atomic_by_default: true,
            progress_reporting: config.progress_reporting.unwrap_or(true),
            error_retry_attempts: 3,
            error_retry_delay_ms: 1000,
            operation_timeout_seconds: 300,
            enable_metrics: true,
            max_concurrent_batches: 10,
            enable_compression: true,
            compression_threshold_bytes: 1024,
        }
    } else {
        BatchConfig::default()
    }
}

/// Workspace collection definition
#[derive(Clone, Debug)]
pub struct WorkspaceCollection {
    pub name: String,
    pub description: String,
    pub dimension: u64,
    pub metric: String,
    pub model: String,
}

/// Shared indexing progress state with watch-based notifications
#[derive(Clone)]
pub struct IndexingProgressState {
    map: Arc<Mutex<HashMap<String, IndexingStatus>>>,
    sender: watch::Sender<HashMap<String, IndexingStatus>>,
}

impl IndexingProgressState {
    /// Create a new progress state from an initial map
    pub fn from_map(initial: HashMap<String, IndexingStatus>) -> Self {
        let map = Arc::new(Mutex::new(initial.clone()));
        let (sender, _) = watch::channel(initial);
        // Keep an initial subscriber alive so broadcast never fails when no dashboard clients are connected yet
        let guard = sender.subscribe();
        std::mem::forget(guard);
        Self { map, sender }
    }

    /// Create an empty progress state
    pub fn new() -> Self {
        Self::from_map(HashMap::new())
    }

    /// Update progress for a collection and broadcast the new snapshot
    pub fn update(
        &self,
        collection_name: &str,
        status: &str,
        progress: f32,
        total_documents: usize,
        processed_documents: usize,
    ) {
        info!(
            "?? Updating progress for '{}' to status '{}' progress {:.1}%",
            collection_name, status, progress
        );

        let mut map_guard = self.map.lock().unwrap();
        let old_count = map_guard.len();
        map_guard.insert(
            collection_name.to_string(),
            IndexingStatus {
                status: status.to_string(),
                progress,
                total_documents,
                processed_documents,
                vector_count: 0,
                estimated_time_remaining: None,
                last_updated: Utc::now().to_rfc3339(),
            },
        );
        let new_count = map_guard.len();
        let snapshot = map_guard.clone();
        drop(map_guard);

        if let Err(err) = self.sender.send(snapshot) {
            warn!(
                "Failed to broadcast indexing progress update for '{}': {}",
                collection_name, err
            );
            return;
        }

        info!(
            "?? Progress map updated: {} -> {} entries",
            old_count, new_count
        );
    }

    /// Ensure a collection status exists without overwriting non-default entries
    pub fn ensure_status<F>(&self, collection_name: &str, builder: F)
    where
        F: FnOnce() -> IndexingStatus,
    {
        let mut map_guard = self.map.lock().unwrap();
        if map_guard.contains_key(collection_name) {
            return;
        }

        let status = builder();
        map_guard.insert(collection_name.to_string(), status.clone());
        let snapshot = map_guard.clone();
        drop(map_guard);

        if let Err(err) = self.sender.send(snapshot) {
            warn!(
                "Failed to initialize indexing progress entry for '{}': {}",
                collection_name, err
            );
        } else {
            info!(
                "?? Initialized progress entry for '{}' with default status '{}'",
                collection_name, status.status
            );
        }
    }

    /// Snapshot the current progress map
    pub fn snapshot(&self) -> HashMap<String, IndexingStatus> {
        self.map.lock().unwrap().clone()
    }

    /// Subscribe to progress updates for streaming APIs
    pub fn subscribe(&self) -> watch::Receiver<HashMap<String, IndexingStatus>> {
        self.sender.subscribe()
    }
}
/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Vector store instance
    pub store: Arc<VectorStore>,
    /// Embedding manager for consistent embedding generation
    pub embedding_manager: Arc<Mutex<EmbeddingManager>>,
    /// GRPC client for communication with vzr
    pub grpc_client: Option<VectorizerGrpcClient>,
    /// Server start time for uptime calculation
    pub start_time: Instant,
    /// Indexing progress tracking
    pub indexing_progress: IndexingProgressState,
    /// Workspace collections (all defined collections, even if not yet indexed)
    pub workspace_collections: Vec<WorkspaceCollection>,
    /// File watcher system for real-time file monitoring
    pub file_watcher: Option<Arc<Mutex<crate::file_watcher::FileWatcherSystem>>>,
}

impl AppState {
    /// Create new application state
    pub fn new(store: Arc<VectorStore>, mut embedding_manager: EmbeddingManager) -> Self {
        // Check if BM25 vocabulary is empty and needs rebuilding
        // Note: Vocabulary is built during document loading, so empty here is expected

        // Initialize GRPC client
        let grpc_client = None; // Will be initialized later if needed

        // Initialize indexing progress for existing collections
        let mut indexing_progress = HashMap::new();
        for collection_name in store.list_collections() {
            indexing_progress.insert(
                collection_name,
                IndexingStatus {
                    status: "completed".to_string(),
                    progress: 100.0,
                    total_documents: 0, // Will be updated when we have stats
                    processed_documents: 0,
                    vector_count: 0,
                    estimated_time_remaining: None,
                    last_updated: Utc::now().to_rfc3339(),
                },
            );
        }

        // Load workspace collections
        let workspace_collections = Self::load_workspace_collections();

        // Initialize indexing progress for all workspace collections
        for collection in &workspace_collections {
            if !indexing_progress.contains_key(&collection.name) {
                indexing_progress.insert(
                    collection.name.clone(),
                    IndexingStatus {
                        status: "pending".to_string(),
                        progress: 0.0,
                        total_documents: 0,
                        processed_documents: 0,
                        vector_count: 0,
                        estimated_time_remaining: None,
                        last_updated: Utc::now().to_rfc3339(),
                    },
                );
            }
        }

        Self {
            store,
            embedding_manager: Arc::new(Mutex::new(embedding_manager)),
            grpc_client,
            start_time: Instant::now(),
            indexing_progress: IndexingProgressState::from_map(indexing_progress),
            workspace_collections,
            file_watcher: None,
        }
    }

    /// Initialize GRPC client
    pub async fn init_grpc_client(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = GrpcConfig::from_env();
        
        match VectorizerGrpcClient::new(config.client).await {
            Ok(client) => {
                self.grpc_client = Some(client);
                info!("✅ GRPC client initialized successfully with config");
                Ok(())
            }
            Err(e) => {
                warn!("⚠️ Failed to initialize GRPC client: {}", e);
                Err(e.into())
            }
        }
    }

    /// Load workspace collections from environment variable or YAML file
    pub fn load_workspace_collections() -> Vec<WorkspaceCollection> {
        // Try to load from environment variable first (JSON format)
        if let Ok(workspace_path) = std::env::var("VECTORIZER_WORKSPACE_INFO") {
            if let Ok(content) = std::fs::read_to_string(&workspace_path) {
                if let Ok(workspace_json) = serde_json::from_str::<serde_json::Value>(&content) {
                    return Self::parse_workspace_json(&workspace_json);
                }
            }
        }
        
        // Try to load from YAML file directly
        if let Ok(workspace_path) = std::env::var("VECTORIZER_WORKSPACE_INFO") {
            if workspace_path.ends_with(".yml") || workspace_path.ends_with(".yaml") {
                if let Ok(content) = std::fs::read_to_string(&workspace_path) {
                    if let Ok(workspace_yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        return Self::parse_workspace_yaml(&workspace_yaml);
                    }
                }
            }
        }
        
        Vec::new()
    }
    
    /// Parse workspace from JSON format
    fn parse_workspace_json(workspace_json: &serde_json::Value) -> Vec<WorkspaceCollection> {
        if let Some(projects) = workspace_json.get("projects").and_then(|p| p.as_array()) {
            let mut collections = Vec::new();
            for project in projects {
                if let Some(project_collections) = project.get("collections").and_then(|c| c.as_array()) {
                    for collection in project_collections {
                        if let (Some(name), Some(description)) = (
                            collection.get("name").and_then(|n| n.as_str()),
                            collection.get("description").and_then(|d| d.as_str()),
                        ) {
                            let dimension = collection
                                .get("dimension")
                                .and_then(|d| d.as_u64())
                                .unwrap_or(512);
                            let metric = collection
                                .get("metric")
                                .and_then(|m| m.as_str())
                                .unwrap_or("cosine")
                                .to_string();
                            let model = collection
                                .get("embedding")
                                .and_then(|e| e.get("model"))
                                .and_then(|m| m.as_str())
                                .unwrap_or("bm25")
                                .to_string();

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
        Vec::new()
    }
    
    /// Parse workspace from YAML format
    fn parse_workspace_yaml(workspace_yaml: &serde_yaml::Value) -> Vec<WorkspaceCollection> {
        if let Some(projects) = workspace_yaml.get("projects").and_then(|p| p.as_sequence()) {
            let mut collections = Vec::new();
            for project in projects {
                if let Some(project_collections) = project.get("collections").and_then(|c| c.as_sequence()) {
                    for collection in project_collections {
                        if let (Some(name), Some(description)) = (
                            collection.get("name").and_then(|n| n.as_str()),
                            collection.get("description").and_then(|d| d.as_str()),
                        ) {
                            let dimension = collection
                                .get("dimension")
                                .and_then(|d| d.as_u64())
                                .unwrap_or(512);
                            let metric = collection
                                .get("metric")
                                .and_then(|m| m.as_str())
                                .unwrap_or("cosine")
                                .to_string();
                            let model = collection
                                .get("embedding")
                                .and_then(|e| e.get("model"))
                                .and_then(|m| m.as_str())
                                .unwrap_or("bm25")
                                .to_string();

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
        Vec::new()
    }

    /// Create new application state with shared indexing progress
    pub fn new_with_progress(
        store: Arc<VectorStore>,
        embedding_manager: EmbeddingManager,
        indexing_progress: IndexingProgressState,
    ) -> Self {
        // Load workspace collections
        let workspace_collections = Self::load_workspace_collections();

        // Initialize indexing progress for all workspace collections
        for collection in &workspace_collections {
            indexing_progress.ensure_status(&collection.name, || IndexingStatus {
                status: "pending".to_string(),
                progress: 0.0,
                total_documents: 0,
                processed_documents: 0,
                vector_count: 0,
                estimated_time_remaining: None,
                last_updated: Utc::now().to_rfc3339(),
            });
        }

        Self {
            store,
            embedding_manager: Arc::new(Mutex::new(embedding_manager)),
            grpc_client: None, // Will be initialized later if needed
            start_time: Instant::now(),
            indexing_progress,
            workspace_collections,
            file_watcher: None,
        }
    }

    /// Initialize file watcher system
    pub async fn init_file_watcher(
        &mut self,
        config: crate::file_watcher::FileWatcherConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing File Watcher System...");
        
        // Convert Mutex to RwLock for compatibility
        // For now, create a new EmbeddingManager with default providers
        let mut new_manager = EmbeddingManager::new();
        
        // Register default providers
        use crate::embedding::{TfIdfEmbedding, Bm25Embedding};
        let tfidf = TfIdfEmbedding::new(128);
        let bm25 = Bm25Embedding::new(128);
        new_manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        new_manager.register_provider("bm25".to_string(), Box::new(bm25));
        new_manager.set_default_provider("tfidf").unwrap();
        
        let embedding_manager_rwlock = Arc::new(tokio::sync::RwLock::new(new_manager));
        
        let file_watcher = crate::file_watcher::FileWatcherSystem::new(
            config,
            self.store.clone(),
            embedding_manager_rwlock,
            None, // No GRPC client for now
        );

        // Start the file watcher
        file_watcher.start().await?;
        
        self.file_watcher = Some(Arc::new(Mutex::new(file_watcher)));
        info!("File Watcher System initialized successfully");
        
        Ok(())
    }

    /// Stop file watcher system
    pub async fn stop_file_watcher(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(watcher) = self.file_watcher.take() {
            info!("Stopping File Watcher System...");
            let mut watcher = watcher.lock().unwrap();
            watcher.stop().await?;
            info!("File Watcher System stopped");
        }
        Ok(())
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
pub async fn list_collections(State(mut state): State<AppState>) -> Json<ListCollectionsResponse> {
    debug!("Listing collections");

    // Try to use GRPC client if available
    if let Some(ref mut grpc_client) = state.grpc_client {
        match grpc_client.list_collections().await {
            Ok(grpc_response) => {
                info!("📊 API: Using GRPC response with {} collections", grpc_response.total_collections);
                
                let collection_infos = grpc_response.collections
                    .into_iter()
                    .map(|collection| CollectionInfo {
                        name: collection.name,
                        vector_count: collection.vector_count as usize,
                        document_count: collection.document_count as usize,
                        dimension: collection.dimension as usize,
                        metric: DistanceMetric::Cosine, // Default metric
                        created_at: collection.last_updated.clone(),
                        updated_at: collection.last_updated,
                        indexing_status: IndexingStatus {
                            status: collection.status,
                            progress: 100.0, // Assume completed if loaded
                            total_documents: 0,
                            processed_documents: 0,
                            vector_count: collection.vector_count as usize,
                            estimated_time_remaining: None,
                            last_updated: chrono::Utc::now().to_rfc3339(),
                        },
                    })
                    .collect();

                return Json(ListCollectionsResponse {
                    collections: collection_infos,
                });
            }
            Err(e) => {
                warn!("⚠️ GRPC call failed, falling back to local store: {}", e);
            }
        }
    }

    // Fallback to local store
    let existing_collections = state.store.list_collections();
    let mut collection_infos = Vec::new();

    let indexing_progress = state.indexing_progress.snapshot();
    info!(
        "📊 API: Indexing progress map has {} entries",
        indexing_progress.len()
    );

    // First, add all workspace-defined collections
    for workspace_collection in &state.workspace_collections {
        let name = &workspace_collection.name;

        let (metadata, indexing_status) = if existing_collections.contains(name) {
            // Collection exists in vector store
            if let Ok(metadata) = state.store.get_collection_metadata(name) {
                let status =
                    indexing_progress
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| IndexingStatus {
                            status: "completed".to_string(),
                            progress: 100.0,
                            total_documents: 0,
                            processed_documents: 0,
                            vector_count: 0,
                            estimated_time_remaining: None,
                            last_updated: Utc::now().to_rfc3339(),
                        });

                (Some(metadata), status)
            } else {
                // Collection exists but can't get metadata - show as error
                (
                    None,
                    IndexingStatus {
                        status: "error".to_string(),
                        progress: 0.0,
                        total_documents: 0,
                        processed_documents: 0,
                        vector_count: 0,
                        estimated_time_remaining: None,
                        last_updated: Utc::now().to_rfc3339(),
                    },
                )
            }
        } else {
            // Collection defined in workspace but not yet indexed
            (
                None,
                indexing_progress
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| IndexingStatus {
                        status: "pending".to_string(),
                        progress: 0.0,
                        total_documents: 0,
                        processed_documents: 0,
                        vector_count: 0,
                        estimated_time_remaining: None,
                        last_updated: Utc::now().to_rfc3339(),
                    }),
            )
        };

        if let Some(metadata) = metadata {
            // Collection exists
            collection_infos.push(CollectionInfo {
                name: metadata.name,
                dimension: metadata.config.dimension,
                metric: metadata.config.metric.into(),
                vector_count: metadata.vector_count,
                document_count: metadata.document_count,
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
                document_count: 0,
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
                let indexing_status =
                    indexing_progress
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| IndexingStatus {
                            status: "completed".to_string(),
                            progress: 100.0,
                            total_documents: 0,
                            processed_documents: 0,
                            vector_count: 0,
                            estimated_time_remaining: None,
                            last_updated: Utc::now().to_rfc3339(),
                        });

                collection_infos.push(CollectionInfo {
                    name: metadata.name,
                    dimension: metadata.config.dimension,
                    metric: metadata.config.metric.into(),
                    vector_count: metadata.vector_count,
                    document_count: metadata.document_count,
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
pub async fn get_indexing_progress(
    State(state): State<AppState>,
) -> Json<IndexingProgressResponse> {
    debug!("Getting indexing progress");

    let snapshot = state.indexing_progress.snapshot();
    let response = build_indexing_progress_response(&snapshot, state.start_time);
    Json(response)
}

fn build_indexing_progress_response(
    progress_map: &HashMap<String, IndexingStatus>,
    start_time: Instant,
) -> IndexingProgressResponse {
    let collections: Vec<IndexingStatus> = progress_map.values().cloned().collect();

    let overall_status = if collections.is_empty() {
        "idle".to_string()
    } else if collections
        .iter()
        .all(|c| c.status == "completed" || c.status == "cached")
    {
        "completed".to_string()
    } else if collections
        .iter()
        .any(|c| c.status == "processing" || c.status == "indexing")
    {
        "indexing".to_string()
    } else {
        "partial".to_string()
    };

    let estimated_completion = if overall_status == "indexing" {
        let active_collections: Vec<_> = collections
            .iter()
            .filter(|c| {
                (c.status == "processing" || c.status == "indexing")
                    && c.estimated_time_remaining.is_some()
            })
            .collect();

        if !active_collections.is_empty() {
            let max_remaining = active_collections
                .iter()
                .map(|c| c.estimated_time_remaining.unwrap())
                .max()
                .unwrap_or(0);

            let completion_time = Utc::now() + Duration::seconds(max_remaining as i64);
            Some(completion_time.to_rfc3339())
        } else {
            None
        }
    } else {
        None
    };

    let started_at = match Duration::from_std(start_time.elapsed()) {
        Ok(elapsed) => (Utc::now() - elapsed).to_rfc3339(),
        Err(_) => Utc::now().to_rfc3339(),
    };

    IndexingProgressResponse {
        overall_status,
        collections,
        started_at,
        estimated_completion,
    }
}

/// Stream indexing progress updates over Server-Sent Events
pub async fn stream_indexing_progress(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    debug!("Streaming indexing progress via SSE");

    let receiver = state.indexing_progress.subscribe();
    let start_time = state.start_time;

    let stream = WatchStream::new(receiver).map(move |progress_map| {
        let response = build_indexing_progress_response(&progress_map, start_time);
        match serde_json::to_string(&response) {
            Ok(payload) => Ok(axum::response::sse::Event::default().data(payload)),
            Err(err) => {
                warn!("Failed to serialize indexing progress update: {}", err);
                Ok(axum::response::sse::Event::default().data("{}"))
            }
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(5))
            .text("keepalive"),
    )
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
            let indexing_snapshot = state.indexing_progress.snapshot();
            let indexing_status = indexing_snapshot
                .get(&collection_name)
                .cloned()
                .unwrap_or_else(|| IndexingStatus {
                    status: "completed".to_string(),
                    progress: 100.0,
                    total_documents: metadata.vector_count,
                    processed_documents: metadata.vector_count,
                    vector_count: metadata.vector_count,
                    estimated_time_remaining: None,
                    last_updated: Utc::now().to_rfc3339(),
                });

            Ok(Json(CollectionInfo {
                name: metadata.name,
                dimension: metadata.config.dimension,
                metric: metadata.config.metric.into(),
                vector_count: metadata.vector_count,
                document_count: metadata.document_count,
                created_at: metadata.created_at.to_rfc3339(),
                updated_at: metadata.updated_at.to_rfc3339(),
                indexing_status,
            }))
        }
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

    // Convert API vectors to internal format with embedding generation
    let mut vectors = Vec::new();

    for vector_data in request.vectors {
        // Generate embedding if not provided
        let embedding_data = if let Some(vector) = vector_data.vector {
            // Validate embedding - reject zero vectors
            let non_zero_count = vector.iter().filter(|&&x| x != 0.0).count();
            if non_zero_count == 0 {
                warn!("Skipping vector {} with zero embedding", vector_data.id);
                continue; // Skip zero vectors
            }
            vector
        } else {
            // Generate embedding from content
            let manager = state.embedding_manager.lock().unwrap();
            manager.embed(&vector_data.content)
                .map_err(|e| {
                    error!("Failed to generate embedding for vector {}: {}", vector_data.id, e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Embedding generation failed".to_string(),
                            code: "EMBEDDING_GENERATION_FAILED".to_string(),
                            details: Some({
                                let mut map = std::collections::HashMap::new();
                                map.insert("vector_id".to_string(), serde_json::Value::String(vector_data.id.clone()));
                                map.insert("error_message".to_string(), serde_json::Value::String(e.to_string()));
                                map
                            }),
                        })
                    )
                })?
        };

        // Create rich payload with content and metadata
        let mut payload_data = serde_json::Map::new();
        payload_data.insert(
            "content".to_string(),
            serde_json::Value::String(vector_data.content.clone()),
        );

        // Add custom metadata if provided
        if let Some(metadata) = vector_data.metadata {
            if let serde_json::Value::Object(meta_obj) = metadata {
                for (key, value) in meta_obj {
                    payload_data.insert(key, value);
                }
            }
        }

        // Add operation metadata
        payload_data.insert(
            "operation_type".to_string(),
            serde_json::Value::String("single_insert".to_string()),
        );
        payload_data.insert(
            "created_at".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );

        let payload = Payload {
            data: serde_json::Value::Object(payload_data),
        };

        vectors.push(Vector {
            id: vector_data.id,
            data: embedding_data,
            payload: Some(payload),
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
    State(mut state): State<AppState>,
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

    // Check if collection exists (for validation)
    if !collection_exists(&state, &collection_name).await {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Collection '{}' not found", collection_name),
                code: "COLLECTION_NOT_FOUND".to_string(),
                details: None,
            }),
        ));
    }

    // Try to use GRPC client if available for search
    if let Some(ref mut grpc_client) = state.grpc_client {
        match grpc_client.search(collection_name.clone(), request.query.clone(), request.limit.unwrap_or(10) as i32).await {
            Ok(grpc_response) => {
                info!("📊 API: Using GRPC response for search in '{}'", collection_name);

                let results = grpc_response.results
                    .into_iter()
                    .take(request.limit.unwrap_or(10))
                    .map(|r| SearchResult {
                        id: r.id,
                        score: r.score,
                        vector: vec![], // GRPC doesn't return vector data in search results
                        payload: Some(serde_json::json!({
                            "content": r.content,
                            "metadata": r.metadata
                        })),
                    })
                    .collect();

                return Ok(Json(SearchResponse {
                    results,
                    query_time_ms: grpc_response.search_time_ms,
                }));
            }
            Err(e) => {
                warn!("⚠️ GRPC search failed, falling back to local store: {}", e);
            }
        }
    }

    // Get collection info to determine embedding dimension
    let collection_info = match state.store.get_collection_metadata(&collection_name) {
        Ok(metadata) => metadata,
        Err(_) => {
            // Collection exists in workspace but not in local store yet
            // Return empty search result for pending collections
            return Ok(Json(SearchResponse {
                results: vec![],
                query_time_ms: 0.0,
            }));
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
    debug!(
        "Using embedding type '{}' for collection '{}'",
        collection_embedding_type, collection_name
    );

    // Create embedding for the query text using the same embedding type as the collection
    let query_vector = {
        let manager = state.embedding_manager.lock().unwrap();
        match manager.embed_with_provider(&collection_embedding_type, &request.query) {
            Ok(vector) => {
                // Validate embedding - reject zero vectors
                let non_zero_count = vector.iter().filter(|&&x| x != 0.0).count();
                if non_zero_count == 0 {
                    error!(
                        "Query embedding is zero for '{}' with provider '{}'",
                        request.query, collection_embedding_type
                    );
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!(
                                "Query '{}' produced zero embedding with provider '{}'. Try a different query or check vocabulary.",
                                request.query, collection_embedding_type
                            ),
                            code: "ZERO_EMBEDDING".to_string(),
                            details: None,
                        }),
                    ));
                }
                vector
            }
            Err(e) => {
                error!(
                    "Failed to create embedding for query '{}' with provider '{}': {}",
                    request.query, collection_embedding_type, e
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!(
                            "Failed to create embedding with provider '{}': {}",
                            collection_embedding_type, e
                        ),
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
    use std::hash::{DefaultHasher, Hash, Hasher};

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
pub async fn list_embedding_providers(
    State(state): State<AppState>,
) -> Json<ListEmbeddingProvidersResponse> {
    let manager = state.embedding_manager.lock().unwrap();
    let providers = manager.list_providers();
    let default_provider = manager
        .get_default_provider()
        .ok()
        .map(|_| "default".to_string());

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
                    info!(
                        "Updated embedding type to '{}' for collection '{}'",
                        request.provider_name, collection_name
                    );
                }
            }

            Ok(Json(SetEmbeddingProviderResponse {
                success: true,
                message: format!(
                    "Default provider set to '{}' and updated all collections",
                    request.provider_name
                ),
                provider_name: request.provider_name,
            }))
        }
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
        Ok(vector) => {
            // Extract content from payload if available
            let content = if let Some(payload) = &vector.payload {
                if let Some(content_val) = payload.data.get("content") {
                    content_val.as_str().unwrap_or("No content available").to_string()
                } else {
                    "No content available".to_string()
                }
            } else {
                "No content available".to_string()
            };

            // Extract metadata (all fields except content, operation_type, created_at)
            let metadata = if let Some(payload) = &vector.payload {
                let mut meta_obj = serde_json::Map::new();
                if let serde_json::Value::Object(obj) = &payload.data {
                    for (key, value) in obj {
                        if !["content", "operation_type", "created_at", "batch_id"].contains(&key.as_str()) {
                            meta_obj.insert(key.clone(), value.clone());
                        }
                    }
                }
                if meta_obj.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::Object(meta_obj))
                }
            } else {
                None
            };

            Ok(Json(VectorData {
                id: vector.id,
                vector: Some(vector.data),
                content,
                metadata,
            }))
        },
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

/// Check if a collection exists (in workspace or via GRPC)
async fn collection_exists(state: &AppState, collection_name: &str) -> bool {
    // First check workspace collections
    if state.workspace_collections.iter().any(|wc| wc.name == collection_name) {
        return true;
    }

    // If GRPC is available, check if collection exists there
    // Note: This will make an extra call, but it's necessary for validation
    // In a production system, we'd cache this information
    if state.grpc_client.is_some() {
        // For now, assume if GRPC is configured, the collection exists
        // This is a simplification - in production we'd cache the collection list
        return true;
    }

    false
}

/// List vectors from a collection with pagination
pub async fn list_vectors(
    State(mut state): State<AppState>,
    Path(collection_name): Path<String>,
    Query(params): Query<ListVectorsQuery>,
) -> Result<Json<ListVectorsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = Instant::now();

    info!("Listing vectors from collection: {}", collection_name);

    // Parse query parameters for pagination - cap at 50 for vector browser
    let limit = params.limit.unwrap_or(10).min(50);
    let offset = params.offset.unwrap_or(0);
    let min_score = params.min_score.unwrap_or(0.0).max(0.0).min(1.0);

    // Check if collection exists (for validation)
    if !collection_exists(&state, &collection_name).await {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Collection '{}' not found", collection_name),
                code: "COLLECTION_NOT_FOUND".to_string(),
                details: None,
            }),
        ));
    }

    // Try to get collection info from GRPC first to check if it exists and get stats
    let collection_info = if let Some(ref mut grpc_client) = state.grpc_client {
        match grpc_client.list_collections().await {
            Ok(response) => {
                response.collections.into_iter()
                    .find(|c| c.name == collection_name)
            }
            Err(_) => None,
        }
    } else {
        None
    };

    // Check if vectors are available locally first
    match state.store.get_collection(&collection_name) {
        Ok(collection) => {
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
            let paginated_vectors: Vec<VectorResponse> = filtered_vectors
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|v| VectorResponse {
                    id: v.id,
                    payload: v.payload.map(|p| p.data),
                })
                .collect();

            let paginated_count = paginated_vectors.len();

            let response = ListVectorsResponse {
                vectors: paginated_vectors,
                total: if min_score > 0.0 { filtered_total } else { total_count },
                limit,
                offset,
                message: if min_score > 0.0 && filtered_total != total_count {
                    Some(format!("Filtered {} of {} vectors by min_score >= {:.2}. Showing {} of {} filtered vectors.",
                        filtered_total, total_count, min_score, paginated_count, filtered_total))
                } else if total_count > limit {
                    Some(format!("Showing {} of {} vectors. Use pagination for more.", limit.min(total_count), total_count))
                } else {
                    None
                },
            };

            let duration = start_time.elapsed();
            info!(
                "Listed {} vectors from local collection '{}' (total: {}) in {:?}",
                response.vectors.len(),
                collection_name,
                total_count,
                duration
            );

            Ok(Json(response))
        }
        Err(_) => {
            if let Some(ref mut grpc_client) = state.grpc_client {
                match grpc_client.search(collection_name.clone(), "document".to_string(), limit.min(10) as i32).await {
                    Ok(grpc_response) => {
                        let sample_vectors: Vec<VectorResponse> = grpc_response.results
                            .into_iter()
                            .filter(|r| r.score >= min_score)
                            .take(limit)
                            .map(|r| VectorResponse {
                                id: r.id,
                                payload: Some(serde_json::json!({
                                    "content": r.content,
                                    "metadata": r.metadata,
                                    "score": r.score,
                                    "note": "Sample vector from semantic search"
                                })),
                            })
                            .collect();

                        let total_count = collection_info
                            .map(|info| info.vector_count as usize)
                            .unwrap_or(grpc_response.total_found as usize);

                        let message = if sample_vectors.is_empty() {
                            if min_score > 0.0 {
                                Some(format!("No sample vectors found with score >= {:.2}. Try lowering min_score or use semantic search (/search/text) for specific queries.", min_score))
                            } else {
                                Some("No sample vectors found. Use semantic search (/search/text) for specific queries.".to_string())
                            }
                        } else {
                            if min_score > 0.0 {
                                Some(format!(
                                    "Showing {} sample vectors (filtered by score >= {:.2}) from semantic search. Total collection: {} vectors. Use semantic search (/search/text) for specific queries.",
                                    sample_vectors.len(),
                                    min_score,
                                    total_count
                                ))
                            } else {
                                Some(format!(
                                    "Showing {} sample vectors from semantic search. Total collection: {} vectors. Use semantic search (/search/text) for specific queries.",
                                    sample_vectors.len(),
                                    total_count
                                ))
                            }
                        };

                        let response = ListVectorsResponse {
                            vectors: sample_vectors,
                            total: total_count,
                            limit,
                            offset,
                            message,
                        };

                        let duration = start_time.elapsed();
                        info!(
                            "Collection '{}' returned {} sample vectors via GRPC search in {:?}",
                            collection_name,
                            response.vectors.len(),
                            duration
                        );

                        return Ok(Json(response));
                    }
                    Err(e) => {
                        warn!("Failed to get sample vectors via GRPC search: {}", e);
                    }
                }
            }

            // Fallback: return info about available vectors
            let total_count = collection_info
                .map(|info| info.vector_count as usize)
                .unwrap_or(0);

            let message = if min_score > 0.0 {
                Some(format!(
                    "Collection has {} vectors available via semantic search (/search/text). Score filtering (min_score={:.2}) is not available when vectors are not cached locally.",
                    total_count, min_score
                ))
            } else if total_count > 0 {
                Some(format!(
                    "Collection has {} vectors available via semantic search (/search/text). Vectors are not cached locally for direct browsing.",
                    total_count
                ))
            } else {
                Some("Vectors are not cached locally. Use semantic search (/search/text) to access vector content.".to_string())
            };

            let response = ListVectorsResponse {
                vectors: vec![],
                total: total_count,
                limit,
                offset,
                message,
            };

            let duration = start_time.elapsed();
            info!(
                "Collection '{}' has {} vectors via GRPC. No sample vectors available. Request completed in {:?}",
                collection_name,
                total_count,
                duration
            );

            Ok(Json(response))
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
    if state
        .store
        .get_collection_metadata(&collection_name)
        .is_err()
    {
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
    if state
        .store
        .get_collection_metadata(&collection_name)
        .is_err()
    {
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
    let mut file_extensions: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();

    for vector in all_vectors {
        if let Some(payload) = &vector.payload {
            if let Some(file_path) = payload.data.get("file_path") {
                if let Some(file_path_str) = file_path.as_str() {
                    *file_map.entry(file_path_str.to_string()).or_insert(0) += 1;

                    // Extract file extension
                    if let Some(extension) = std::path::Path::new(file_path_str).extension() {
                        file_extensions.insert(
                            file_path_str.to_string(),
                            Some(extension.to_string_lossy().to_string()),
                        );
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
            file.extension
                .as_ref()
                .map_or(false, |ext| ext == extension_filter)
        });
    }

    // Sort by file path for consistent ordering
    files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    let total = files.len();
    let files: Vec<super::types::FileInfo> = files.into_iter().skip(offset).take(limit).collect();

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

    let request_id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let params = request
        .get("params")
        .and_then(|p| p.as_object())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32600,
                        "message": "Invalid Request",
                        "data": "Missing params"
                    }
                })),
            )
        })?;

    let tool_name = params.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request",
                    "data": "Missing tool name"
                }
            })),
        )
    })?;

    let empty_map = serde_json::Map::new();
    let arguments = params
        .get("arguments")
        .and_then(|a| a.as_object())
        .unwrap_or(&empty_map);

    let result = match tool_name {
        "search_vectors" => {
            let collection = arguments
                .get("collection")
                .and_then(|c| c.as_str())
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {
                                "code": -32602,
                                "message": "Invalid params",
                                "data": "Missing collection parameter"
                            }
                        })),
                    )
                })?;

            let query = arguments
                .get("query")
                .and_then(|q| q.as_str())
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {
                                "code": -32602,
                                "message": "Invalid params",
                                "data": "Missing query parameter"
                            }
                        })),
                    )
                })?;

            let limit = arguments
                .get("limit")
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
                }),
            )
            .await
            {
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
                        })),
                    ));
                }
            }
        }

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
        }

        "embed_text" => {
            let text = arguments
                .get("text")
                .and_then(|t| t.as_str())
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {
                                "code": -32602,
                                "message": "Invalid params",
                                "data": "Missing text parameter"
                            }
                        })),
                    )
                })?;

            // Generate embedding directly using the embedding manager
            match state
                .embedding_manager
                .lock()
                .unwrap()
                .embed_with_provider("bm25", text)
            {
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
                }
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "error": {
                                "code": -32603,
                                "message": "Internal error",
                                "data": format!("Embedding failed: {}", e)
                            }
                        })),
                    ));
                }
            }
        }

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
                })),
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

    let request_id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

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
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

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

        let _ =
            tx_clone.send(axum::response::sse::Event::default().data(tools_message.to_string()));
    });

    // Create stream from receiver
    let stream = futures_util::StreamExt::map(UnboundedReceiverStream::new(rx), |event| Ok(event));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("keepalive"),
    )
}

/// MCP HTTP endpoint for tool calls (fallback for SSE)
pub async fn mcp_http_tools_call(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    debug!("MCP HTTP tools/call requested: {:?}", request);

    // Extract the method and params from the request
    let method = request["method"].as_str().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing method"})),
        )
    })?;

    let params = request["params"].as_object().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing params"})),
        )
    })?;

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
                Ok(Json(result)) => {
                    serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "result": result})
                }
                Err((status, Json(error))) => {
                    return Err((
                        status,
                        Json(
                            serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": error}),
                        ),
                    ));
                }
            }
        }
        "tools/list" => match mcp_tools_list(State(state)).await {
            Ok(Json(result)) => {
                serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "result": result})
            }
            Err((status, Json(error))) => {
                return Err((
                    status,
                    Json(
                        serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": error}),
                    ),
                ));
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
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": -32601, "message": format!("Unknown method: {}", method)}}),
                ),
            ));
        }
    };

    Ok(Json(result))
}

/// Update indexing progress for a collection via HTTP POST
pub async fn update_indexing_progress(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let collection_name = request["collection"].as_str().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing collection name"})),
        )
    })?;

    let status = request["status"].as_str().unwrap_or("processing");
    let progress = request["progress"].as_f64().unwrap_or(0.0) as f32;
    let total_documents = request["total_documents"].as_u64().unwrap_or(100) as usize;
    let processed_documents = request["processed_documents"].as_u64().unwrap_or(0) as usize;
    let vector_count = request["vector_count"].as_u64().unwrap_or(0) as usize;

    // Update the progress state
    update_indexing_progress_internal(
        &state.indexing_progress,
        collection_name,
        status,
        progress,
        total_documents,
        processed_documents,
        vector_count,
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "collection": collection_name,
        "status": status,
        "progress": progress
    })))
}

/// Update indexing progress for a collection (internal function)
pub fn update_indexing_progress_internal(
    progress_state: &IndexingProgressState,
    collection_name: &str,
    status: &str,
    progress: f32,
    total_documents: usize,
    processed_documents: usize,
    vector_count: usize,
) {
    let mut state = progress_state.map.lock().unwrap();
    
    if let Some(collection_progress) = state.get_mut(collection_name) {
        collection_progress.status = status.to_string();
        collection_progress.progress = progress;
        collection_progress.total_documents = total_documents;
        collection_progress.processed_documents = processed_documents;
        collection_progress.vector_count = vector_count;
        collection_progress.last_updated = chrono::Utc::now().to_string();
        
        // Calculate estimated time remaining
        if progress > 0.0 && processed_documents > 0 {
            let total_time_estimated = (processed_documents as f32 / progress) * 100.0;
            let remaining_time = total_time_estimated - processed_documents as f32;
            collection_progress.estimated_time_remaining = Some(remaining_time as u64);
        }
    }
}

/// Batch insert vectors into a collection
pub async fn batch_insert_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<BatchInsertRequest>,
) -> Result<Json<BatchResponse>, (StatusCode, Json<serde_json::Value>)> {
    let start_time = Instant::now();

    // Create batch processor
    let batch_processor = BatchProcessorBuilder::new(
        Arc::clone(&state.store),
        Arc::clone(&state.embedding_manager),
    )
    .with_config(convert_batch_config(request.config))
    .build();

    // Convert request vectors to batch operation with embedding generation
    let mut vectors = Vec::new();

    for v in request.vectors {
        // Generate embedding if not provided
        let embedding_data = if let Some(data) = v.data {
            data
        } else {
            // Generate embedding from content
            let manager = state.embedding_manager.lock().unwrap();
            manager.embed(&v.content)
                .map_err(|e| {
                    error!("Failed to generate embedding for vector {}: {}", v.id, e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Embedding generation failed",
                            "vector_id": v.id,
                            "message": e.to_string()
                        }))
                    )
                })?
        };

        // Create rich payload with content and metadata
        let mut payload_data = serde_json::Map::new();
        payload_data.insert(
            "content".to_string(),
            serde_json::Value::String(v.content.clone()),
        );

        // Add custom metadata if provided
        if let Some(metadata) = v.metadata {
            if let serde_json::Value::Object(meta_obj) = metadata {
                for (key, value) in meta_obj {
                    payload_data.insert(key, value);
                }
            }
        }

        // Add batch operation metadata
        payload_data.insert(
            "operation_type".to_string(),
            serde_json::Value::String("batch_insert".to_string()),
        );
        payload_data.insert(
            "created_at".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );
        payload_data.insert(
            "batch_id".to_string(),
            serde_json::Value::String(format!("batch_{}", uuid::Uuid::new_v4())),
        );

        let payload = Payload {
            data: serde_json::Value::Object(payload_data),
        };

        vectors.push(Vector::with_payload(v.id, embedding_data, payload));
    }

    let operation = BatchOperation::Insert {
        vectors,
        atomic: request.atomic.unwrap_or(true),
    };

    // Execute batch operation
    match batch_processor.execute_operation(collection_name.clone(), operation).await {
        Ok(result) => {
            let response = BatchResponse {
                success: true,
                collection: collection_name,
                operation: "insert".to_string(),
                total_operations: result.total_operations,
                successful_operations: result.successful_count,
                failed_operations: result.failed_count,
                duration_ms: start_time.elapsed().as_millis() as u64,
                errors: result.failed_operations.into_iter().map(|e| e.error_message).collect(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Batch insert failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Batch insert failed",
                    "message": e.to_string()
                }))
            ))
        }
    }
}

/// Batch update vectors in a collection
pub async fn batch_update_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<BatchUpdateRequest>,
) -> Result<Json<BatchResponse>, (StatusCode, Json<serde_json::Value>)> {
    let start_time = Instant::now();

    // Create batch processor
    let batch_processor = BatchProcessorBuilder::new(
        Arc::clone(&state.store),
        Arc::clone(&state.embedding_manager),
    )
    .with_config(convert_batch_config(request.config))
    .build();

    // Convert request updates to batch operation
    let updates: Vec<crate::batch::VectorUpdate> = request.updates.into_iter()
        .map(|u| crate::batch::VectorUpdate {
            id: u.id,
            data: u.data,
            metadata: u.metadata,
        })
        .collect();

    let operation = BatchOperation::Update {
        updates,
        atomic: request.atomic.unwrap_or(true),
    };

    // Execute batch operation
    match batch_processor.execute_operation(collection_name.clone(), operation).await {
        Ok(result) => {
            let response = BatchResponse {
                success: true,
                collection: collection_name,
                operation: "update".to_string(),
                total_operations: result.total_operations,
                successful_operations: result.successful_count,
                failed_operations: result.failed_count,
                duration_ms: start_time.elapsed().as_millis() as u64,
                errors: result.failed_operations.into_iter().map(|e| e.error_message).collect(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Batch update failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Batch update failed",
                    "message": e.to_string()
                }))
            ))
        }
    }
}

/// Batch delete vectors from a collection
pub async fn batch_delete_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<BatchDeleteRequest>,
) -> Result<Json<BatchResponse>, (StatusCode, Json<serde_json::Value>)> {
    let start_time = Instant::now();

    // Create batch processor
    let batch_processor = BatchProcessorBuilder::new(
        Arc::clone(&state.store),
        Arc::clone(&state.embedding_manager),
    )
    .with_config(convert_batch_config(request.config))
    .build();

    let operation = BatchOperation::Delete {
        vector_ids: request.vector_ids,
        atomic: request.atomic.unwrap_or(true),
    };

    // Execute batch operation
    match batch_processor.execute_operation(collection_name.clone(), operation).await {
        Ok(result) => {
            let response = BatchResponse {
                success: true,
                collection: collection_name,
                operation: "delete".to_string(),
                total_operations: result.total_operations,
                successful_operations: result.successful_count,
                failed_operations: result.failed_count,
                duration_ms: start_time.elapsed().as_millis() as u64,
                errors: result.failed_operations.into_iter().map(|e| e.error_message).collect(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Batch delete failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Batch delete failed",
                    "message": e.to_string()
                }))
            ))
        }
    }
}

/// Batch search vectors in a collection
pub async fn batch_search_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(request): Json<BatchSearchRequest>,
) -> Result<Json<BatchSearchResponse>, (StatusCode, Json<serde_json::Value>)> {
    let start_time = Instant::now();

    // Create batch processor
    let batch_processor = BatchProcessorBuilder::new(
        Arc::clone(&state.store),
        Arc::clone(&state.embedding_manager),
    )
    .with_config(convert_batch_config(request.config))
    .build();

    // Convert search queries
    let queries: Vec<crate::batch::SearchQuery> = request.queries.into_iter()
        .map(|q| crate::batch::SearchQuery {
            query_vector: q.query_vector,
            query_text: q.query_text,
            limit: q.limit as i32,
            threshold: q.threshold,
            filters: q.filters.unwrap_or_default(),
        })
        .collect();

    let operation = BatchOperation::Search {
        queries,
        atomic: request.atomic.unwrap_or(true),
    };

    // Execute batch operation
    match batch_processor.execute_operation(collection_name.clone(), operation).await {
        Ok(result) => {
            let response = BatchSearchResponse {
                success: true,
                collection: collection_name,
                total_queries: result.total_operations,
                successful_queries: result.successful_count,
                failed_queries: result.failed_count,
                duration_ms: start_time.elapsed().as_millis() as u64,
                results: vec![], // TODO: Implement proper search result handling
                errors: result.failed_operations.into_iter().map(|e| e.error_message).collect(),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Batch search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Batch search failed",
                    "message": e.to_string()
                }))
            ))
        }
    }
}
