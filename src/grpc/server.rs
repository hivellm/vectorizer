use crate::VectorStore;
use tracing::{info, warn};
use crate::models::QuantizationConfig;
use crate::embedding::EmbeddingManager;
use crate::config::GrpcServerConfig;
use crate::api::handlers::WorkspaceCollection;
use crate::workspace::WorkspaceManager;
use crate::summarization::{SummarizationManager, SummarizationConfig, SummarizationMethod};
use crate::grpc::vectorizer::{
    vectorizer_service_server::VectorizerService,
    SearchRequest, SearchResponse, SearchResult,
    MemoryAnalysisResponse, CollectionMemoryInfo, MemoryInfo, QuantizationInfo, PerformanceInfo, MemorySummary, QuantizationSummary,
    RequantizeCollectionRequest, RequantizeCollectionResponse,
    Empty, ListCollectionsResponse, CollectionInfo,
    EmbedRequest, EmbedResponse,
    IndexingProgressResponse, IndexingStatus,
    UpdateIndexingProgressRequest,
    HealthResponse,
    CreateCollectionRequest, CreateCollectionResponse,
    DeleteCollectionRequest, DeleteCollectionResponse,
    InsertTextsRequest, InsertTextsResponse,
    DeleteVectorsRequest, DeleteVectorsResponse,
    GetVectorRequest, GetVectorResponse,
    TextData,
    // Sumarização
    SummarizeTextRequest, SummarizeTextResponse,
    SummarizeContextRequest, SummarizeContextResponse,
    GetSummaryRequest, GetSummaryResponse,
    ListSummariesRequest, ListSummariesResponse,
    SummaryInfo,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use chrono::Utc;

pub struct VectorizerGrpcService {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    indexing_progress: Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>>,
    summarization_manager: Option<Arc<Mutex<SummarizationManager>>>,
}

impl VectorizerGrpcService {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<Mutex<EmbeddingManager>>,
        indexing_progress: Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>>,
        summarization_config: Option<SummarizationConfig>,
    ) -> Self {
        let summarization_manager = summarization_config.map(|config| {
            Arc::new(Mutex::new(
                SummarizationManager::new(config).unwrap_or_else(|_| SummarizationManager::with_default_config())
            ))
        });
        
        Self {
            vector_store,
            embedding_manager,
            indexing_progress,
            summarization_manager,
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
        let results = self.vector_store
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
        for workspace_collection in &workspace_collections {
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

        // Add dynamic collections (like summary collections) that are not in workspace config
        for collection_name in indexed_collections {
            // Skip if already added from workspace collections
            if workspace_collections.iter().any(|wc| wc.name == collection_name) {
                continue;
            }
            
            // Add dynamic collection
            match self.vector_store.get_collection_metadata(&collection_name) {
                Ok(metadata) => {
                    collection_infos.push(CollectionInfo {
                        name: collection_name.clone(),
                        vector_count: metadata.vector_count as i32,
                        document_count: metadata.document_count as i32,
                        dimension: metadata.config.dimension as i32,
                        similarity_metric: "cosine".to_string(),
                        status: "ready".to_string(),
                        last_updated: Utc::now().to_rfc3339(),
                        error_message: None,
                    });
                }
                Err(_) => {
                    // Skip collections we can't get metadata for
                    tracing::warn!("Failed to get metadata for dynamic collection: {}", collection_name);
                }
            }
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
            quantization: QuantizationConfig::SQ { bits: 8 },
            compression: crate::models::CompressionConfig::default(),
        };

        self.vector_store.create_collection_with_quantization(&req.name, config)
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

    async fn insert_texts(&self, request: Request<InsertTextsRequest>) -> Result<Response<InsertTextsResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC InsertVectors request: collection={}, texts_count={}, provider={}", 
                       req.collection, req.texts.len(), req.provider);

        use crate::models::Vector;

        let mut vector_objects = Vec::new();
        let mut errors = Vec::new();
        
        for text_data in req.texts {
            // Gerar embedding do texto
            let embedding = match self
                .embedding_manager
                .lock()
                .await
                .embed_with_provider(&req.provider, &text_data.text)
            {
                Ok(embedding) => embedding,
                Err(e) => {
                    errors.push(format!("Failed to generate embedding for {}: {}", text_data.id, e));
                    continue;
                }
            };

            // Create payload with metadata and content (like document_loader.rs)
            let mut payload_data = std::collections::HashMap::new();
            
            // Add user-provided metadata
            for (key, value) in text_data.metadata {
                payload_data.insert(key, serde_json::Value::String(value));
            }
            
            // Add content to payload (like document_loader does)
            payload_data.insert(
                "content".to_string(),
                serde_json::Value::String(text_data.text.clone()),
            );
            
            // Add system metadata
            payload_data.insert(
                "source".to_string(),
                serde_json::Value::String("grpc_insert".to_string()),
            );
            payload_data.insert(
                "provider".to_string(),
                serde_json::Value::String(req.provider.clone()),
            );
            payload_data.insert(
                "created_at".to_string(),
                serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
            );
            
            let payload = crate::models::Payload::new(serde_json::Value::Object(
                payload_data.into_iter().collect(),
            ));
            
            vector_objects.push(Vector::with_payload(text_data.id, embedding, payload));
        }

        let inserted_count = vector_objects.len();
        
        if !vector_objects.is_empty() {
            self.vector_store.insert(&req.collection, vector_objects)
                .map_err(|e| Status::internal(format!("Failed to insert vectors: {}", e)))?;
        }

        let status = if errors.is_empty() { "success" } else if inserted_count > 0 { "partial_success" } else { "error" };
        let message = if errors.is_empty() {
            format!("Successfully inserted {} vectors", inserted_count)
        } else {
            format!("Inserted {} vectors, {} errors: {}", inserted_count, errors.len(), errors.join(", "))
        };

        Ok(Response::new(InsertTextsResponse {
            collection: req.collection,
            inserted_count: inserted_count as i32,
            status: status.to_string(),
            message,
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

    async fn summarize_text(&self, request: Request<SummarizeTextRequest>) -> Result<Response<SummarizeTextResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC SummarizeText request: method={}, text_length={}", 
                       req.method, req.text.len());

        // Check if summarization is enabled
        let summarization_manager = match &self.summarization_manager {
            Some(manager) => manager,
            None => {
                return Err(Status::unavailable("Summarization service is disabled"));
            }
        };

        // Parse method
        let method = req.method.parse::<SummarizationMethod>()
            .map_err(|e| Status::invalid_argument(format!("Invalid summarization method: {}", e)))?;

        // Create parameters
        let params = crate::summarization::types::SummarizationParams {
            text: req.text.clone(),
            method,
            max_length: req.max_length.map(|l| l as usize),
            compression_ratio: req.compression_ratio,
            language: req.language,
            metadata: req.metadata,
        };

        // Perform summarization
        let mut summarization_manager = summarization_manager.lock().await;
        let result = summarization_manager.summarize_text(params)
            .map_err(|e| Status::internal(format!("Summarization failed: {}", e)))?;

        Ok(Response::new(SummarizeTextResponse {
            summary_id: result.summary_id,
            original_text: result.original_text,
            summary: result.summary,
            method: result.method.to_string(),
            original_length: result.original_length as i32,
            summary_length: result.summary_length as i32,
            compression_ratio: result.compression_ratio,
            language: result.language,
            status: "success".to_string(),
            message: "Text summarized successfully".to_string(),
            metadata: result.metadata,
        }))
    }

    async fn summarize_context(&self, request: Request<SummarizeContextRequest>) -> Result<Response<SummarizeContextResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC SummarizeContext request: method={}, context_length={}", 
                       req.method, req.context.len());

        // Check if summarization is enabled
        let summarization_manager = match &self.summarization_manager {
            Some(manager) => manager,
            None => {
                return Err(Status::unavailable("Summarization service is disabled"));
            }
        };

        // Parse method
        let method = req.method.parse::<SummarizationMethod>()
            .map_err(|e| Status::invalid_argument(format!("Invalid summarization method: {}", e)))?;

        // Create parameters
        let params = crate::summarization::types::ContextSummarizationParams {
            context: req.context.clone(),
            method,
            max_length: req.max_length.map(|l| l as usize),
            compression_ratio: req.compression_ratio,
            language: req.language,
            metadata: req.metadata,
        };

        // Perform summarization
        let mut summarization_manager = summarization_manager.lock().await;
        let result = summarization_manager.summarize_context(params)
            .map_err(|e| Status::internal(format!("Context summarization failed: {}", e)))?;

        Ok(Response::new(SummarizeContextResponse {
            summary_id: result.summary_id,
            original_context: result.original_text,
            summary: result.summary,
            method: result.method.to_string(),
            original_length: result.original_length as i32,
            summary_length: result.summary_length as i32,
            compression_ratio: result.compression_ratio,
            language: result.language,
            status: "success".to_string(),
            message: "Context summarized successfully".to_string(),
            metadata: result.metadata,
        }))
    }

    async fn get_summary(&self, request: Request<GetSummaryRequest>) -> Result<Response<GetSummaryResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC GetSummary request: summary_id={}", req.summary_id);

        // Check if summarization is enabled
        let summarization_manager = match &self.summarization_manager {
            Some(manager) => manager,
            None => {
                return Err(Status::unavailable("Summarization service is disabled"));
            }
        };

        let summarization_manager = summarization_manager.lock().await;
        let summary = summarization_manager.get_summary(&req.summary_id)
            .ok_or_else(|| Status::not_found(format!("Summary not found: {}", req.summary_id)))?;

        Ok(Response::new(GetSummaryResponse {
            summary_id: summary.summary_id.clone(),
            original_text: summary.original_text.clone(),
            summary: summary.summary.clone(),
            method: summary.method.to_string(),
            original_length: summary.original_length as i32,
            summary_length: summary.summary_length as i32,
            compression_ratio: summary.compression_ratio,
            language: summary.language.clone(),
            created_at: summary.created_at.to_rfc3339(),
            metadata: summary.metadata.clone(),
            status: "found".to_string(),
        }))
    }

    async fn list_summaries(&self, request: Request<ListSummariesRequest>) -> Result<Response<ListSummariesResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!("GRPC ListSummaries request: method={:?}, language={:?}, limit={:?}", 
                       req.method, req.language, req.limit);

        // Check if summarization is enabled
        let summarization_manager = match &self.summarization_manager {
            Some(manager) => manager,
            None => {
                return Err(Status::unavailable("Summarization service is disabled"));
            }
        };

        let summarization_manager = summarization_manager.lock().await;
        let summaries = summarization_manager.list_summaries(
            req.method.as_deref(),
            req.language.as_deref(),
            req.limit.map(|l| l as usize),
            req.offset.map(|o| o as usize),
        );

        let summary_infos: Vec<SummaryInfo> = summaries
            .into_iter()
            .map(|info| SummaryInfo {
                summary_id: info.summary_id,
                method: info.method.to_string(),
                language: info.language,
                original_length: info.original_length as i32,
                summary_length: info.summary_length as i32,
                compression_ratio: info.compression_ratio,
                created_at: info.created_at.to_rfc3339(),
                metadata: info.metadata,
            })
            .collect();

        Ok(Response::new(ListSummariesResponse {
            summaries: summary_infos,
            total_count: summarization_manager.summaries.len() as i32,
            status: "success".to_string(),
        }))
    }

    /// Get detailed memory analysis for all collections
    async fn get_memory_analysis(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<MemoryAnalysisResponse>, Status> {
        info!("Generating detailed memory analysis via GRPC");

        // Get collections list
        let collections_response = self.list_collections(Request::new(Empty {})).await?;
        let collections_data = collections_response.into_inner();

        let mut collections = Vec::new();
        let mut total_theoretical_memory = 0i64;
        let mut total_actual_memory = 0i64;
        let mut collections_with_quantization = 0;
        let mut collections_without_quantization = 0;

        // Analyze each collection individually
        for collection_info in &collections_data.collections {
            let vector_count = collection_info.vector_count as usize;
            let dimension = collection_info.dimension as usize;

            // Calculate theoretical memory usage (f32 vectors)
            let theoretical_memory = (vector_count * dimension * 4) as i64; // 4 bytes per f32

            // Try to get actual memory usage from collection
            let actual_memory = match self.vector_store.get_collection(&collection_info.name) {
                Ok(collection_ref) => {
                    let estimated_memory = (*collection_ref).estimated_memory_usage() as i64;
                    let quantization_enabled = matches!((*collection_ref).config().quantization, crate::models::QuantizationConfig::SQ { bits: 8 });

                    if quantization_enabled {
                        collections_with_quantization += 1;
                    } else {
                        collections_without_quantization += 1;
                    }

                    (estimated_memory, quantization_enabled)
                },
                Err(_) => {
                    // Fallback to theoretical if can't access collection
                    (theoretical_memory, false)
                }
            };

            let (actual_memory_bytes, quantization_enabled) = actual_memory;
            let compression_ratio = if theoretical_memory > 0 {
                actual_memory_bytes as f64 / theoretical_memory as f64
            } else { 1.0 };

            let memory_savings_percent = if theoretical_memory > 0 {
                (1.0 - compression_ratio) * 100.0
            } else { 0.0 };

            let quantization_status = if quantization_enabled {
                if compression_ratio < 0.3 { "4x compression (SQ-8bit)" }
                else if compression_ratio < 0.6 { "2x compression" }
                else if compression_ratio < 0.8 { "Partial compression" }
                else { "Quantization enabled but not effective" }
            } else {
                "No quantization"
            };

            let collection_memory_info = CollectionMemoryInfo {
                name: collection_info.name.clone(),
                dimension: collection_info.dimension,
                vector_count: collection_info.vector_count,
                document_count: collection_info.document_count,
                embedding_provider: "bm25".to_string(), // Default
                metric: collection_info.similarity_metric.clone(),
                created_at: collection_info.last_updated.clone(),
                updated_at: collection_info.last_updated.clone(),
                indexing_status: Some(crate::grpc::vectorizer::IndexingStatus {
                    collection_name: collection_info.name.clone(),
                    status: collection_info.status.clone(),
                    progress: 100.0, // Assume complete for now
                    vector_count: collection_info.vector_count,
                    error_message: collection_info.error_message.clone(),
                    last_updated: collection_info.last_updated.clone(),
                }),
                memory_analysis: Some(MemoryInfo {
                    theoretical_memory_bytes: theoretical_memory,
                    theoretical_memory_mb: theoretical_memory as f64 / (1024.0 * 1024.0),
                    actual_memory_bytes: actual_memory_bytes,
                    actual_memory_mb: actual_memory_bytes as f64 / (1024.0 * 1024.0),
                    memory_saved_bytes: theoretical_memory.saturating_sub(actual_memory_bytes),
                    memory_saved_mb: (theoretical_memory.saturating_sub(actual_memory_bytes)) as f64 / (1024.0 * 1024.0),
                    compression_ratio,
                    memory_savings_percent,
                    memory_per_vector_bytes: if vector_count > 0 { actual_memory_bytes / vector_count as i64 } else { 0 },
                    theoretical_memory_per_vector_bytes: (dimension * 4) as i64,
                }),
                quantization: Some(QuantizationInfo {
                    enabled: quantization_enabled,
                    status: quantization_status.to_string(),
                    effective: compression_ratio < 0.8,
                    compression_factor: if compression_ratio > 0.0 { 1.0 / compression_ratio } else { 1.0 },
                }),
                performance: Some(PerformanceInfo {
                    memory_efficiency: if compression_ratio < 0.3 { "Excellent" }
                    else if compression_ratio < 0.6 { "Good" }
                    else if compression_ratio < 0.8 { "Fair" }
                    else { "Poor" }.to_string(),
                    recommendation: if quantization_enabled && compression_ratio >= 0.8 {
                        "Quantization enabled but not working - check implementation"
                    } else if !quantization_enabled && vector_count > 1000 {
                        "Enable quantization for memory savings"
                    } else if quantization_enabled && compression_ratio < 0.3 {
                        "Excellent quantization performance"
                    } else {
                        "No action needed"
                    }.to_string(),
                }),
            };

            collections.push(collection_memory_info);

            total_theoretical_memory += theoretical_memory;
            total_actual_memory += actual_memory_bytes;
        }

        // Calculate overall summary
        let overall_compression_ratio = if total_theoretical_memory > 0 {
            total_actual_memory as f64 / total_theoretical_memory as f64
        } else { 1.0 };

        let overall_memory_savings = if total_theoretical_memory > 0 {
            (1.0 - overall_compression_ratio) * 100.0
        } else { 0.0 };

        let response = MemoryAnalysisResponse {
            timestamp: chrono::Utc::now().to_rfc3339(),
            collections,
            summary: Some(MemorySummary {
                total_collections: collections_data.collections.len() as i32,
                collections_with_quantization,
                collections_without_quantization,
                total_vectors: collections_data.collections.iter().map(|c| c.vector_count).sum(),
                total_documents: collections_data.collections.iter().map(|c| c.document_count).sum(),
                memory_analysis: Some(MemoryInfo {
                    theoretical_memory_bytes: total_theoretical_memory,
                    theoretical_memory_mb: total_theoretical_memory as f64 / (1024.0 * 1024.0),
                    actual_memory_bytes: total_actual_memory,
                    actual_memory_mb: total_actual_memory as f64 / (1024.0 * 1024.0),
                    memory_saved_bytes: total_theoretical_memory.saturating_sub(total_actual_memory),
                    memory_saved_mb: (total_theoretical_memory.saturating_sub(total_actual_memory)) as f64 / (1024.0 * 1024.0),
                    compression_ratio: overall_compression_ratio,
                    memory_savings_percent: overall_memory_savings,
                    memory_per_vector_bytes: if collections_data.collections.iter().map(|c| c.vector_count as usize).sum::<usize>() > 0 {
                        total_actual_memory / collections_data.collections.iter().map(|c| c.vector_count as i64).sum::<i64>()
                    } else { 0 },
                    theoretical_memory_per_vector_bytes: 0, // Not applicable for summary
                }),
                quantization_summary: Some(QuantizationSummary {
                    quantization_coverage_percent: if collections_data.collections.len() > 0 {
                        (collections_with_quantization as f64 / collections_data.collections.len() as f64) * 100.0
                    } else { 0.0 },
                    overall_quantization_status: if overall_compression_ratio < 0.3 { "4x compression achieved" }
                    else if overall_compression_ratio < 0.6 { "2x compression achieved" }
                    else if overall_compression_ratio < 0.8 { "Partial compression" }
                    else { "Quantization not effective" }.to_string(),
                    recommendation: if overall_compression_ratio >= 0.8 {
                        "Enable quantization on more collections for better memory efficiency"
                    } else if overall_compression_ratio < 0.3 {
                        "Excellent quantization performance across all collections"
                    } else {
                        "Good quantization performance"
                    }.to_string(),
                }),
            }),
        };

        info!("Detailed memory analysis complete via GRPC: {} collections analyzed, {}MB actual vs {}MB theoretical",
              collections_data.collections.len(),
              total_actual_memory as f64 / (1024.0 * 1024.0),
              total_theoretical_memory as f64 / (1024.0 * 1024.0));

        Ok(Response::new(response))
    }

    /// Requantize all vectors in a collection for memory optimization
    async fn requantize_collection(
        &self,
        request: Request<RequantizeCollectionRequest>,
    ) -> Result<Response<RequantizeCollectionResponse>, Status> {
        let req = request.into_inner();
        let collection_name = req.collection_name;

        info!("Requantizing collection '{}' via GRPC", collection_name);

        // Get the collection
        let collection = match self.vector_store.get_collection(&collection_name) {
            Ok(coll) => coll,
            Err(_) => {
                return Ok(Response::new(RequantizeCollectionResponse {
                    collection_name: collection_name.clone(),
                    success: false,
                    message: format!("Collection '{}' not found", collection_name),
                    status: "not_found".to_string(),
                }));
            }
        };

        // Check if quantization is enabled for this collection
        let quantization_enabled = matches!(collection.config().quantization, crate::models::QuantizationConfig::SQ { bits: 8 });

        if !quantization_enabled {
            return Ok(Response::new(RequantizeCollectionResponse {
                collection_name: collection_name.clone(),
                success: false,
                message: format!("Quantization not enabled for collection '{}'", collection_name),
                status: "quantization_disabled".to_string(),
            }));
        }

        // Perform requantization
        match collection.requantize_existing_vectors() {
            Ok(_) => {
                info!("✅ Successfully requantized collection '{}' via GRPC", collection_name);
                Ok(Response::new(RequantizeCollectionResponse {
                    collection_name,
                    success: true,
                    message: "Collection requantized successfully".to_string(),
                    status: "success".to_string(),
                }))
            },
            Err(e) => {
                warn!("⚠️ Failed to requantize collection '{}' via GRPC: {}", collection_name, e);
                Ok(Response::new(RequantizeCollectionResponse {
                    collection_name,
                    success: false,
                    message: format!("Failed to requantize collection: {}", e),
                    status: "error".to_string(),
                }))
            }
        }
    }
}

/// Start GRPC server with configuration
pub async fn start_grpc_server(
    config: GrpcServerConfig,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
    indexing_progress: Arc<Mutex<std::collections::HashMap<String, IndexingStatus>>>,
    summarization_config: Option<SummarizationConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled {
        tracing::info!("GRPC server is disabled in configuration");
        return Ok(());
    }

    let addr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| format!("Invalid GRPC server address: {}", e))?;

    let service = VectorizerGrpcService::new(vector_store, embedding_manager, indexing_progress, summarization_config);
    
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
