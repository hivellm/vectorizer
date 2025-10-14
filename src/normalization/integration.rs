//! Integration helpers for text normalization pipeline
//!
//! This module provides integration utilities to connect text normalization
//! with the vector ingestion and search pipelines.

use super::{
    cache::CacheManager, config::NormalizationConfig, ContentHashCalculator, ContentType,
    ContentTypeDetector, NormalizedContent, TextNormalizer,
};
use anyhow::Result;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;

/// Normalization pipeline that integrates detector, normalizer, and cache
pub struct NormalizationPipeline {
    /// Content type detector
    detector: ContentTypeDetector,
    /// Text normalizer
    normalizer: TextNormalizer,
    /// Optional cache manager
    cache: Option<Arc<RwLock<CacheManager>>>,
    /// Configuration
    config: NormalizationConfig,
}

impl NormalizationPipeline {
    /// Create a new normalization pipeline from configuration
    pub fn new(config: NormalizationConfig, data_dir: &Path) -> Result<Self> {
        let detector = ContentTypeDetector::new();
        let normalizer = TextNormalizer::new(config.policy.clone());

        let cache = if config.cache_enabled {
            let cache_config = super::cache::CacheConfig {
                hot_cache_size: config.hot_cache_size,
                warm_store_path: data_dir.join("normalization/warm"),
                cold_store_path: data_dir.join("normalization/cold"),
                compression_level: 3,
                enable_metrics: true,
            };
            Some(Arc::new(RwLock::new(CacheManager::new(cache_config)?)))
        } else {
            None
        };

        Ok(Self {
            detector,
            normalizer,
            cache,
            config,
        })
    }

    /// Process text for ingestion (document normalization)
    ///
    /// Returns the normalized text and optionally the original text
    /// based on configuration.
    pub async fn process_document(
        &self,
        text: &str,
        file_path: Option<&Path>,
    ) -> Result<ProcessedDocument> {
        // Detect content type
        let content_type = self.detector.detect(text, file_path);

        // Normalize the text FIRST (before hashing for cache key)
        // This ensures cache keys are based on normalized content, not raw text
        let normalized = self.normalizer.normalize(text, Some(content_type.clone()));

        // Try to get from cache using the normalized content hash
        if let Some(cache) = &self.cache {
            // Check cache using the hash of the NORMALIZED text
            let cache_read = cache.read();
            if let Some(cached_text) = cache_read.get_normalized(&normalized.content_hash).await? {
                drop(cache_read);
                return Ok(ProcessedDocument {
                    normalized_text: cached_text,
                    original_text: if self.config.store_raw_text {
                        Some(text.to_string())
                    } else {
                        None
                    },
                    content_hash: Some(normalized.content_hash),
                    content_type: content_type.to_string(),
                    from_cache: true,
                });
            }
            drop(cache_read);
        }

        // Store in cache if enabled
        if let Some(cache) = &self.cache {
            let mut cache_write = cache.write();
            cache_write
                .put_normalized(normalized.content_hash, &normalized.text)
                .await?;
        }

        Ok(ProcessedDocument {
            normalized_text: normalized.text,
            original_text: if self.config.store_raw_text {
                Some(text.to_string())
            } else {
                None
            },
            content_hash: Some(normalized.content_hash),
            content_type: content_type.to_string(),
            from_cache: false,
        })
    }

    /// Process text for search queries
    ///
    /// Queries always use aggressive normalization for better matching,
    /// unless explicitly disabled in configuration.
    pub fn process_query(&self, query: &str) -> String {
        if !self.config.normalize_queries {
            return query.to_string();
        }

        self.normalizer.normalize_query(query)
    }

