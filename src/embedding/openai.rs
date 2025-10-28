//! OpenAI embedding provider
//!
//! This module provides integration with OpenAI's embedding API for generating
//! high-quality vector representations of text using various OpenAI models.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::embedding::{EmbeddingError, EmbeddingProvider, EmbeddingProviderType};

/// OpenAI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// API key for OpenAI
    pub api_key: String,
    /// Model name to use
    pub model: String,
    /// Organization ID (optional)
    pub organization: Option<String>,
    /// Base URL for API (for custom endpoints)
    pub base_url: Option<String>,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Timeout for requests (seconds)
    pub timeout_seconds: u64,
    /// Batch size for processing
    pub batch_size: usize,
    /// Enable request logging
    pub enable_logging: bool,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "text-embedding-ada-002".to_string(),
            organization: None,
            base_url: None,
            max_retries: 3,
            timeout_seconds: 30,
            batch_size: 100,
            enable_logging: false,
        }
    }
}

/// OpenAI embedding provider
pub struct OpenAIProvider {
    config: OpenAIConfig,
    client: Arc<RwLock<Option<OpenAIClient>>>,
    dimension: usize,
}

/// OpenAI API client (simulated)
struct OpenAIClient {
    api_key: String,
    base_url: String,
    organization: Option<String>,
}

impl OpenAIClient {
    fn new(config: &OpenAIConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            organization: config.organization.clone(),
        }
    }

    async fn create_embeddings(&self, input: &[String]) -> Result<OpenAIResponse, EmbeddingError> {
        // Simulate API call
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let embeddings = input
            .iter()
            .map(|_| {
                // Generate random embedding for simulation
                (0..1536).map(|_| fastrand::f32() - 0.5).collect()
            })
            .collect();

        Ok(OpenAIResponse {
            data: embeddings,
            model: self.model().to_string(),
            usage: Usage {
                prompt_tokens: input.iter().map(|s| s.len() / 4).sum(),
                total_tokens: input.iter().map(|s| s.len() / 4).sum(),
            },
        })
    }

    fn model(&self) -> &str {
        "text-embedding-ada-002"
    }
}

/// OpenAI API response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    data: Vec<Vec<f32>>,
    model: String,
    usage: Usage,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: OpenAIConfig) -> Self {
        let dimension = match config.model.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536, // Default
        };

        Self {
            config,
            client: Arc::new(RwLock::new(None)),
            dimension,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(OpenAIConfig::default())
    }

    /// Initialize the OpenAI client
    pub async fn initialize(&self) -> Result<(), EmbeddingError> {
        if self.config.api_key.is_empty() {
            return Err(EmbeddingError::InvalidConfiguration(
                "API key is required".to_string(),
            ));
        }

        let client = OpenAIClient::new(&self.config);
        *self.client.write().await = Some(client);
        Ok(())
    }

    /// Generate OpenAI embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let client = self.client.read().await;
        let client = client
            .as_ref()
            .ok_or_else(|| EmbeddingError::Internal("OpenAI client not initialized".to_string()))?;

        let response = client.create_embeddings(&[text.to_string()]).await?;

        if response.data.is_empty() {
            return Err(EmbeddingError::Internal(
                "Empty response from OpenAI API".to_string(),
            ));
        }

        Ok(response.data[0].clone())
    }

    /// Generate embeddings for multiple texts
    async fn generate_embeddings_batch(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let client = self.client.read().await;
        let client = client
            .as_ref()
            .ok_or_else(|| EmbeddingError::Internal("OpenAI client not initialized".to_string()))?;

        let response = client.create_embeddings(texts).await?;
        Ok(response.data)
    }

    /// Get API usage statistics
    pub async fn get_usage_stats(&self) -> Option<UsageStats> {
        // In a real implementation, this would track actual usage
        Some(UsageStats {
            total_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
        })
    }

    /// Get available models
    pub fn available_models() -> Vec<OpenAIModel> {
        vec![
            OpenAIModel {
                name: "text-embedding-ada-002".to_string(),
                dimension: 1536,
                max_tokens: 8191,
                description: "Most capable embedding model".to_string(),
            },
            OpenAIModel {
                name: "text-embedding-3-small".to_string(),
                dimension: 1536,
                max_tokens: 8191,
                description: "Smaller, faster embedding model".to_string(),
            },
            OpenAIModel {
                name: "text-embedding-3-large".to_string(),
                dimension: 3072,
                max_tokens: 8191,
                description: "Largest embedding model".to_string(),
            },
        ]
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total number of requests
    pub total_requests: usize,
    /// Total tokens used
    pub total_tokens: usize,
    /// Total cost in USD
    pub total_cost: f64,
}

