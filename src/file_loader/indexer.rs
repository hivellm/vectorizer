//! Thin wrapper for indexing using existing embedding infrastructure

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use rayon::prelude::*;
use tracing::{error, info, warn};

use super::config::{DocumentChunk, LoaderConfig};
use crate::{
    VectorStore,
    embedding::EmbeddingManager, // Use existing EmbeddingManager
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig, Vector},
};

/// Thin indexer - delegates to existing embedding module
pub struct Indexer {
    config: LoaderConfig,
    embedding_manager: Arc<EmbeddingManager>,
}

impl Indexer {
    /// Create indexer with shared embedding manager
    pub fn with_embedding_manager(
        config: LoaderConfig,
        embedding_manager: EmbeddingManager,
    ) -> Self {
        Self {
            config,
            embedding_manager: Arc::new(embedding_manager),
        }
    }

    /// Get the embedding manager as Arc
    pub fn embedding_manager_arc(&self) -> Arc<EmbeddingManager> {
        self.embedding_manager.clone()
    }

    /// Build vocabulary (CRITICAL: builds TOP N terms where N = embedding_dimension)
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

        // Build vocabulary with TOP embedding_dimension terms (e.g. 512)
        if self.config.embedding_type == "bm25" {
            // Get the BM25 provider
            let bm25_provider = self.embedding_manager
                .get_provider(&crate::embedding::EmbeddingProviderType::BM25)
                .ok_or_else(|| anyhow::anyhow!("BM25 provider not found"))?;
            
            // Downcast to BM25Provider to call add_documents
            let bm25 = bm25_provider
                .as_any()
                .downcast_ref::<crate::embedding::BM25Provider>()
                .ok_or_else(|| anyhow::anyhow!("Failed to downcast to BM25Provider"))?;
            
            // Train with all documents - this will select TOP embedding_dimension terms
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    bm25.add_documents(&texts).await
                })
            })?;
            
            let vocab_size = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    bm25.vocabulary_size().await
                })
            });
            
            info!(
                "✅ BM25 vocabulary built: {} terms (target: {} dimensions)",
                vocab_size, self.config.embedding_dimension
            );
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

        // Use sequential processing since embedding is async
        for batch in chunks.chunks(BATCH_SIZE) {
            let mut batch_vectors: Vec<Vector> = Vec::with_capacity(batch.len());

            for chunk in batch {
                // Spawn blocking task for embedding (sync wrapper around async)
                let content = chunk.content.clone();
                let embedding_manager = &self.embedding_manager;

                let embedding_result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { embedding_manager.embed(&content).await })
                });

                match embedding_result {
                    Ok(emb_result) => {
                        let embedding = emb_result.embedding;
                        if embedding.iter().all(|&x| x == 0.0) {
                            continue;
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

                        batch_vectors.push(Vector {
                            id: uuid::Uuid::new_v4().to_string(),
                            data: embedding,
                            payload: Some(payload),
                        });
                    }
                    Err(e) => {
                        warn!("Failed to embed chunk: {}", e);
                    }
                }
            }

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
        use std::fs;
        
        info!("🔍 save_vocabulary called for provider '{}' at path: {}", provider_name, path.display());
        
        if provider_name != "bm25" {
            info!("Vocabulary save only needed for BM25, skipping for {}", provider_name);
            return Ok(());
        }

        info!("📝 Extracting BM25 vocabulary...");

        // Get BM25 provider and extract vocabulary
        let bm25_provider = self.embedding_manager
            .get_provider(&crate::embedding::EmbeddingProviderType::BM25)
            .ok_or_else(|| {
                error!("❌ BM25 provider not found!");
                crate::error::VectorizerError::EmbeddingError(
                    "BM25 provider not found".to_string()
                )
            })?;
        
        let bm25 = bm25_provider
            .as_any()
            .downcast_ref::<crate::embedding::BM25Provider>()
            .ok_or_else(|| {
                error!("❌ Failed to downcast to BM25Provider!");
                crate::error::VectorizerError::EmbeddingError(
                    "Failed to downcast to BM25Provider".to_string()
                )
            })?;
        
        info!("✅ BM25 provider obtained, extracting data...");
        
        // Extract vocabulary synchronously
        let vocabulary = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                bm25.get_vocabulary().await
            })
        });
        
        let doc_freqs = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                bm25.get_document_frequencies().await
            })
        });
        
        let doc_count = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                bm25.document_count().await
            })
        });

        info!("📊 Vocabulary size: {}, doc_count: {}", vocabulary.len(), doc_count);

        // Create tokenizer JSON structure
        let tokenizer_data = serde_json::json!({
            "type": "BM25",
            "max_vocab_size": self.config.embedding_dimension,
            "vocabulary": vocabulary,
            "document_frequencies": doc_freqs,
            "document_count": doc_count,
            "version": "1.0"
        });

        // Ensure data directory exists
        if let Some(parent) = path.parent() {
            info!("📁 Ensuring parent directory exists: {}", parent.display());
            fs::create_dir_all(parent)
                .map_err(|e| {
                    error!("❌ Failed to create directory: {}", e);
                    crate::error::VectorizerError::IoError(e)
                })?;
        }

        info!("💾 Writing tokenizer file to: {}", path.display());
        // Write vocabulary file
        fs::write(path, serde_json::to_string_pretty(&tokenizer_data)?)
            .map_err(|e| {
                error!("❌ Failed to write tokenizer file: {}", e);
                crate::error::VectorizerError::IoError(e)
            })?;

        info!("✅ Saved BM25 vocabulary to {}: {} terms", path.display(), vocabulary.len());
        
        // Verify file was created
        if path.exists() {
            let metadata = fs::metadata(path)?;
            info!("✅ File verified: {} bytes", metadata.len());
        } else {
            error!("❌ CRITICAL: File was NOT created!");
        }
        
        Ok(())
    }
}