    /// Check if a document is a duplicate based on content hash
    ///
    /// Returns true if the content hash already exists in cache
    pub async fn is_duplicate(&self, text: &str) -> Result<bool> {
        if let Some(cache) = &self.cache {
            let hash_calculator = ContentHashCalculator::new();
            let content_hash = hash_calculator.hash(text);

            let cache_read = cache.read();
            let result = cache_read.get_normalized(&content_hash).await?;
            Ok(result.is_some())
        } else {
            Ok(false)
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Option<super::cache::CacheStats> {
        self.cache.as_ref().map(|c| c.read().stats())
    }

    /// Clear all caches
    pub async fn clear_cache(&self) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.write().clear().await?;
        }
        Ok(())
    }
}

/// Processed document with normalization metadata
#[derive(Debug, Clone)]
pub struct ProcessedDocument {
    /// Normalized text (ready for embedding)
    pub normalized_text: String,
    /// Original text (optional, based on configuration)
    pub original_text: Option<String>,
    /// Content hash for deduplication
    pub content_hash: Option<super::ContentHash>,
    /// Detected content type
    pub content_type: String,
    /// Whether this result came from cache
    pub from_cache: bool,
}

impl ProcessedDocument {
    /// Get the text to use for embedding generation
    pub fn embedding_text(&self) -> &str {
        &self.normalized_text
    }

    /// Get the text to display to users
    pub fn display_text(&self) -> &str {
        self.original_text
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(&self.normalized_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_pipeline_basic() {
        let dir = tempdir().unwrap();
        let config = NormalizationConfig::moderate();
        let pipeline = NormalizationPipeline::new(config, dir.path()).unwrap();

        let text = "Hello   World\n\n  Test  ";
        let result = pipeline.process_document(text, None).await.unwrap();

        assert!(!result.normalized_text.is_empty());
        assert!(result.original_text.is_some());
        assert!(!result.from_cache);
    }

    #[tokio::test]
    async fn test_pipeline_cache() {
        let dir = tempdir().unwrap();
        let config = NormalizationConfig::moderate();
        let pipeline = NormalizationPipeline::new(config, dir.path()).unwrap();

        let text = "Hello World";

        // First call - not from cache
        let result1 = pipeline.process_document(text, None).await.unwrap();
        assert!(!result1.from_cache);

        // Second call - should be from cache
        let result2 = pipeline.process_document(text, None).await.unwrap();
        assert!(result2.from_cache);
        assert_eq!(result1.normalized_text, result2.normalized_text);
    }

    #[tokio::test]
    async fn test_query_normalization() {
        let dir = tempdir().unwrap();
        let config = NormalizationConfig::moderate();
        let pipeline = NormalizationPipeline::new(config, dir.path()).unwrap();

        let query = "  Search   Query  ";
        let normalized = pipeline.process_query(query);

        assert!(!normalized.is_empty());
        assert_ne!(query, normalized);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let dir = tempdir().unwrap();
        let config = NormalizationConfig::moderate();
        let pipeline = NormalizationPipeline::new(config, dir.path()).unwrap();

        let text = "Test document";

        // First time - not a duplicate
        assert!(!pipeline.is_duplicate(text).await.unwrap());

        // Process it
        pipeline.process_document(text, None).await.unwrap();

        // Second time - is a duplicate
        assert!(pipeline.is_duplicate(text).await.unwrap());
    }

    #[test]
    fn test_processed_document() {
        let doc = ProcessedDocument {
            normalized_text: "normalized".to_string(),
            original_text: Some("original".to_string()),
            content_hash: None,
            content_type: "plain".to_string(),
            from_cache: false,
        };

        assert_eq!(doc.embedding_text(), "normalized");
        assert_eq!(doc.display_text(), "original");
    }

    #[test]
    fn test_processed_document_no_original() {
        let doc = ProcessedDocument {
            normalized_text: "normalized".to_string(),
            original_text: None,
            content_hash: None,
            content_type: "plain".to_string(),
            from_cache: false,
        };

        assert_eq!(doc.embedding_text(), "normalized");
        assert_eq!(doc.display_text(), "normalized");
    }
}

