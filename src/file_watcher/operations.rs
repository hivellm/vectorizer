//! Vector operations for file watcher

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::error::{Result, VectorizerError};
use crate::file_loader::{FileLoader, LoaderConfig};

/// Vector operations for file watcher
pub struct VectorOperations {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    config: crate::file_watcher::FileWatcherConfig,
}

impl VectorOperations {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
        config: crate::file_watcher::FileWatcherConfig,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
            config,
        }
    }

    /// Process file change event
    pub async fn process_file_change(
        &self,
        event: &crate::file_watcher::FileChangeEventWithMetadata,
    ) -> Result<()> {
        tracing::info!(
            "🔍 PROCESS: Processing file change event: {:?}",
            event.event
        );
        match &event.event {
            crate::file_watcher::FileChangeEvent::Created(path)
            | crate::file_watcher::FileChangeEvent::Modified(path) => {
                // Skip events with empty paths (ignored events like Access)
                if path.as_os_str().is_empty() || path.to_string_lossy().is_empty() {
                    tracing::debug!(
                        "⏭️ PROCESS: Skipping event with empty path (ignored event): {:?}",
                        path
                    );
                    return Ok(());
                }

                tracing::info!("🔍 PROCESS: Indexing file: {:?}", path);
                self.index_file_from_path(path).await?;
                tracing::info!("✅ PROCESS: Successfully indexed file: {:?}", path);
            }
            crate::file_watcher::FileChangeEvent::Deleted(path) => {
                tracing::info!("🔍 PROCESS: Removing file: {:?}", path);
                self.remove_file_from_path(path).await?;
                tracing::info!("✅ PROCESS: Successfully removed file: {:?}", path);
            }
            crate::file_watcher::FileChangeEvent::Renamed(old_path, new_path) => {
                tracing::info!(
                    "🔍 PROCESS: Renaming file from {:?} to {:?}",
                    old_path,
                    new_path
                );
                // Remove from old path and add to new path
                self.remove_file_from_path(old_path).await?;
                self.index_file_from_path(new_path).await?;
                tracing::info!(
                    "✅ PROCESS: Successfully renamed file from {:?} to {:?}",
                    old_path,
                    new_path
                );
            }
        }
        Ok(())
    }

    /// Index file content using DocumentLoader
    pub async fn index_file(
        &self,
        file_path: &str,
        content: &str,
        collection_name: &str,
    ) -> Result<()> {
        // Create a temporary directory to process with DocumentLoader
        let temp_dir = std::env::temp_dir().join(format!("temp_dir_{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir)
            .await
            .map_err(|e| VectorizerError::IoError(e))?;

        // Write content to temporary file
        let temp_file_path = temp_dir.join(
            std::path::Path::new(file_path)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("temp_file")),
        );
        tokio::fs::write(&temp_file_path, content)
            .await
            .map_err(|e| VectorizerError::IoError(e))?;

        // Create LoaderConfig for this file
        let mut loader_config = LoaderConfig {
            max_chunk_size: 2048,
            chunk_overlap: 256,
            include_patterns: vec!["*".to_string()], // Include all files
            exclude_patterns: vec![],
            embedding_dimension: 512, // Default dimension
            embedding_type: "bm25".to_string(),
            collection_name: collection_name.to_string(),
            max_file_size: 10 * 1024 * 1024, // 10MB
        };

        // CRITICAL: Always enforce hardcoded exclusions (Python cache, binaries, etc.)
        loader_config.ensure_hardcoded_excludes();

        // Create FileLoader and process the file
        let embedding_manager = {
            let guard = self.embedding_manager.read().await;
            // Create new embedding manager for this operation
            let mut em = EmbeddingManager::new();
            let bm25 = crate::embedding::Bm25Embedding::new(512);
            em.register_provider("bm25".to_string(), Box::new(bm25));
            // SAFE: `bm25` was just registered above, so set_default cannot
            // fail with `ProviderNotFound`.
            #[allow(clippy::unwrap_used)]
            em.set_default_provider("bm25").unwrap();
            em
        };
        let mut loader = FileLoader::with_embedding_manager(loader_config, embedding_manager);

        // Process the file
        match loader
            .load_and_index_project(&temp_dir.to_string_lossy(), &self.vector_store)
            .await
        {
            Ok(_) => {
                tracing::info!(
                    "Successfully indexed file: {} in collection: {}",
                    file_path,
                    collection_name
                );
            }
            Err(e) => {
                tracing::warn!("Failed to index file {}: {}", file_path, e);
                return Err(VectorizerError::IndexError(e.to_string()));
            }
        }

        // Clean up temporary directory
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;

        Ok(())
    }

    /// Remove file from index
    pub async fn remove_file(&self, file_path: &str, collection_name: &str) -> Result<()> {
        // Remove vector by ID (file path)
        // If vector not found, treat as warning (file may have been deleted already)
        match self.vector_store.delete(collection_name, file_path) {
            Ok(_) => {
                tracing::info!(
                    "Removed file: {} from collection: {}",
                    file_path,
                    collection_name
                );
            }
            Err(crate::error::VectorizerError::VectorNotFound(_)) => {
                tracing::debug!(
                    "File already removed (not found in vector store): {} from collection: {}",
                    file_path,
                    collection_name
                );
                // This is fine - file may have been deleted already or never indexed
            }
            Err(e) => {
                // Other errors should still be propagated
                return Err(e);
            }
        }
        Ok(())
    }

    /// Update file in index
    pub async fn update_file(
        &self,
        file_path: &str,
        content: &str,
        collection_name: &str,
    ) -> Result<()> {
        // For now, just re-index the file (remove and add again)
        self.remove_file(file_path, collection_name).await?;
        self.index_file(file_path, content, collection_name).await?;

        tracing::info!(
            "Updated file: {} in collection: {}",
            file_path,
            collection_name
        );
        Ok(())
    }

    /// Index file from path
    pub async fn index_file_from_path(&self, path: &std::path::Path) -> Result<()> {
        // Check if file should be processed
        if !self.should_process_file(path) {
            tracing::debug!("Skipping file (doesn't match patterns): {:?}", path);
            return Ok(());
        }

        // Determine collection name based on file path
        let collection_name = self.determine_collection_name(path);

        // Store the original path for metadata
        let original_path = path.to_path_buf();
        let original_path_str = original_path.to_string_lossy().to_string();

        // Read file content directly (more efficient for single files)
        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                // For binary files or encoding issues, skip silently
                tracing::debug!("Skipping file {:?} (cannot read as text): {}", path, e);
                return Ok(());
            }
        };

        // Check file size
        if content.len() > self.config.max_file_size as usize {
            tracing::debug!(
                "Skipping file {:?} (too large: {} bytes)",
                path,
                content.len()
            );
            return Ok(());
        }

        // Create embedding manager
        let embedding_manager = {
            let mut em = EmbeddingManager::new();
            let bm25 = crate::embedding::Bm25Embedding::new(512);
            em.register_provider("bm25".to_string(), Box::new(bm25));
            // SAFE: `bm25` was just registered above, so set_default cannot
            // fail with `ProviderNotFound`.
            #[allow(clippy::unwrap_used)]
            em.set_default_provider("bm25").unwrap();
            em
        };

        // Get file extension for metadata
        let file_extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Ensure collection exists
        if !self
            .vector_store
            .list_collections()
            .contains(&collection_name)
        {
            let config = crate::models::CollectionConfig::default();
            if let Err(e) = self
                .vector_store
                .create_collection(&collection_name, config)
            {
                tracing::warn!("Failed to create collection {}: {}", collection_name, e);
                return Ok(());
            }
        }

        // Chunk the content manually (simple approach)
        let max_chunk_size = 2048;
        let chunk_overlap = 256;
        let mut chunks: Vec<String> = Vec::new();
        let mut start = 0;

        while start < content.len() {
            let mut end = std::cmp::min(start + max_chunk_size, content.len());

            // Ensure we're at a UTF-8 character boundary
            while end > start && !content.is_char_boundary(end) {
                end -= 1;
            }

            // Try to break at a word boundary
            if end < content.len() {
                if let Some(pos) =
                    content[start..end].rfind(|c: char| c.is_whitespace() || c == '.' || c == '\n')
                {
                    let new_end = start + pos + 1;
                    if content.is_char_boundary(new_end) {
                        end = new_end;
                    }
                }
            }

            let chunk_text = content[start..end].trim();
            if !chunk_text.is_empty() {
                chunks.push(chunk_text.to_string());
            }

            // Move start with overlap
            start = if end >= content.len() {
                content.len()
            } else {
                end.saturating_sub(chunk_overlap)
            };
        }

        // Build vectors to insert
        let mut vectors_to_insert: Vec<crate::models::Vector> = Vec::new();

        // Index each chunk with ORIGINAL path in metadata
        for (chunk_idx, chunk_content) in chunks.iter().enumerate() {
            // Generate embedding
            let embedding = match embedding_manager.embed(chunk_content) {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Failed to embed chunk: {}", e);
                    continue;
                }
            };

            // Create payload data with ORIGINAL file path (not temp path)
            let payload_data = serde_json::json!({
                "content": chunk_content,
                "file_path": original_path_str,
                "file_extension": file_extension,
                "chunk_index": chunk_idx,
                "chunk_size": chunk_content.len()
            });

            // Generate unique ID
            let vector_id = format!("{}_{}", original_path_str, chunk_idx);

            // Create vector with payload
            let payload = crate::models::Payload { data: payload_data };

            let vector = crate::models::Vector {
                id: vector_id,
                data: embedding,
                sparse: None,
                payload: Some(payload),
                document_id: None,
            };

            vectors_to_insert.push(vector);
        }

        // Insert all vectors at once (filter out duplicates first)
        if !vectors_to_insert.is_empty() {
            // Filter out vectors that already exist to avoid duplicate errors
            let mut vectors_to_insert_filtered = Vec::new();
            for vector in vectors_to_insert {
                // Check if vector already exists
                match self.vector_store.get_vector(&collection_name, &vector.id) {
                    Ok(_) => {
                        // Vector already exists, skip it
                        tracing::debug!(
                            "Skipping duplicate vector '{}' for file {:?}",
                            vector.id,
                            original_path
                        );
                    }
                    Err(_) => {
                        // Vector doesn't exist, add it
                        vectors_to_insert_filtered.push(vector);
                    }
                }
            }

            if !vectors_to_insert_filtered.is_empty() {
                let chunk_count = vectors_to_insert_filtered.len();
                if let Err(e) = self
                    .vector_store
                    .insert(&collection_name, vectors_to_insert_filtered)
                {
                    tracing::warn!("Failed to insert vectors for {:?}: {}", original_path, e);
                    return Ok(());
                }

                tracing::info!(
                    "Successfully indexed file {:?} with {} chunks into collection '{}'",
                    original_path,
                    chunk_count,
                    collection_name
                );
            } else {
                tracing::debug!(
                    "All vectors for file {:?} already exist, skipping insertion",
                    original_path
                );
            }
        }

        Ok(())
    }

    /// Remove file from index by path
    async fn remove_file_from_path(&self, path: &std::path::Path) -> Result<()> {
        let collection_name = self.determine_collection_name(path);
        self.remove_file(&path.to_string_lossy(), &collection_name)
            .await?;
        Ok(())
    }

    /// Check if file should be processed based on patterns
    pub fn should_process_file(&self, path: &std::path::Path) -> bool {
        // Use the configuration to check if file should be processed
        self.config.should_process_file(path)
    }

    /// Determine collection name based on file path
    ///
    /// Priority order:
    /// 1. Check collection mapping from config (YAML collection_mapping patterns)
    /// 2. Match known project patterns from workspace.yml
    /// 3. Use configured default collection (NO automatic path-based generation)
    ///
    /// This prevents the aggressive automatic creation of empty collections
    /// that was happening with path-based name generation.
    pub fn determine_collection_name(&self, path: &std::path::Path) -> String {
        // PRIORITY 1: Check collection mapping from config (YAML collection_mapping patterns)
        if let Some(collection) = self.config.get_collection_for_path(path) {
            return collection;
        }

        // PRIORITY 2: Try to match against known project patterns from workspace.yml
        let path_str = path.to_string_lossy();

        if path_str.contains("/docs/") {
            if path_str.contains("/architecture/") {
                "docs-architecture".to_string()
            } else if path_str.contains("/templates/") {
                "docs-templates".to_string()
            } else if path_str.contains("/processes/") {
                "docs-processes".to_string()
            } else if path_str.contains("/governance/") {
                "docs-governance".to_string()
            } else if path_str.contains("/navigation/") {
                "docs-navigation".to_string()
            } else if path_str.contains("/testing/") {
                "docs-testing".to_string()
            } else {
                "docs-architecture".to_string() // Default docs collection
            }
        } else if path_str.contains("/vectorizer/") {
            if path_str.contains("/docs/") {
                "vectorizer-docs".to_string()
            } else if path_str.contains("/src/") {
                "vectorizer-source".to_string()
            } else if path_str.contains("/client-sdks/") || path_str.contains("/sdks/") {
                if path_str.contains(".ts") || path_str.contains(".js") {
                    "vectorizer-sdk-typescript".to_string()
                } else if path_str.contains(".py") {
                    "vectorizer-sdk-python".to_string()
                } else if path_str.contains(".rs") {
                    "vectorizer-sdk-rust".to_string()
                } else {
                    "vectorizer-source".to_string()
                }
            } else {
                "vectorizer-source".to_string()
            }
        } else if path_str.contains("/gov/") {
            if path_str.contains("/bips/") {
                "gov-bips".to_string()
            } else if path_str.contains("/guidelines/") {
                "gov-guidelines".to_string()
            } else if path_str.contains("/proposals/") {
                "gov-proposals".to_string()
            } else if path_str.contains("/minutes/") {
                "gov-minutes".to_string()
            } else if path_str.contains("/schemas/") {
                "gov-schemas".to_string()
            } else if path_str.contains("/teams/") {
                "gov-teams".to_string()
            } else if path_str.contains("/metrics/") {
                "gov-metrics".to_string()
            } else if path_str.contains("/issues/") {
                "gov-issues".to_string()
            } else if path_str.contains("/snapshot/") {
                "gov-snapshot".to_string()
            } else {
                "gov-core".to_string()
            }
        } else {
            // PRIORITY 3: Use configured default collection
            // NO automatic path-based generation to prevent empty collection proliferation
            self.config
                .default_collection
                .clone()
                .unwrap_or_else(|| "workspace-default".to_string())
        }
    }

    /// Get collection name for a file path based on collection mapping patterns
    ///
    /// This is a convenience method that delegates to the config's get_collection_for_path.
    pub fn get_collection_for_path(&self, path: &std::path::Path) -> Option<String> {
        self.config.get_collection_for_path(path)
    }
}

#[cfg(test)]
#[path = "operations_tests.rs"]
mod tests;
