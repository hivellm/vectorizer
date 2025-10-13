//! Normalization support for Collections
//!
//! This module provides helper methods to integrate text normalization
//! with vector collections.

use crate::{
    error::Result,
    models::{CollectionConfig, Payload, Vector},
    normalization::{NormalizationPipeline, ProcessedDocument},
};
use serde_json::json;
use std::path::Path;

/// Helper methods for working with normalized text in collections
pub struct CollectionNormalizationHelper {
    pipeline: Option<NormalizationPipeline>,
}

impl CollectionNormalizationHelper {
    /// Create a new helper from collection config
    pub fn from_config(config: &CollectionConfig, data_dir: &Path) -> Result<Self> {
        let pipeline = if let Some(norm_config) = &config.normalization {
            if norm_config.enabled {
                Some(
                    NormalizationPipeline::new(norm_config.clone(), data_dir)
                        .map_err(|e| crate::error::VectorizerError::Other(e.to_string()))?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self { pipeline })
    }

    /// Check if normalization is enabled
    pub fn is_enabled(&self) -> bool {
        self.pipeline.is_some()
    }

    /// Process text for document ingestion
    ///
    /// Returns the processed document with normalized text and metadata
    pub async fn process_document(
        &self,
        text: &str,
        file_path: Option<&Path>,
    ) -> Result<ProcessedDocument> {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .process_document(text, file_path)
                .await
                .map_err(|e| crate::error::VectorizerError::Other(e.to_string()))
        } else {
            // No normalization - return original text
            Ok(ProcessedDocument {
                normalized_text: text.to_string(),
                original_text: Some(text.to_string()),
                content_hash: None,
                content_type: "plain".to_string(),
                from_cache: false,
            })
        }
    }

    /// Process query text for search
    pub fn process_query(&self, query: &str) -> String {
        if let Some(pipeline) = &self.pipeline {
            pipeline.process_query(query)
        } else {
            query.to_string()
        }
    }

    /// Check if text is a duplicate
    pub async fn is_duplicate(&self, text: &str) -> Result<bool> {
        if let Some(pipeline) = &self.pipeline {
            pipeline
                .is_duplicate(text)
                .await
                .map_err(|e| crate::error::VectorizerError::Other(e.to_string()))
        } else {
            Ok(false)
        }
    }

    /// Enrich payload with normalization metadata
    ///
    /// Adds normalized_text, original_text, content_type, and content_hash to payload
    pub fn enrich_payload(
        &self,
        mut payload: Payload,
        processed: &ProcessedDocument,
    ) -> Payload {
        if let Some(pipeline) = &self.pipeline {
            // Add normalization metadata to payload
            if let Some(original) = &processed.original_text {
                payload.data["_original_text"] = json!(original);
            }
            payload.data["_normalized_text"] = json!(processed.normalized_text);
            payload.data["_content_type"] = json!(processed.content_type);
            if let Some(hash) = &processed.content_hash {
                payload.data["_content_hash"] = json!(format!("{:?}", hash));
            }
            payload.data["_from_cache"] = json!(processed.from_cache);
        }
        payload
    }

    /// Create a vector with normalized text
    ///
    /// This is a helper to create a vector with properly enriched payload
    pub fn create_vector_with_normalization(
        &self,
        id: String,
        embedding: Vec<f32>,
        processed: &ProcessedDocument,
        mut payload: Payload,
    ) -> Vector {
        // Enrich payload with normalization data
        payload = self.enrich_payload(payload, processed);

        Vector {
            id,
            data: embedding,
            payload: Some(payload),
        }
    }

    /// Get cache statistics if available
    pub fn cache_stats(&self) -> Option<crate::normalization::CacheStats> {
        self.pipeline.as_ref().and_then(|p| p.cache_stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalization::NormalizationConfig;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_helper_no_normalization() {
        let config = CollectionConfig {
            dimension: 512,
            normalization: None,
            ..Default::default()
        };

        let dir = tempdir().unwrap();
        let helper = CollectionNormalizationHelper::from_config(&config, dir.path()).unwrap();

        assert!(!helper.is_enabled());

        let processed = helper.process_document("test text", None).await.unwrap();
        assert_eq!(processed.normalized_text, "test text");
        assert_eq!(processed.original_text, Some("test text".to_string()));
    }

    #[tokio::test]
    async fn test_helper_with_normalization() {
        let config = CollectionConfig {
            dimension: 512,
            normalization: Some(NormalizationConfig::moderate()),
            ..Default::default()
        };

        let dir = tempdir().unwrap();
        let helper = CollectionNormalizationHelper::from_config(&config, dir.path()).unwrap();

        assert!(helper.is_enabled());

        let processed = helper
            .process_document("  Hello   World  ", None)
            .await
            .unwrap();
        assert!(!processed.normalized_text.is_empty());
    }

    #[tokio::test]
    async fn test_query_processing() {
        let config = CollectionConfig {
            dimension: 512,
            normalization: Some(NormalizationConfig::moderate()),
            ..Default::default()
        };

        let dir = tempdir().unwrap();
        let helper = CollectionNormalizationHelper::from_config(&config, dir.path()).unwrap();

        let query = helper.process_query("  Search   Query  ");
        assert!(!query.is_empty());
    }

    #[test]
    fn test_payload_enrichment() {
        let config = CollectionConfig {
            dimension: 512,
            normalization: Some(NormalizationConfig::moderate()),
            ..Default::default()
        };

        let dir = tempfile::tempdir().unwrap();
        let helper = CollectionNormalizationHelper::from_config(&config, dir.path()).unwrap();

        let processed = ProcessedDocument {
            normalized_text: "normalized".to_string(),
            original_text: Some("original".to_string()),
            content_hash: None,
            content_type: "plain".to_string(),
            from_cache: false,
        };

        let payload = Payload {
            data: json!({
                "file_path": "/test/file.txt"
            }),
        };

        let enriched = helper.enrich_payload(payload, &processed);

        assert_eq!(enriched.data["_normalized_text"], "normalized");
        assert_eq!(enriched.data["_original_text"], "original");
        assert_eq!(enriched.data["_content_type"], "plain");
        assert_eq!(enriched.data["file_path"], "/test/file.txt");
    }
}

