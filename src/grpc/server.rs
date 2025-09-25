use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::grpc::vectorizer::{
    vectorizer_service_server::VectorizerService,
    SearchRequest, SearchResponse, SearchResult,
    Empty, ListCollectionsResponse, CollectionInfo,
    EmbedRequest, EmbedResponse,
    IndexingProgressResponse, IndexingStatus,
    UpdateIndexingProgressRequest,
    HealthResponse,
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
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
            indexing_progress: Arc::new(Mutex::new(std::collections::HashMap::new())),
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
            .map(|result| SearchResult {
                id: result.id,
                content: "".to_string(), // TODO: Extract content from payload
                score: result.score,
                metadata: std::collections::HashMap::new(), // TODO: Extract metadata from payload
            })
            .collect();

        Ok(Response::new(SearchResponse {
            results: grpc_results,
            total_found: total_found as i32,
            search_time_ms: search_time,
        }))
    }

    async fn list_collections(&self, _request: Request<Empty>) -> Result<Response<ListCollectionsResponse>, Status> {
        let collections = self.vector_store.list_collections();
        
        let collection_infos: Vec<CollectionInfo> = collections
            .into_iter()
            .map(|name| {
                let metadata = self.vector_store.get_collection_metadata(&name).unwrap_or_else(|_| {
                    panic!("Collection {} should exist", name)
                });
                
                CollectionInfo {
                    name: name.clone(),
                    vector_count: metadata.vector_count as i32,
                    dimension: metadata.config.dimension as i32,
                    similarity_metric: "cosine".to_string(),
                    status: "ready".to_string(),
                    last_updated: Utc::now().to_rfc3339(),
                    error_message: None,
                }
            })
            .collect();

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
}
