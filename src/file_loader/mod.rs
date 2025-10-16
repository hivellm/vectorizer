//! Unified file loading, indexing, and persistence module
//!
//! Thin orchestrator that uses existing embedding, persistence, and storage modules

pub mod chunker;
pub mod config;
pub mod indexer;
pub mod persistence;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub use chunker::Chunker;
pub use config::{DocumentChunk, LoaderConfig};
use glob::Pattern;
pub use indexer::Indexer;
pub use persistence::Persistence;
use tracing::{info, warn};

use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Thin file loader orchestrator - uses existing infrastructure
pub struct FileLoader {
    config: LoaderConfig,
    chunker: Chunker,
    indexer: Indexer,
    persistence: Persistence,
}

impl FileLoader {
    /// Create with shared embedding manager (uses existing infrastructure)
    pub fn with_embedding_manager(
        config: LoaderConfig,
        embedding_manager: EmbeddingManager,
    ) -> Self {
        let chunker = Chunker::new(config.clone());
        let indexer = Indexer::with_embedding_manager(config.clone(), embedding_manager);
        let persistence = Persistence::new();

        Self {
            config,
            chunker,
            indexer,
            persistence,
        }
    }

    /// Load and index a project (main entry point)
    pub async fn load_and_index_project(
        &mut self,
        project_path: &str,
        store: &VectorStore,
    ) -> Result<usize> {
        let collection_name = &self.config.collection_name;

        // FAST PATH: Check if collection already exists in .vecdb
        if self.persistence.collection_exists_in_vecdb(collection_name) {
            info!(
                "Collection '{}' found in .vecdb, skipping indexing",
                collection_name
            );
            return Ok(0); // No new indexing
        }

        info!(
            "Starting indexing for collection '{}' from {}",
            collection_name, project_path
        );

        // Step 1: Collect documents
        let documents = self.collect_documents_sync(project_path)?;

        if documents.is_empty() {
            warn!("No documents found for collection '{}'", collection_name);
            return Ok(0);
        }

        info!(
            "Found {} documents for collection '{}'",
            documents.len(),
            collection_name
        );

        // Step 2: Chunk documents
        let chunks = self.chunker.chunk_documents(&documents)?;
        info!(
            "Created {} chunks for collection '{}'",
            chunks.len(),
            collection_name
        );

        // Step 3: Build vocabulary
        self.indexer.build_vocabulary(&documents)?;

        // Step 4: Create collection
        self.indexer.create_collection(store)?;

        // Step 5: Store vectors
        let vector_count = self.indexer.store_chunks_parallel(store, &chunks)?;

        // Step 6: Save to temporary format (will be compacted in batch later)
        self.save_collection_temp(store)?;

        // Step 7: Save tokenizer/vocabulary for file watcher
        self.save_tokenizer()?;

        info!(
            "Indexed {} vectors for collection '{}'",
            vector_count, collection_name
        );
        Ok(vector_count)
    }

    /// Collect documents from project directory (sync - just filesystem I/O)
    fn collect_documents_sync(&self, project_path: &str) -> Result<Vec<(PathBuf, String)>> {
        let path = Path::new(project_path);
        let mut documents = Vec::new();
        self.collect_documents_recursive(path, path, &mut documents)?;
        Ok(documents)
    }

    /// Recursively collect documents from directory (sync)
    fn collect_documents_recursive(
        &self,
        dir: &Path,
        project_root: &Path,
        documents: &mut Vec<(PathBuf, String)>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip excluded directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                        || dir_name == "data"
                        || dir_name == "__pycache__"
                        || dir_name == "dist"
                        || dir_name == "build"
                    {
                        continue;
                    }
                }
                self.collect_documents_recursive(&path, project_root, documents)?;
            } else if path.is_file() {
                if self.matches_patterns(&path, project_root) {
                    // Check file size
                    if let Ok(metadata) = fs::metadata(&path) {
                        if metadata.len() > self.config.max_file_size as u64 {
                            continue;
                        }
                    }

                    // Read file content
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            let normalized_content = content.replace("\r\n", "\n");
                            documents.push((path.clone(), normalized_content));
                        }
                        Err(e) => {
                            warn!("Failed to read file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if file matches include/exclude patterns
    fn matches_patterns(&self, file_path: &Path, project_root: &Path) -> bool {
        // Safety checks - never process these
        let path_str = file_path.to_string_lossy();
        if path_str.contains("/data/") || path_str.contains("\\data\\") {
            return false;
        }
        if file_path.extension().and_then(|e| e.to_str()) == Some("bin") {
            return false;
        }

        // Get relative path
        let relative_path = match file_path.strip_prefix(project_root) {
            Ok(rel) => rel,
            Err(_) => return false,
        };

        let path_str = relative_path.to_string_lossy();

        // Check exclude patterns first
        for exclude_pattern in &self.config.exclude_patterns {
            if let Ok(pattern) = Pattern::new(exclude_pattern) {
                if pattern.matches(&path_str) {
                    return false;
                }
            }
        }

        // Check include patterns
        for include_pattern in &self.config.include_patterns {
            if let Ok(pattern) = Pattern::new(include_pattern) {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }

    /// Save collection to temporary format (for batch compaction)
    pub fn save_collection_temp(&self, store: &VectorStore) -> Result<()> {
        self.persistence
            .save_collection_legacy_temp(store, &self.config.collection_name)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Save tokenizer/vocabulary for file watcher
    pub fn save_tokenizer(&self) -> Result<()> {
        let data_dir = std::path::PathBuf::from("./data");
        let tokenizer_path =
            data_dir.join(format!("{}_tokenizer.json", self.config.collection_name));

        // Get the embedding type from config
        let embedding_type = &self.config.embedding_type;

        // Save vocabulary using the indexer's embedding manager
        self.indexer
            .save_vocabulary(&tokenizer_path, embedding_type)
            .map_err(|e| anyhow::anyhow!("Failed to save vocabulary: {}", e))
    }

    /// Compact all collections to .vecdb
    pub fn compact_all(&self) -> Result<usize> {
        self.persistence
            .compact_and_cleanup()
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
