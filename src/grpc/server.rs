use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::config::GrpcServerConfig;
use crate::api::handlers::WorkspaceCollection;
use crate::workspace::WorkspaceManager;
use crate::grpc::vectorizer::{
    vectorizer_service_server::VectorizerService,
    SearchRequest, SearchResponse, SearchResult,
    Empty, ListCollectionsResponse, CollectionInfo,
    EmbedRequest, EmbedResponse,
    IndexingProgressResponse, IndexingStatus,
    UpdateIndexingProgressRequest,
    HealthResponse,
    CreateCollectionRequest, CreateCollectionResponse,
    DeleteCollectionRequest, DeleteCollectionResponse,
    InsertVectorsRequest, InsertVectorsResponse,
    DeleteVectorsRequest, DeleteVectorsResponse,
    GetVectorRequest, GetVectorResponse,
    VectorData,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use chrono::Utc;

pub struct VectorizerGrpcService {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    indexing_progress: Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>>,
}

impl VectorizerGrpcService {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<Mutex<EmbeddingManager>>,
        indexing_progress: Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
            indexing_progress,
        }
    }

    pub fn get_indexing_progress(&self) -> Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>> {
        self.indexing_progress.clone()
    }
}

#[tonic::async_trait]
impl VectorizerService for VectorizerGrpcService {
    async fn search(&self, request: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC Search request: collection={}, query={}, limit={}", 
                       req.collection, req.query, req.limit);

        // Generate embedding
        let embedding = self
            .embedding_manager
            .lock()
            .await
            .embed_with_provider("bm25", &req.query)
            .map_err(|e| Status::internal(format!("Failed to generate embedding: {}", e)))?;

        // Search in vector store
        let start_time = std::time::Instant::now();
        let results = self
            .vector_store
            .search(&req.collection, &embedding, req.limit as usize)
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;
        
        let search_time = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to ms

        // Convert to GRPC response
        let total_found = results.len();
        let grpc_results: Vec<SearchResult> = results
            .into_iter()
            .map(|result| {
                let mut content = String::new();
                let mut metadata = std::collections::HashMap::new();
                
                // Extract content and metadata from payload
                if let Some(payload) = result.payload {
                    // Extract content from payload
                    if let Some(content_value) = payload.data.get("content") {
                        if let Some(content_str) = content_value.as_str() {
                            content = content_str.to_string();
                        }
                    }
                    
                    // Extract metadata from payload
                    if let Some(metadata_value) = payload.data.get("metadata") {
                        if let Some(metadata_obj) = metadata_value.as_object() {
                            for (key, value) in metadata_obj {
                                if let Some(str_value) = value.as_str() {
                                    metadata.insert(key.clone(), str_value.to_string());
                                } else {
                                    metadata.insert(key.clone(), value.to_string());
                                }
                            }
                        }
                    }
                    
                    // Also extract other fields as metadata
                    for (key, value) in payload.data.as_object().unwrap_or(&serde_json::Map::new()) {
                        if key != "content" && key != "metadata" {
                            if let Some(str_value) = value.as_str() {
                                metadata.insert(key.clone(), str_value.to_string());
                            } else {
                                metadata.insert(key.clone(), value.to_string());
                            }
                        }
                    }
                }
                
                SearchResult {
                    id: result.id,
                    content,
                    score: result.score,
                    metadata,
                }
            })
            .collect();

