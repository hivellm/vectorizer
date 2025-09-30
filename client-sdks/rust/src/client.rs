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
            base_url: "http://localhost:15001".to_string(),
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
        let response = self.make_request("GET", "/api/v1/health", None).await?;
        let health: HealthStatus = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse health check response: {}", e)))?;
        Ok(health)
    }

    /// List collections
    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self.make_request("GET", "/api/v1/collections", None).await?;
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

        let response = self.make_request("POST", &format!("/api/v1/collections/{}/search/text", collection), Some(serde_json::Value::Object(payload))).await?;
        let search_response: SearchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse search response: {}", e)))?;
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

        let response = self.make_request("POST", "/api/v1/collections", Some(serde_json::Value::Object(payload))).await?;
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

        let response = self.make_request("POST", &format!("/api/v1/collections/{}/documents", collection), Some(serde_json::to_value(payload)?)).await?;
        let batch_response: BatchResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse insert texts response: {}", e)))?;
        Ok(batch_response)
    }

    /// Delete collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/api/v1/collections/{}", name), None).await?;
        Ok(())
    }

    /// Get vector
    pub async fn get_vector(&self, collection: &str, vector_id: &str) -> Result<Vector> {
        let response = self.make_request("GET", &format!("/api/v1/collections/{}/vectors/{}", collection, vector_id), None).await?;
        let vector: Vector = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse get vector response: {}", e)))?;
        Ok(vector)
    }

    /// Get collection info
    pub async fn get_collection_info(&self, collection: &str) -> Result<CollectionInfo> {
        let response = self.make_request("GET", &format!("/api/v1/collections/{}", collection), None).await?;
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

        let response = self.make_request("POST", "/api/v1/embed", Some(serde_json::Value::Object(payload))).await?;
        let embedding_response: EmbeddingResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse embedding response: {}", e)))?;
        Ok(embedding_response)
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
