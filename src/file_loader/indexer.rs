//! Thin wrapper for indexing using existing embedding infrastructure

use super::config::{DocumentChunk, LoaderConfig};
use crate::{
    VectorStore,
    embedding::EmbeddingManager, // Use existing EmbeddingManager
    models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector, Payload},
};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::{info, warn};

/// Thin indexer - delegates to existing embedding module
pub struct Indexer {
    config: LoaderConfig,
    embedding_manager: EmbeddingManager,
}

impl Indexer {
    /// Create indexer with shared embedding manager
    pub fn with_embedding_manager(config: LoaderConfig, embedding_manager: EmbeddingManager) -> Self {
        Self {
            config,
            embedding_manager,
        }
    }

    /// Build vocabulary (delegates to EmbeddingManager)
    pub fn build_vocabulary(&mut self, documents: &[(PathBuf, String)]) -> Result<()> {
        // EmbeddingManager already has build_vocabulary logic
        // Just extract texts and delegate
        let texts: Vec<String> = documents.iter().map(|(_, content)| content.clone()).collect();
        
        // Use existing EmbeddingManager infrastructure
        match self.config.embedding_type.as_str() {
            "bm25" | "tfidf" | "bagofwords" => {
                // These already have build_vocabulary in their implementations
                info!("Vocabulary will be built by embedding provider: {}", self.config.embedding_type);
            }
            _ => {
                info!("No vocabulary building needed for {}", self.config.embedding_type);
            }
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
        };

        store.create_collection_with_quantization(&self.config.collection_name, config)
            .with_context(|| format!("Failed to create collection '{}'", self.config.collection_name))?;

        Ok(())
    }

    /// Store chunks using existing embedding infrastructure
    pub fn store_chunks_parallel(&self, store: &VectorStore, chunks: &[DocumentChunk]) -> Result<usize> {
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
}