        Ok(Response::new(SearchResponse {
            results: grpc_results,
            total_found: total_found as i32,
            search_time_ms: search_time,
        }))
    }

    async fn list_collections(&self, _request: Request<Empty>) -> Result<Response<ListCollectionsResponse>, Status> {
        // Load workspace collections first
        let workspace_collections = match load_workspace_collections() {
            Ok(collections) => collections,
            Err(e) => {
                tracing::warn!("Failed to load workspace collections: {}", e);
                vec![]
            }
        };
        
        let indexed_collections = self.vector_store.list_collections();
        
        let mut collection_infos = Vec::new();
        
        // Add all workspace collections
        for workspace_collection in workspace_collections {
            let status = if indexed_collections.contains(&workspace_collection.name) {
                // Collection is indexed, get real data
                match self.vector_store.get_collection_metadata(&workspace_collection.name) {
                    Ok(metadata) => {
                        CollectionInfo {
                            name: workspace_collection.name.clone(),
                            vector_count: metadata.vector_count as i32,
                            document_count: metadata.document_count as i32,
                            dimension: metadata.config.dimension as i32,
                            similarity_metric: "cosine".to_string(),
                            status: "ready".to_string(),
                            last_updated: Utc::now().to_rfc3339(),
                            error_message: None,
                        }
                    }
                    Err(_) => {
                        CollectionInfo {
                            name: workspace_collection.name.clone(),
                            vector_count: 0,
                            document_count: 0,
                            dimension: workspace_collection.dimension as i32,
                            similarity_metric: "cosine".to_string(),
                            status: "error".to_string(),
                            last_updated: Utc::now().to_rfc3339(),
                            error_message: Some("Failed to get collection metadata".to_string()),
                        }
                    }
                }
            } else {
                // Collection not yet indexed
                let indexing_status = {
                    let indexing_progress_guard = self.indexing_progress.lock().await;
                    indexing_progress_guard.get(&workspace_collection.name).cloned()
                };
                let (status, progress) = match indexing_status {
                    Some(status) => (status.status.clone(), status.progress),
                    None => ("pending".to_string(), 0.0),
                };
                
                CollectionInfo {
                    name: workspace_collection.name.clone(),
                    vector_count: 0,
                    document_count: 0,
                    dimension: workspace_collection.dimension as i32,
                    similarity_metric: "cosine".to_string(),
                    status: format!("{}-{}", status, (progress * 100.0) as i32),
                    last_updated: Utc::now().to_rfc3339(),
                    error_message: None,
                }
            };
            
            collection_infos.push(status);
        }

        let total_collections = collection_infos.len();
        Ok(Response::new(ListCollectionsResponse {
            collections: collection_infos,
            total_collections: total_collections as i32,
        }))
    }

    async fn get_collection_info(&self, request: Request<crate::grpc::vectorizer::GetCollectionInfoRequest>) -> Result<Response<CollectionInfo>, Status> {
        let req = request.into_inner();
        
        let metadata = self.vector_store.get_collection_metadata(&req.collection_name)
            .map_err(|_| Status::not_found(format!("Collection {} not found", req.collection_name)))?;

        Ok(Response::new(CollectionInfo {
            name: req.collection_name,
            vector_count: metadata.vector_count as i32,
            document_count: metadata.document_count as i32,
            dimension: metadata.config.dimension as i32,
            similarity_metric: "cosine".to_string(),
            status: "ready".to_string(),
            last_updated: Utc::now().to_rfc3339(),
            error_message: None,
        }))
    }

    async fn embed_text(&self, request: Request<EmbedRequest>) -> Result<Response<EmbedResponse>, Status> {
        let req = request.into_inner();
        
        let embedding = self
            .embedding_manager
            .lock()
            .await
            .embed_with_provider(&req.provider, &req.text)
            .map_err(|e| Status::internal(format!("Failed to generate embedding: {}", e)))?;

        let dimension = embedding.len();
        Ok(Response::new(EmbedResponse {
            embedding,
            dimension: dimension as i32,
            provider: req.provider,
        }))
    }

    async fn get_indexing_progress(&self, _request: Request<Empty>) -> Result<Response<IndexingProgressResponse>, Status> {
        let progress = self.indexing_progress.lock().await;
        let collections: Vec<IndexingStatus> = progress.values().cloned().collect();
        
        let is_indexing = collections.iter().any(|c| c.status == "indexing");
        let has_error = collections.iter().any(|c| c.status == "error");
        let overall_status = if is_indexing {
            "indexing".to_string()
        } else if has_error {
            "error".to_string()
        } else {
            "completed".to_string()
        };
        
        Ok(Response::new(IndexingProgressResponse {
            collections,
            is_indexing,
            overall_status,
        }))
    }

    async fn update_indexing_progress(&self, request: Request<UpdateIndexingProgressRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        
        let status = IndexingStatus {
            collection_name: req.collection_name.clone(),
            status: req.status,
            progress: req.progress,
            vector_count: req.vector_count,
            error_message: req.error_message,
            last_updated: Utc::now().to_rfc3339(),
        };

        let mut progress = self.indexing_progress.lock().await;
        progress.insert(req.collection_name, status);

        Ok(Response::new(Empty {}))
    }

    async fn health_check(&self, _request: Request<Empty>) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "healthy".to_string(),
            service: "vectorizer-grpc".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Utc::now().to_rfc3339(),
            error_message: None,
        }))
    }

    async fn create_collection(&self, request: Request<CreateCollectionRequest>) -> Result<Response<CreateCollectionResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC CreateCollection request: name={}, dimension={}, metric={}", 
                       req.name, req.dimension, req.similarity_metric);

        use crate::models::{CollectionConfig, DistanceMetric};

        let distance_metric = match req.similarity_metric.to_lowercase().as_str() {
            "euclidean" => DistanceMetric::Euclidean,
            "cosine" => DistanceMetric::Cosine,
            "dot_product" => DistanceMetric::DotProduct,
            _ => DistanceMetric::Cosine, // Default
        };

        let config = CollectionConfig {
            dimension: req.dimension as usize,
            metric: distance_metric,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: None,
            compression: crate::models::CompressionConfig::default(),
        };

        self.vector_store.create_collection(&req.name, config)
            .map_err(|e| Status::internal(format!("Failed to create collection: {}", e)))?;

        Ok(Response::new(CreateCollectionResponse {
            name: req.name,
            dimension: req.dimension,
            similarity_metric: req.similarity_metric,
            status: "created".to_string(),
            message: "Collection created successfully".to_string(),
        }))
    }

    async fn delete_collection(&self, request: Request<DeleteCollectionRequest>) -> Result<Response<DeleteCollectionResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC DeleteCollection request: name={}", req.collection_name);

        self.vector_store.delete_collection(&req.collection_name)
            .map_err(|e| Status::internal(format!("Failed to delete collection: {}", e)))?;

        Ok(Response::new(DeleteCollectionResponse {
            collection_name: req.collection_name,
            status: "deleted".to_string(),
            message: "Collection deleted successfully".to_string(),
        }))
    }

    async fn insert_vectors(&self, request: Request<InsertVectorsRequest>) -> Result<Response<InsertVectorsResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC InsertVectors request: collection={}, vectors_count={}", 
                       req.collection, req.vectors.len());

        use crate::models::Vector;

        let vector_objects: Vec<Vector> = req.vectors
            .into_iter()
            .map(|vector_data| {
                // Convert metadata to JSON object
                let mut metadata_obj = serde_json::Map::new();
                for (key, value) in vector_data.metadata {
                    metadata_obj.insert(key, serde_json::Value::String(value));
                }
                
                let payload = crate::models::Payload::from_value(serde_json::Value::Object(metadata_obj))
                    .unwrap_or_default();
                
                Vector::with_payload(vector_data.id, vector_data.data, payload)
            })
            .collect();

        let inserted_count = vector_objects.len();
        self.vector_store.insert(&req.collection, vector_objects)
            .map_err(|e| Status::internal(format!("Failed to insert vectors: {}", e)))?;

        Ok(Response::new(InsertVectorsResponse {
            collection: req.collection,
            inserted_count: inserted_count as i32,
            status: "success".to_string(),
            message: format!("Successfully inserted {} vectors", inserted_count),
        }))
    }

    async fn delete_vectors(&self, request: Request<DeleteVectorsRequest>) -> Result<Response<DeleteVectorsResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC DeleteVectors request: collection={}, vector_ids_count={}", 
                       req.collection, req.vector_ids.len());

        let mut deleted_count = 0;
        let mut errors = Vec::new();

        for vector_id in req.vector_ids {
            match self.vector_store.delete(&req.collection, &vector_id) {
                Ok(_) => deleted_count += 1,
                Err(e) => {
                    errors.push(format!("Failed to delete {}: {}", vector_id, e));
                }
            }
        }

        let status = if errors.is_empty() { "success" } else { "partial_success" };
        let message = if errors.is_empty() {
            format!("Successfully deleted {} vectors", deleted_count)
        } else {
            format!("Deleted {} vectors, {} errors", deleted_count, errors.len())
        };

        Ok(Response::new(DeleteVectorsResponse {
            collection: req.collection,
            deleted_count: deleted_count as i32,
            status: status.to_string(),
            message,
        }))
    }

    async fn get_vector(&self, request: Request<GetVectorRequest>) -> Result<Response<GetVectorResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC GetVector request: collection={}, vector_id={}", 
                       req.collection, req.vector_id);

        let vector = self.vector_store.get_vector(&req.collection, &req.vector_id)
            .map_err(|e| Status::not_found(format!("Vector not found: {}", e)))?;

        let mut metadata = std::collections::HashMap::new();
        
        // Convert payload to metadata
        if let Some(payload) = vector.payload {
            if let Some(obj) = payload.data.as_object() {
                for (key, value) in obj {
                    if let Some(str_value) = value.as_str() {
                        metadata.insert(key.clone(), str_value.to_string());
                    } else {
                        metadata.insert(key.clone(), value.to_string());
                    }
                }
            }
        }

        Ok(Response::new(GetVectorResponse {
            id: vector.id,
            data: vector.data,
            metadata,
            collection: req.collection,
            status: "found".to_string(),
        }))
    }
}

