//! Simplified client for Vectorizer

use crate::error::{VectorizerError, Result};
use crate::models::*;
use reqwest::{Client, ClientBuilder, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json;

/// Simplified client for Vectorizer
pub struct VectorizerClient {
    http_client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl VectorizerClient {
    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a new client
    pub fn new_default() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|e| VectorizerError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client: client,
            base_url: "http://localhost:15002".to_string(),
            api_key: None,
        })
    }

    /// Create client with custom URL
    pub fn new_with_url(base_url: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|e| VectorizerError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client: client,
            base_url: base_url.to_string(),
            api_key: None,
        })
    }

    /// Create client with API key
    pub fn new_with_api_key(base_url: &str, api_key: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| VectorizerError::configuration(format!("Invalid API key: {}", e)))?);

        let client = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|e| VectorizerError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client: client,
            base_url: base_url.to_string(),
            api_key: Some(api_key.to_string()),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.make_request("GET", "/health", None).await?;
        let health: HealthStatus = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse health check response: {}", e)))?;
        Ok(health)
    }

    /// List collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self.make_request("GET", "/collections", None).await?;
        let collections_response: CollectionsResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse collections response: {}", e)))?;
        Ok(collections_response.collections)
    }

    /// Search vectors
    pub async fn search_vectors(
        &self,
        collection: &str,
        query: &str,
        limit: Option<usize>,
        score_threshold: Option<f32>,
    ) -> Result<SearchResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert("query".to_string(), serde_json::Value::String(query.to_string()));
        payload.insert("limit".to_string(), serde_json::Value::Number(limit.unwrap_or(10).into()));

        if let Some(threshold) = score_threshold {
            payload.insert("score_threshold".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(threshold as f64).unwrap()));
        }

        let response = self.make_request("POST", &format!("/collections/{}/search/text", collection), Some(serde_json::Value::Object(payload))).await?;
        let search_response: SearchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse search response: {}", e)))?;
        Ok(search_response)
    }

    // ===== INTELLIGENT SEARCH OPERATIONS =====

    /// Intelligent search with multi-query expansion and semantic reranking
    pub async fn intelligent_search(&self, request: IntelligentSearchRequest) -> Result<IntelligentSearchResponse> {
        let response = self.make_request("POST", "/intelligent_search", Some(serde_json::to_value(request).unwrap())).await?;
        let search_response: IntelligentSearchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse intelligent search response: {}", e)))?;
        Ok(search_response)
    }

    /// Semantic search with advanced reranking and similarity thresholds
    pub async fn semantic_search(&self, request: SemanticSearchRequest) -> Result<SemanticSearchResponse> {
        let response = self.make_request("POST", "/semantic_search", Some(serde_json::to_value(request).unwrap())).await?;
        let search_response: SemanticSearchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse semantic search response: {}", e)))?;
        Ok(search_response)
    }

    /// Context-aware search with metadata filtering and contextual reranking
    pub async fn contextual_search(&self, request: ContextualSearchRequest) -> Result<ContextualSearchResponse> {
        let response = self.make_request("POST", "/contextual_search", Some(serde_json::to_value(request).unwrap())).await?;
        let search_response: ContextualSearchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse contextual search response: {}", e)))?;
        Ok(search_response)
    }

    /// Multi-collection search with cross-collection reranking and aggregation
    pub async fn multi_collection_search(&self, request: MultiCollectionSearchRequest) -> Result<MultiCollectionSearchResponse> {
        let response = self.make_request("POST", "/multi_collection_search", Some(serde_json::to_value(request).unwrap())).await?;
        let search_response: MultiCollectionSearchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse multi-collection search response: {}", e)))?;
        Ok(search_response)
    }

    /// Create collection
    pub async fn create_collection(
        &self,
        name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<CollectionInfo> {
        let mut payload = serde_json::Map::new();
        payload.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        payload.insert("dimension".to_string(), serde_json::Value::Number(dimension.into()));
        payload.insert("metric".to_string(), serde_json::Value::String(format!("{:?}", metric.unwrap_or_default()).to_lowercase()));

        let response = self.make_request("POST", "/collections", Some(serde_json::Value::Object(payload))).await?;
        let create_response: CreateCollectionResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse create collection response: {}", e)))?;

        // Create a basic CollectionInfo from the response
        let info = CollectionInfo {
            name: create_response.collection,
            dimension: dimension,
            metric: format!("{:?}", metric.unwrap_or_default()).to_lowercase(),
            vector_count: 0,
            document_count: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            indexing_status: crate::models::IndexingStatus {
                status: "created".to_string(),
                progress: 0.0,
                total_documents: 0,
                processed_documents: 0,
                vector_count: 0,
                estimated_time_remaining: None,
                last_updated: "".to_string(),
            },
        };
        Ok(info)
    }

    /// Insert texts
    pub async fn insert_texts(
        &self,
        collection: &str,
        texts: Vec<BatchTextRequest>,
    ) -> Result<BatchResponse> {
        let payload = serde_json::json!({
            "texts": texts
        });

        let response = self.make_request("POST", &format!("/collections/{}/documents", collection), Some(serde_json::to_value(payload)?)).await?;
        let batch_response: BatchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse insert texts response: {}", e)))?;
        Ok(batch_response)
    }

    /// Delete collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/collections/{}", name), None).await?;
        Ok(())
    }

    /// Get vector
    pub async fn get_vector(&self, collection: &str, vector_id: &str) -> Result<Vector> {
        let response = self.make_request("GET", &format!("/collections/{}/vectors/{}", collection, vector_id), None).await?;
        let vector: Vector = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse get vector response: {}", e)))?;
        Ok(vector)
    }

    /// Get collection info
    pub async fn get_collection_info(&self, collection: &str) -> Result<CollectionInfo> {
        let response = self.make_request("GET", &format!("/collections/{}", collection), None).await?;
        let info: CollectionInfo = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse collection info: {}", e)))?;
        Ok(info)
    }

    /// Generate embeddings
    pub async fn embed_text(&self, text: &str, model: Option<&str>) -> Result<EmbeddingResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert("text".to_string(), serde_json::Value::String(text.to_string()));

        if let Some(model) = model {
            payload.insert("model".to_string(), serde_json::Value::String(model.to_string()));
        }

        let response = self.make_request("POST", "/embed", Some(serde_json::Value::Object(payload))).await?;
        let embedding_response: EmbeddingResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse embedding response: {}", e)))?;
        Ok(embedding_response)
    }

    // =============================================================================
    // DISCOVERY OPERATIONS
    // =============================================================================

    /// Complete discovery pipeline with intelligent search and prompt generation
    pub async fn discover(
        &self,
        query: &str,
        include_collections: Option<Vec<String>>,
        exclude_collections: Option<Vec<String>>,
        max_bullets: Option<usize>,
        broad_k: Option<usize>,
        focus_k: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("query".to_string(), serde_json::Value::String(query.to_string()));
        
        if let Some(inc) = include_collections {
            payload.insert("include_collections".to_string(), serde_json::to_value(inc).unwrap());
        }
        if let Some(exc) = exclude_collections {
            payload.insert("exclude_collections".to_string(), serde_json::to_value(exc).unwrap());
        }
        if let Some(max) = max_bullets {
            payload.insert("max_bullets".to_string(), serde_json::Value::Number(max.into()));
        }
        if let Some(k) = broad_k {
            payload.insert("broad_k".to_string(), serde_json::Value::Number(k.into()));
        }
        if let Some(k) = focus_k {
            payload.insert("focus_k".to_string(), serde_json::Value::Number(k.into()));
        }

        let response = self.make_request("POST", "/discover", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse discover response: {}", e)))?;
        Ok(result)
    }

    /// Pre-filter collections by name patterns
    pub async fn filter_collections(
        &self,
        query: &str,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("query".to_string(), serde_json::Value::String(query.to_string()));
        
        if let Some(inc) = include {
            payload.insert("include".to_string(), serde_json::to_value(inc).unwrap());
        }
        if let Some(exc) = exclude {
            payload.insert("exclude".to_string(), serde_json::to_value(exc).unwrap());
        }

        let response = self.make_request("POST", "/discovery/filter_collections", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse filter response: {}", e)))?;
        Ok(result)
    }

    /// Rank collections by relevance
    pub async fn score_collections(
        &self,
        query: &str,
        name_match_weight: Option<f32>,
        term_boost_weight: Option<f32>,
        signal_boost_weight: Option<f32>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("query".to_string(), serde_json::Value::String(query.to_string()));
        
        if let Some(w) = name_match_weight {
            payload.insert("name_match_weight".to_string(), serde_json::json!(w));
        }
        if let Some(w) = term_boost_weight {
            payload.insert("term_boost_weight".to_string(), serde_json::json!(w));
        }
        if let Some(w) = signal_boost_weight {
            payload.insert("signal_boost_weight".to_string(), serde_json::json!(w));
        }

        let response = self.make_request("POST", "/discovery/score_collections", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse score response: {}", e)))?;
        Ok(result)
    }

    /// Generate query variations
    pub async fn expand_queries(
        &self,
        query: &str,
        max_expansions: Option<usize>,
        include_definition: Option<bool>,
        include_features: Option<bool>,
        include_architecture: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("query".to_string(), serde_json::Value::String(query.to_string()));
        
        if let Some(max) = max_expansions {
            payload.insert("max_expansions".to_string(), serde_json::Value::Number(max.into()));
        }
        if let Some(def) = include_definition {
            payload.insert("include_definition".to_string(), serde_json::Value::Bool(def));
        }
        if let Some(feat) = include_features {
            payload.insert("include_features".to_string(), serde_json::Value::Bool(feat));
        }
        if let Some(arch) = include_architecture {
            payload.insert("include_architecture".to_string(), serde_json::Value::Bool(arch));
        }

        let response = self.make_request("POST", "/discovery/expand_queries", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse expand response: {}", e)))?;
        Ok(result)
    }

    // =============================================================================
    // FILE OPERATIONS
    // =============================================================================

    /// Retrieve complete file content from a collection
    pub async fn get_file_content(
        &self,
        collection: &str,
        file_path: &str,
        max_size_kb: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        payload.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
        
        if let Some(max) = max_size_kb {
            payload.insert("max_size_kb".to_string(), serde_json::Value::Number(max.into()));
        }

        let response = self.make_request("POST", "/file/content", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse file content response: {}", e)))?;
        Ok(result)
    }

    /// List all indexed files in a collection
    pub async fn list_files_in_collection(
        &self,
        collection: &str,
        filter_by_type: Option<Vec<String>>,
        min_chunks: Option<usize>,
        max_results: Option<usize>,
        sort_by: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        
        if let Some(types) = filter_by_type {
            payload.insert("filter_by_type".to_string(), serde_json::to_value(types).unwrap());
        }
        if let Some(min) = min_chunks {
            payload.insert("min_chunks".to_string(), serde_json::Value::Number(min.into()));
        }
        if let Some(max) = max_results {
            payload.insert("max_results".to_string(), serde_json::Value::Number(max.into()));
        }
        if let Some(sort) = sort_by {
            payload.insert("sort_by".to_string(), serde_json::Value::String(sort.to_string()));
        }

        let response = self.make_request("POST", "/file/list", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse list files response: {}", e)))?;
        Ok(result)
    }

    /// Get extractive or structural summary of an indexed file
    pub async fn get_file_summary(
        &self,
        collection: &str,
        file_path: &str,
        summary_type: Option<&str>,
        max_sentences: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        payload.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
        
        if let Some(stype) = summary_type {
            payload.insert("summary_type".to_string(), serde_json::Value::String(stype.to_string()));
        }
        if let Some(max) = max_sentences {
            payload.insert("max_sentences".to_string(), serde_json::Value::Number(max.into()));
        }

        let response = self.make_request("POST", "/file/summary", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse file summary response: {}", e)))?;
        Ok(result)
    }

    /// Retrieve chunks in original file order for progressive reading
    pub async fn get_file_chunks_ordered(
        &self,
        collection: &str,
        file_path: &str,
        start_chunk: Option<usize>,
        limit: Option<usize>,
        include_context: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        payload.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
        
        if let Some(start) = start_chunk {
            payload.insert("start_chunk".to_string(), serde_json::Value::Number(start.into()));
        }
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(ctx) = include_context {
            payload.insert("include_context".to_string(), serde_json::Value::Bool(ctx));
        }

        let response = self.make_request("POST", "/file/chunks", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse chunks response: {}", e)))?;
        Ok(result)
    }

    /// Generate hierarchical project structure overview
    pub async fn get_project_outline(
        &self,
        collection: &str,
        max_depth: Option<usize>,
        include_summaries: Option<bool>,
        highlight_key_files: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        
        if let Some(depth) = max_depth {
            payload.insert("max_depth".to_string(), serde_json::Value::Number(depth.into()));
        }
        if let Some(summ) = include_summaries {
            payload.insert("include_summaries".to_string(), serde_json::Value::Bool(summ));
        }
        if let Some(highlight) = highlight_key_files {
            payload.insert("highlight_key_files".to_string(), serde_json::Value::Bool(highlight));
        }

        let response = self.make_request("POST", "/file/outline", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse outline response: {}", e)))?;
        Ok(result)
    }

    /// Find semantically related files using vector similarity
    pub async fn get_related_files(
        &self,
        collection: &str,
        file_path: &str,
        limit: Option<usize>,
        similarity_threshold: Option<f32>,
        include_reason: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        payload.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
        
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(thresh) = similarity_threshold {
            payload.insert("similarity_threshold".to_string(), serde_json::json!(thresh));
        }
        if let Some(reason) = include_reason {
            payload.insert("include_reason".to_string(), serde_json::Value::Bool(reason));
        }

        let response = self.make_request("POST", "/file/related", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse related files response: {}", e)))?;
        Ok(result)
    }

    /// Semantic search filtered by file type
    pub async fn search_by_file_type(
        &self,
        collection: &str,
        query: &str,
        file_types: Vec<String>,
        limit: Option<usize>,
        return_full_files: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
        payload.insert("query".to_string(), serde_json::Value::String(query.to_string()));
        payload.insert("file_types".to_string(), serde_json::to_value(file_types).unwrap());
        
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(full) = return_full_files {
            payload.insert("return_full_files".to_string(), serde_json::Value::Bool(full));
        }

        let response = self.make_request("POST", "/file/search_by_type", Some(serde_json::Value::Object(payload))).await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse search by type response: {}", e)))?;
        Ok(result)
    }

    /// Make HTTP request
    async fn make_request(
        &self,
        method: &str,
        endpoint: &str,
        payload: Option<serde_json::Value>,
    ) -> Result<String> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request = match method {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(VectorizerError::configuration(format!("Unsupported HTTP method: {}", method))),
        };

        if let Some(payload) = payload {
            request = request.json(&payload);
        }

        let response = request
            .send()
            .await
            .map_err(|e| VectorizerError::network(format!("Request failed: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VectorizerError::server(format!("HTTP {}: {}", status.as_u16(), error_text)));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| VectorizerError::network(format!("Failed to read response: {}", e)))?;

        Ok(response_text)
    }
}