/// OpenAI model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModel {
    /// Model name
    pub name: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Maximum tokens
    pub max_tokens: usize,
    /// Model description
    pub description: String,
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenAIProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.generate_embedding(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.generate_embeddings_batch(texts).await
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn is_available(&self) -> bool {
        self.client.read().await.is_some()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// OpenAI factory for creating providers
pub struct OpenAIFactory;

impl OpenAIFactory {
    /// Create a new OpenAI provider with default config
    pub fn create_default() -> OpenAIProvider {
        OpenAIProvider::default()
    }

    /// Create a new OpenAI provider with custom config
    pub fn create_with_config(config: OpenAIConfig) -> OpenAIProvider {
        OpenAIProvider::new(config)
    }

    /// Create an OpenAI provider and initialize it
    pub async fn create_and_initialize(
        api_key: String,
        model: Option<String>,
    ) -> Result<OpenAIProvider, EmbeddingError> {
        let mut config = OpenAIConfig::default();
        config.api_key = api_key;
        if let Some(model) = model {
            config.model = model;
        }

        let provider = OpenAIProvider::new(config);
        provider.initialize().await?;
        Ok(provider)
    }

    /// Create with API key only
    pub async fn create_with_api_key(api_key: String) -> Result<OpenAIProvider, EmbeddingError> {
        Self::create_and_initialize(api_key, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_provider_creation() {
        let config = OpenAIConfig::default();
        let provider = OpenAIProvider::new(config);

        assert_eq!(provider.name(), "OpenAI");
        assert_eq!(provider.dimension(), 1536);
        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn test_openai_initialization() {
        let mut config = OpenAIConfig::default();
        config.api_key = "test-key".to_string();

        let provider = OpenAIProvider::new(config);
        assert!(!provider.is_available().await);

        provider.initialize().await.unwrap();
        assert!(provider.is_available().await);
    }

    #[tokio::test]
    async fn test_openai_embedding() {
        let mut config = OpenAIConfig::default();
        config.api_key = "test-key".to_string();

        let provider = OpenAIProvider::new(config);
        provider.initialize().await.unwrap();

        let embedding = provider.embed("Hello, world!").await.unwrap();
        assert_eq!(embedding.len(), 1536);
    }

    #[tokio::test]
    async fn test_openai_batch_embedding() {
        let mut config = OpenAIConfig::default();
        config.api_key = "test-key".to_string();

        let provider = OpenAIProvider::new(config);
        provider.initialize().await.unwrap();

        let texts = vec!["Hello, world!".to_string(), "This is a test".to_string()];

        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 1536);
        assert_eq!(embeddings[1].len(), 1536);
    }

    #[tokio::test]
    async fn test_openai_factory() {
        let provider = OpenAIFactory::create_with_api_key("test-key".to_string())
            .await
            .unwrap();
        assert!(provider.is_available().await);
        assert_eq!(provider.dimension(), 1536);
    }

    #[test]
    fn test_openai_config_default() {
        let config = OpenAIConfig::default();
        assert_eq!(config.model, "text-embedding-ada-002");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_openai_available_models() {
        let models = OpenAIProvider::available_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.name == "text-embedding-ada-002"));
    }

    #[tokio::test]
    async fn test_openai_usage_stats() {
        let mut config = OpenAIConfig::default();
        config.api_key = "test-key".to_string();

        let provider = OpenAIProvider::new(config);
        provider.initialize().await.unwrap();

        let stats = provider.get_usage_stats().await;
        assert!(stats.is_some());
    }
}