/// Start GRPC server with configuration
pub async fn start_grpc_server(
    config: GrpcServerConfig,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    indexing_progress: Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled {
        tracing::info!("GRPC server is disabled in configuration");
        return Ok(());
    }

    let addr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| format!("Invalid GRPC server address: {}", e))?;

    let service = VectorizerGrpcService::new(vector_store, embedding_manager, indexing_progress);
    
    tracing::info!("🚀 Starting GRPC server on {}:{}", config.host, config.port);

    tonic::transport::Server::builder()
        .add_service(crate::grpc::vectorizer::vectorizer_service_server::VectorizerServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Load workspace collections from vectorize-workspace.yml
fn load_workspace_collections() -> Result<Vec<WorkspaceCollection>, Box<dyn std::error::Error>> {
    // Load workspace using the proper WorkspaceManager
    let workspace_manager = WorkspaceManager::load_from_file("vectorize-workspace.yml")?;
    let enabled_projects = workspace_manager.enabled_projects();
    
    let mut collections = Vec::new();
    
    for project in enabled_projects {
        for collection in &project.collections {
            collections.push(WorkspaceCollection {
                name: collection.name.clone(),
                description: collection.description.clone(),
                dimension: collection.dimension as u64,
                metric: format!("{:?}", collection.metric).to_lowercase(),
                model: format!("{:?}", collection.embedding.model).to_lowercase(),
            });
        }
    }
    
    Ok(collections)
}
