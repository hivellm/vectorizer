//! Embedding provider implementations
//!
//! This module provides concrete implementations of the EmbeddingProvider trait
//! for various embedding models and services.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::embedding::{EmbeddingError, EmbeddingProvider, EmbeddingProviderType};

/// Provider factory for creating embedding providers
pub struct EmbeddingProviderFactory;

impl EmbeddingProviderFactory {
    /// Create a provider based on type
    pub async fn create_provider(
        provider_type: EmbeddingProviderType,
        config: ProviderConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>, EmbeddingError> {
        match provider_type {
            EmbeddingProviderType::BM25 => {
                let bm25_provider = crate::embedding::bm25::BM25Factory::create_default();
                Ok(Arc::new(bm25_provider))
            }
            EmbeddingProviderType::BERT => {
                let bert_provider =
                    crate::embedding::bert::BERTFactory::create_and_initialize(None).await?;
                Ok(Arc::new(bert_provider))
            }
            EmbeddingProviderType::OpenAI => {
                let openai_provider = crate::embedding::openai::OpenAIFactory::create_with_api_key(
                    config.api_key.unwrap_or_default(),
                )
                .await?;
                Ok(Arc::new(openai_provider))
            }
            EmbeddingProviderType::SentenceTransformers => {
                // Placeholder for SentenceTransformers implementation
                Err(EmbeddingError::ProviderNotAvailable(
                    "SentenceTransformers not implemented".to_string(),
                ))
            }
            EmbeddingProviderType::Custom(name) => Err(EmbeddingError::ProviderNotAvailable(
                format!("Custom provider '{}' not implemented", name),
            )),
        }
    }

    /// Get available provider types
    pub fn available_providers() -> Vec<EmbeddingProviderType> {
        vec![
            EmbeddingProviderType::BM25,
            EmbeddingProviderType::BERT,
            EmbeddingProviderType::OpenAI,
            // EmbeddingProviderType::SentenceTransformers, // Not implemented yet
        ]
    }

    /// Get provider information
    pub fn get_provider_info(provider_type: &EmbeddingProviderType) -> ProviderInfo {
        match provider_type {
            EmbeddingProviderType::BM25 => ProviderInfo {
                name: "BM25".to_string(),
                description: "Sparse embeddings using BM25 scoring".to_string(),
                dimension: 0, // Variable based on vocabulary
                max_text_length: usize::MAX,
                requires_training: true,
                supports_batch: true,
            },
            EmbeddingProviderType::BERT => ProviderInfo {
                name: "BERT".to_string(),
                description: "Dense embeddings using BERT models".to_string(),
                dimension: 768,
                max_text_length: 512,
                requires_training: false,
                supports_batch: true,
            },
            EmbeddingProviderType::OpenAI => ProviderInfo {
                name: "OpenAI".to_string(),
                description: "High-quality embeddings from OpenAI API".to_string(),
                dimension: 1536,
                max_text_length: 8191,
                requires_training: false,
                supports_batch: true,
            },
            EmbeddingProviderType::SentenceTransformers => ProviderInfo {
                name: "SentenceTransformers".to_string(),
                description: "Sentence embeddings using transformers".to_string(),
                dimension: 384,
                max_text_length: 512,
                requires_training: false,
                supports_batch: true,
            },
            EmbeddingProviderType::Custom(name) => ProviderInfo {
                name: format!("Custom({})", name),
                description: "Custom embedding provider".to_string(),
                dimension: 0,
                max_text_length: 0,
                requires_training: false,
                supports_batch: false,
            },
        }
    }
}

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for external services
    pub api_key: Option<String>,
    /// Model name or identifier
    pub model: Option<String>,
    /// Additional configuration parameters
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: None,
            parameters: std::collections::HashMap::new(),
        }
    }
}

/// Information about an embedding provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name
    pub name: String,
    /// Provider description
    pub description: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Maximum text length
    pub max_text_length: usize,
    /// Whether provider requires training
    pub requires_training: bool,
    /// Whether provider supports batch processing
    pub supports_batch: bool,
}

