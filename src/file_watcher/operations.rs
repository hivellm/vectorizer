//! Vector operations for file watcher

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{VectorStore, embedding::EmbeddingManager, document_loader::{DocumentLoader, LoaderConfig}};
use crate::error::{Result, VectorizerError};

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
    pub async fn process_file_change(&self, event: &crate::file_watcher::FileChangeEventWithMetadata) -> Result<()> {
        tracing::info!("🔍 PROCESS: Processing file change event: {:?}", event.event);
        match &event.event {
            crate::file_watcher::FileChangeEvent::Created(path) | crate::file_watcher::FileChangeEvent::Modified(path) => {
                // Skip events with empty paths (ignored events like Access)
                if path.as_os_str().is_empty() || path.to_string_lossy().is_empty() {
                    tracing::debug!("⏭️ PROCESS: Skipping event with empty path (ignored event): {:?}", path);
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
                tracing::info!("🔍 PROCESS: Renaming file from {:?} to {:?}", old_path, new_path);
                // Remove from old path and add to new path
                self.remove_file_from_path(old_path).await?;
                self.index_file_from_path(new_path).await?;
                tracing::info!("✅ PROCESS: Successfully renamed file from {:?} to {:?}", old_path, new_path);
            }
        }
        Ok(())
    }

    /// Index file content using DocumentLoader
    pub async fn index_file(&self, file_path: &str, content: &str, collection_name: &str) -> Result<()> {
        // Create a temporary directory to process with DocumentLoader
        let temp_dir = std::env::temp_dir().join(format!("temp_dir_{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await
            .map_err(|e| VectorizerError::IoError(e))?;
        
        // Write content to temporary file
        let temp_file_path = temp_dir.join(std::path::Path::new(file_path).file_name().unwrap_or(std::ffi::OsStr::new("temp_file")));
        tokio::fs::write(&temp_file_path, content).await
            .map_err(|e| VectorizerError::IoError(e))?;
        
        // Create LoaderConfig for this file
        let loader_config = LoaderConfig {
            max_chunk_size: 2048,
            chunk_overlap: 256,
            allowed_extensions: vec![],
            include_patterns: vec!["*".to_string()], // Include all files
            exclude_patterns: vec![],
            embedding_dimension: 512, // Default dimension
            embedding_type: "bm25".to_string(),
            collection_name: collection_name.to_string(),
            max_file_size: 10 * 1024 * 1024, // 10MB
        };
        
        // Create DocumentLoader and process the file
        let mut loader = DocumentLoader::new(loader_config);
        
        // Process the file using load_project_async
        match loader.load_project_async(&temp_dir.to_string_lossy(), &self.vector_store).await {
            Ok(_) => {
                tracing::info!("Successfully indexed file: {} in collection: {}", file_path, collection_name);
            },
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
        self.vector_store.delete(collection_name, file_path)?;
        
        tracing::info!("Removed file: {} from collection: {}", file_path, collection_name);
        Ok(())
    }

    /// Update file in index
    pub async fn update_file(&self, file_path: &str, content: &str, collection_name: &str) -> Result<()> {
        // For now, just re-index the file (remove and add again)
        self.remove_file(file_path, collection_name).await?;
        self.index_file(file_path, content, collection_name).await?;
        
        tracing::info!("Updated file: {} in collection: {}", file_path, collection_name);
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

        // For large/binary-safe handling, avoid read_to_string; copy file to temp dir
        // and process via DocumentLoader
        let temp_dir = std::env::temp_dir().join(format!("temp_single_{}", uuid::Uuid::new_v4()));
        if let Err(e) = tokio::fs::create_dir_all(&temp_dir).await {
            tracing::warn!("Failed to create temp dir {:?}: {}", temp_dir, e);
            return Ok(());
        }

        // Destination file keeps original filename
        let dest_file = temp_dir.join(path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("file")));
        match tokio::fs::copy(path, &dest_file).await {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Failed to copy file {:?} to temp {:?}: {}", path, dest_file, e);
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                return Ok(());
            }
        }

        // Build loader config
        let loader_config = LoaderConfig {
            max_chunk_size: 2048,
            chunk_overlap: 256,
            allowed_extensions: vec![],
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: collection_name.clone(),
            max_file_size: self.config.max_file_size as usize,
        };

        let mut loader = DocumentLoader::new(loader_config);
        match loader.load_project_async(&temp_dir.to_string_lossy(), &self.vector_store).await {
            Ok(_) => {
                tracing::info!("Successfully indexed file via temp copy: {:?}", path);
            }
            Err(e) => {
                tracing::warn!("Failed to index file via temp copy {:?}: {}", path, e);
            }
        }

        // Cleanup temp dir
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;

        Ok(())
    }

    /// Remove file from index by path
    async fn remove_file_from_path(&self, path: &std::path::Path) -> Result<()> {
        let collection_name = self.determine_collection_name(path);
        self.remove_file(&path.to_string_lossy(), &collection_name).await?;
        Ok(())
    }

    /// Check if file should be processed based on patterns
    pub fn should_process_file(&self, path: &std::path::Path) -> bool {
        // Use the configuration to check if file should be processed
        self.config.should_process_file(path)
    }

    /// Determine collection name based on file path
    pub fn determine_collection_name(&self, path: &std::path::Path) -> String {
        // Try to load workspace configuration to determine collection name
        match self.load_workspace_collection_name(path) {
            Some(collection_name) => collection_name,
            None => {
                // Fallback to default collection name from config
                self.config.collection_name.clone()
            }
        }
    }

    /// Load collection name from workspace configuration
    fn load_workspace_collection_name(&self, path: &std::path::Path) -> Option<String> {
        let workspace_file = std::env::current_dir()
            .ok()?
            .join("vectorize-workspace.yml");
        
        if !workspace_file.exists() {
            return None;
        }
        
        let content = std::fs::read_to_string(&workspace_file).ok()?;
        let workspace: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
        
        let path_str = path.to_string_lossy();
        
        // Check projects and their collections
        if let Some(projects) = workspace.get("projects") {
            if let Some(projects_array) = projects.as_sequence() {
                for project in projects_array {
                    if let Some(project_path) = project.get("path") {
                        if let Some(project_path_str) = project_path.as_str() {
                            if path_str.contains(project_path_str) {
                                if let Some(collections) = project.get("collections") {
                                    if let Some(collections_array) = collections.as_sequence() {
                                        for collection in collections_array {
                                            if let Some(collection_name) = collection.get("name") {
                                                if let Some(collection_name_str) = collection_name.as_str() {
                                                    // Check if file matches collection patterns
                                                    if let Some(include_patterns) = collection.get("include_patterns") {
                                                        if let Some(patterns_array) = include_patterns.as_sequence() {
                                                            for pattern in patterns_array {
                                                                if let Some(pattern_str) = pattern.as_str() {
                                                                    if self.matches_pattern(&path_str, pattern_str) {
                                                                        return Some(collection_name_str.to_string());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Check if a path matches a glob pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple pattern matching - in a real implementation, you'd use a proper glob library
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        } else if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split("*").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return path.starts_with(prefix) && path.ends_with(suffix);
            }
        } else {
            return path.contains(pattern);
        }
        
        false
    }
}
