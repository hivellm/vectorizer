//! Thin wrapper for indexing using existing embedding infrastructure

use std::path::PathBuf;

use anyhow::{Context, Result};
use rayon::prelude::*;
use tracing::{info, warn};

use super::config::{DocumentChunk, LoaderConfig};
use crate::{
    VectorStore,
    embedding::EmbeddingManager, // Use existing EmbeddingManager
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector},
};

/// Thin indexer - delegates to existing embedding module
pub struct Indexer {
    config: LoaderConfig,
    embedding_manager: EmbeddingManager,
}

impl Indexer {
    /// Create indexer with shared embedding manager
    pub fn with_embedding_manager(
        config: LoaderConfig,
        embedding_manager: EmbeddingManager,
    ) -> Self {
        Self {
            config,
            embedding_manager,
        }
    }

    /// Build vocabulary (delegates to EmbeddingManager)
    pub fn build_vocabulary(&mut self, documents: &[(PathBuf, String)]) -> Result<()> {
        // Extract texts from documents
        let texts: Vec<String> = documents
            .iter()
            .map(|(_, content)| content.clone())
            .collect();

        if texts.is_empty() {
            warn!("No documents to build vocabulary from");
            return Ok(());
        }

        info!(
            "Building vocabulary from {} documents for provider: {}",
            texts.len(),
            self.config.embedding_type
        );

        // Build vocabulary using the embedding provider
        let provider = self.config.embedding_type.as_str();

        if let Some(emb) = self.embedding_manager.get_provider_mut(provider) {
            // Try to downcast to specific types that support vocabulary building
            if let Some(bm25) = emb
                .as_any_mut()
                .downcast_mut::<crate::embedding::Bm25Embedding>()
            {
                bm25.build_vocabulary(&texts);
                info!(
                    "✅ Built BM25 vocabulary with {} documents, vocab size: {}",
                    texts.len(),
                    bm25.vocabulary_size()
                );
            } else if let Some(tfidf) = emb
                .as_any_mut()
                .downcast_mut::<crate::embedding::TfIdfEmbedding>()
            {
                // TF-IDF expects &[&str]
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                tfidf.build_vocabulary(&text_refs);
                info!("✅ Built TF-IDF vocabulary with {} documents", texts.len());
            } else if let Some(bow) = emb
                .as_any_mut()
                .downcast_mut::<crate::embedding::BagOfWordsEmbedding>()
            {
                // BagOfWords expects &[&str]
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                bow.build_vocabulary(&text_refs);
                info!(
                    "✅ Built Bag-of-Words vocabulary with {} documents",
                    texts.len()
                );
            } else if let Some(charngram) = emb
                .as_any_mut()
                .downcast_mut::<crate::embedding::CharNGramEmbedding>()
            {
                // CharNGram expects &[&str]
                let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                charngram.build_vocabulary(&text_refs);
                info!(
                    "✅ Built CharNGram vocabulary with {} documents",
                    texts.len()
                );
            } else {
                info!(
                    "Provider '{}' does not require vocabulary building",
                    provider
                );
            }
        } else {
            warn!("Embedding provider '{}' not found in manager", provider);
        }

        Ok(())
    }

    /// Create collection
    pub fn create_collection(&self, store: &VectorStore) -> Result<()> {
        if store.has_collection_in_memory(&self.config.collection_name) {
            return Ok(());
        }

        let config = CollectionConfig {
            dimension: self.config.embedding_dimension,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::SQ { bits: 8 },
            compression: Default::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
            sharding: None,
            graph: None,
            encryption: None,
        };

        store
            .create_collection_with_quantization(&self.config.collection_name, config)
            .with_context(|| {
                format!(
                    "Failed to create collection '{}'",
                    self.config.collection_name
                )
            })?;

        Ok(())
    }

    /// Store chunks using existing embedding infrastructure
    pub fn store_chunks_parallel(
        &self,
        store: &VectorStore,
        chunks: &[DocumentChunk],
    ) -> Result<usize> {
        const BATCH_SIZE: usize = 256;
        let mut total_vectors = 0;

        for batch in chunks.chunks(BATCH_SIZE) {
            let batch_vectors: Vec<Vector> = batch
                .par_iter()
                .filter_map(|chunk| {
                    // Use existing EmbeddingManager
                    match self.embedding_manager.embed(&chunk.content) {
                        Ok(embedding) => {
                            if embedding.iter().all(|&x| x == 0.0) {
                                return None;
                            }

                            let mut payload = Payload {
                                data: serde_json::json!({
                                    "content": chunk.content,
                                    "file_path": chunk.file_path,
                                    "chunk_index": chunk.chunk_index,
                                    "metadata": chunk.metadata
                                }),
                            };
                            payload.normalize();

                            Some(Vector {
                                id: uuid::Uuid::new_v4().to_string(),
                                data: embedding,
                                sparse: None,
                                payload: Some(payload),
                            })
                        }
                        Err(e) => {
                            warn!("Failed to embed chunk: {}", e);
                            None
                        }
                    }
                })
                .collect();

            if !batch_vectors.is_empty() {
                store.insert(&self.config.collection_name, batch_vectors)?;
                total_vectors += batch.len();
            }
        }

        Ok(total_vectors)
    }

    /// Save vocabulary/tokenizer for file watcher
    pub fn save_vocabulary(
        &self,
        path: &std::path::Path,
        provider_name: &str,
    ) -> crate::error::Result<()> {
        self.embedding_manager
            .save_vocabulary_json(provider_name, path)
    }
}