/// Provider registry for managing multiple providers
pub struct ProviderRegistry {
    providers: std::collections::HashMap<EmbeddingProviderType, Arc<dyn EmbeddingProvider>>,
    default_provider: EmbeddingProviderType,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new(default_provider: EmbeddingProviderType) -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            default_provider,
        }
    }

    /// Register a provider
    pub fn register_provider(
        &mut self,
        provider_type: EmbeddingProviderType,
        provider: Arc<dyn EmbeddingProvider>,
    ) {
        self.providers.insert(provider_type, provider);
    }

    /// Get a provider by type
    pub fn get_provider(
        &self,
        provider_type: &EmbeddingProviderType,
    ) -> Option<&Arc<dyn EmbeddingProvider>> {
        self.providers.get(provider_type)
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Option<&Arc<dyn EmbeddingProvider>> {
        self.get_provider(&self.default_provider)
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, provider_type: EmbeddingProviderType) {
        self.default_provider = provider_type;
    }

    /// Get all registered providers
    pub fn get_providers(
        &self,
    ) -> &std::collections::HashMap<EmbeddingProviderType, Arc<dyn EmbeddingProvider>> {
        &self.providers
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, provider_type: &EmbeddingProviderType) -> bool {
        self.providers.contains_key(provider_type)
    }

    /// Get available provider types
    pub fn available_providers(&self) -> Vec<EmbeddingProviderType> {
        self.providers.keys().cloned().collect()
    }
}

/// Provider health checker
pub struct ProviderHealthChecker;

impl ProviderHealthChecker {
    /// Check the health of a provider
    pub async fn check_health(provider: &dyn EmbeddingProvider) -> ProviderHealth {
        let start_time = std::time::Instant::now();

        // Test with a simple text
        let test_text = "Hello, world!";
        let result = provider.embed(test_text).await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(embedding) => {
                if embedding.is_empty() {
                    ProviderHealth::Unhealthy {
                        reason: "Empty embedding returned".to_string(),
                        response_time_ms: response_time,
                    }
                } else {
                    ProviderHealth::Healthy {
                        response_time_ms: response_time,
                        embedding_dimension: embedding.len(),
                    }
                }
            }
            Err(e) => ProviderHealth::Unhealthy {
                reason: format!("Embedding failed: {}", e),
                response_time_ms: response_time,
            },
        }
    }

    /// Check health of multiple providers
    pub async fn check_multiple_providers(
        providers: &std::collections::HashMap<EmbeddingProviderType, Arc<dyn EmbeddingProvider>>,
    ) -> std::collections::HashMap<EmbeddingProviderType, ProviderHealth> {
        let mut results = std::collections::HashMap::new();

        for (provider_type, provider) in providers {
            let health = Self::check_health(provider.as_ref()).await;
            results.insert(provider_type.clone(), health);
        }

        results
    }
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderHealth {
    /// Provider is healthy
    Healthy {
        /// Response time in milliseconds
        response_time_ms: u64,
        /// Embedding dimension
        embedding_dimension: usize,
    },
    /// Provider is unhealthy
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
        /// Response time in milliseconds
        response_time_ms: u64,
    },
}

impl ProviderHealth {
    /// Check if provider is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, ProviderHealth::Healthy { .. })
    }

    /// Get response time
    pub fn response_time_ms(&self) -> u64 {
        match self {
            ProviderHealth::Healthy {
                response_time_ms, ..
            } => *response_time_ms,
            ProviderHealth::Unhealthy {
                response_time_ms, ..
            } => *response_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_factory_available_providers() {
        let providers = EmbeddingProviderFactory::available_providers();
        assert!(!providers.is_empty());
        assert!(providers.contains(&EmbeddingProviderType::BM25));
        assert!(providers.contains(&EmbeddingProviderType::BERT));
        assert!(providers.contains(&EmbeddingProviderType::OpenAI));
    }

    #[test]
    fn test_provider_info() {
        let bm25_info = EmbeddingProviderFactory::get_provider_info(&EmbeddingProviderType::BM25);
        assert_eq!(bm25_info.name, "BM25");
        assert!(bm25_info.requires_training);

        let bert_info = EmbeddingProviderFactory::get_provider_info(&EmbeddingProviderType::BERT);
        assert_eq!(bert_info.name, "BERT");
        assert_eq!(bert_info.dimension, 768);
        assert!(!bert_info.requires_training);
    }

    #[test]
    fn test_provider_registry() {
        let mut registry = ProviderRegistry::new(EmbeddingProviderType::BM25);
        assert_eq!(registry.available_providers().len(), 0);
        assert!(!registry.has_provider(&EmbeddingProviderType::BM25));
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.model.is_none());
        assert!(config.parameters.is_empty());
    }

    #[test]
    fn test_provider_health() {
        let healthy = ProviderHealth::Healthy {
            response_time_ms: 100,
            embedding_dimension: 768,
        };
        assert!(healthy.is_healthy());
        assert_eq!(healthy.response_time_ms(), 100);

        let unhealthy = ProviderHealth::Unhealthy {
            reason: "Test error".to_string(),
            response_time_ms: 50,
        };
        assert!(!unhealthy.is_healthy());
        assert_eq!(unhealthy.response_time_ms(), 50);
    }
}
