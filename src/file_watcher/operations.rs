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
    hash_validator: Arc<crate::file_watcher::HashValidator>,
}

impl VectorOperations {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
        config: crate::file_watcher::FileWatcherConfig,
        hash_validator: Arc<crate::file_watcher::HashValidator>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
            config,
            hash_validator,
        }
    }

    /// Process file change event with intelligent lifecycle management
    pub async fn process_file_change(&self, event: &crate::file_watcher::FileChangeEventWithMetadata) -> Result<()> {
        tracing::info!("🔍 PROCESS: Processing file change event: {:?}", event.event);
        match &event.event {
            crate::file_watcher::FileChangeEvent::Created(path) => {
                // Skip events with empty paths (ignored events like Access)
                if path.as_os_str().is_empty() || path.to_string_lossy().is_empty() {
                    tracing::debug!("⏭️ PROCESS: Skipping event with empty path (ignored event): {:?}", path);
                    return Ok(());
                }
                
                tracing::info!("🔍 PROCESS: Processing CREATE event for file: {:?}", path);
                self.handle_file_created(path).await?;
                tracing::info!("✅ PROCESS: Successfully processed CREATE event for file: {:?}", path);
            }
            crate::file_watcher::FileChangeEvent::Modified(path) => {
                // Skip events with empty paths (ignored events like Access)
                if path.as_os_str().is_empty() || path.to_string_lossy().is_empty() {
                    tracing::debug!("⏭️ PROCESS: Skipping event with empty path (ignored event): {:?}", path);
                    return Ok(());
                }
                
                tracing::info!("🔍 PROCESS: Processing MODIFY event for file: {:?}", path);
                self.handle_file_modified(path).await?;
                tracing::info!("✅ PROCESS: Successfully processed MODIFY event for file: {:?}", path);
            }
            crate::file_watcher::FileChangeEvent::Deleted(path) => {
                tracing::info!("🔍 PROCESS: Processing DELETE event for file: {:?}", path);
                self.handle_file_deleted(path).await?;
                tracing::info!("✅ PROCESS: Successfully processed DELETE event for file: {:?}", path);
            }
            crate::file_watcher::FileChangeEvent::Renamed(old_path, new_path) => {
                tracing::info!("🔍 PROCESS: Processing RENAME event from {:?} to {:?}", old_path, new_path);
                self.handle_file_renamed(old_path, new_path).await?;
                tracing::info!("✅ PROCESS: Successfully processed RENAME event from {:?} to {:?}", old_path, new_path);
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

    /// Handle file created event
    async fn handle_file_created(&self, path: &std::path::Path) -> Result<()> {
        // Check if file should be processed
        if !self.should_process_file(path) {
            tracing::debug!("⏭️ CREATE: Skipping file (doesn't match patterns): {:?}", path);
            return Ok(());
        }
        
        // Check if file exists (sometimes CREATE events are sent for non-existent files)
        if !path.exists() {
            tracing::warn!("⚠️ CREATE: File does not exist, skipping: {:?}", path);
            return Ok(());
        }
        
        // Determine collection and index the file
        let collection_name = self.determine_collection_name(path);
        tracing::info!("🔍 CREATE: Indexing new file {:?} in collection '{}'", path, collection_name);
        
        self.index_file_from_path(path).await?;
        
        // Update hash after successful indexing
        if let Err(e) = self.hash_validator.update_hash(path).await {
            tracing::warn!("⚠️ CREATE: Failed to update hash for {:?}: {}", path, e);
        }
        
        Ok(())
    }
    
    /// Handle file modified event
    async fn handle_file_modified(&self, path: &std::path::Path) -> Result<()> {
        // Check if file should be processed
        if !self.should_process_file(path) {
            tracing::debug!("⏭️ MODIFY: Skipping file (doesn't match patterns): {:?}", path);
            return Ok(());
        }
        
        // Check if file exists
        if !path.exists() {
            tracing::warn!("⚠️ MODIFY: File does not exist, treating as DELETE: {:?}", path);
            return self.handle_file_deleted(path).await;
        }
        
        // Check if content has actually changed using hash comparison
        match self.hash_validator.has_content_changed(path).await {
            Ok(false) => {
                tracing::debug!("⏭️ MODIFY: File content unchanged, skipping: {:?}", path);
                return Ok(());
            }
            Ok(true) => {
                tracing::info!("🔍 MODIFY: File content changed, updating: {:?}", path);
            }
            Err(e) => {
                tracing::warn!("⚠️ MODIFY: Failed to check hash, proceeding with update: {:?} - {}", path, e);
            }
        }
        
        // Determine collection and update the file
        let collection_name = self.determine_collection_name(path);
        tracing::info!("🔍 MODIFY: Updating file {:?} in collection '{}'", path, collection_name);
        
        // Remove old version first, then add new version
        self.remove_file(&path.to_string_lossy(), &collection_name).await?;
        self.index_file_from_path(path).await?;
        
        // Update hash after successful indexing
        if let Err(e) = self.hash_validator.update_hash(path).await {
            tracing::warn!("⚠️ MODIFY: Failed to update hash for {:?}: {}", path, e);
        }
        
        Ok(())
    }
    
    /// Handle file deleted event
    async fn handle_file_deleted(&self, path: &std::path::Path) -> Result<()> {
        // Determine collection and remove the file
        let collection_name = self.determine_collection_name(path);
        tracing::info!("🔍 DELETE: Removing file {:?} from collection '{}'", path, collection_name);
        
        self.remove_file(&path.to_string_lossy(), &collection_name).await?;
        
        // Remove hash from cache
        self.hash_validator.remove_hash(path).await;
        
        Ok(())
    }
    
    /// Handle file renamed event
    async fn handle_file_renamed(&self, old_path: &std::path::Path, new_path: &std::path::Path) -> Result<()> {
        tracing::info!("🔍 RENAME: Renaming file from {:?} to {:?}", old_path, new_path);
        
        // Remove from old path
        self.handle_file_deleted(old_path).await?;
        
        // Add to new path (if it exists)
        if new_path.exists() {
            self.handle_file_created(new_path).await?;
        } else {
            tracing::warn!("⚠️ RENAME: New path does not exist: {:?}", new_path);
        }
        
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
        tracing::debug!("🔍 DETERMINE_COLLECTION: Determining collection for path: {:?}", path);
        // Try to load workspace configuration to determine collection name
        match self.load_workspace_collection_name(path) {
            Some(collection_name) => {
                tracing::info!("✅ DETERMINE_COLLECTION: Found collection '{}' for path: {:?}", collection_name, path);
                collection_name
            },
            None => {
                // Fallback to default collection name from config
                tracing::warn!("⚠️ DETERMINE_COLLECTION: No collection found, using fallback '{}' for path: {:?}", 
                    self.config.collection_name, path);
                self.config.collection_name.clone()
            }
        }
    }

    /// Load collection name from workspace configuration
    fn load_workspace_collection_name(&self, path: &std::path::Path) -> Option<String> {
        tracing::debug!("🔍 LOAD_WORKSPACE: Loading workspace config for path: {:?}", path);
        
        let workspace_file = std::env::current_dir()
            .ok()?
            .join("vectorize-workspace.yml");
        
        tracing::debug!("🔍 LOAD_WORKSPACE: Workspace file: {:?}", workspace_file);
        
        if !workspace_file.exists() {
            tracing::warn!("⚠️ LOAD_WORKSPACE: Workspace file does not exist: {:?}", workspace_file);
            return None;
        }
        
        let content = std::fs::read_to_string(&workspace_file).ok()?;
        let workspace: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
        
        let path_str = path.to_string_lossy();
        tracing::debug!("🔍 LOAD_WORKSPACE: File path string: {}", path_str);
        
        // Check projects and their collections
        if let Some(projects) = workspace.get("projects") {
            if let Some(projects_array) = projects.as_sequence() {
                for project in projects_array {
                    if let Some(project_path) = project.get("path") {
                        if let Some(project_path_str) = project_path.as_str() {
                            // Resolve relative paths to absolute paths
                            let absolute_project_path = if project_path_str.starts_with("..") || project_path_str.starts_with(".") {
                                std::env::current_dir()
                                    .ok()?
                                    .join(project_path_str)
                                    .canonicalize()
                                    .ok()?
                            } else {
                                std::path::PathBuf::from(project_path_str)
                            };
                            
                            let absolute_project_path_str = absolute_project_path.to_string_lossy();
                            
                            // Check if the file path starts with the project path
                            if path_str.starts_with(absolute_project_path_str.as_ref()) {
                                if let Some(collections) = project.get("collections") {
                                    if let Some(collections_array) = collections.as_sequence() {
                                        // Sort collections by specificity (more specific patterns first)
                                        let mut sorted_collections = Vec::new();
                                        for collection in collections_array {
                                            if let Some(collection_name) = collection.get("name") {
                                                if let Some(collection_name_str) = collection_name.as_str() {
                                                    if let Some(include_patterns) = collection.get("include_patterns") {
                                                        if let Some(patterns_array) = include_patterns.as_sequence() {
                                                            for pattern in patterns_array {
                                                                if let Some(pattern_str) = pattern.as_str() {
                                                                    let specificity = self.calculate_pattern_specificity(pattern_str);
                                                                    sorted_collections.push((specificity, collection_name_str, pattern_str));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Sort by specificity (higher specificity first)
                                        sorted_collections.sort_by(|a, b| b.0.cmp(&a.0));
                                        
                                        // Try patterns in order of specificity
                                        for (_, collection_name_str, pattern_str) in sorted_collections {
                                            // Get the relative path within the project
                                            let relative_path = path_str.strip_prefix(absolute_project_path_str.as_ref())
                                                .unwrap_or(&path_str)
                                                .trim_start_matches('/');
                                            
                                            if self.matches_pattern(relative_path, pattern_str) {
                                                tracing::info!("✅ COLLECTION: Matched file {:?} to collection {:?} with pattern {:?}", 
                                                    path, collection_name_str, pattern_str);
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
        
        tracing::warn!("⚠️ LOAD_WORKSPACE: No collection found for path: {:?}", path);
        None
    }

    /// Calculate pattern specificity (higher = more specific)
    fn calculate_pattern_specificity(&self, pattern: &str) -> u32 {
        let mut specificity = 0;
        
        // More specific patterns get higher scores
        if pattern.contains("**") {
            // Double wildcard is less specific
            specificity += 10;
        } else if pattern.contains("*") {
            // Single wildcard is more specific
            specificity += 20;
        } else {
            // Exact match is most specific
            specificity += 50;
        }
        
        // Count directory separators (more specific paths)
        specificity += pattern.matches('/').count() as u32 * 5;
        
        // Count literal characters (more specific)
        specificity += pattern.chars().filter(|c| !matches!(c, '*')).count() as u32;
        
        // Bonus for specific file extensions
        if pattern.ends_with(".md") {
            specificity += 10;
        }
        
        specificity
    }

    /// Check if a path matches a glob pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        tracing::debug!("🔍 PATTERN: Checking if '{}' matches pattern '{}'", path, pattern);
        
        // Handle ** patterns (recursive directory matching)
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                
                // For ** patterns, check if path starts with prefix and ends with suffix
                // and contains the directory structure in between
                let matches = if prefix.is_empty() && suffix.is_empty() {
                    true // ** matches everything
                } else if prefix.is_empty() {
                    path.ends_with(suffix) // **/suffix
                } else if suffix.is_empty() {
                    path.starts_with(prefix) // prefix/**
                } else {
                    // prefix/**/suffix - check if path starts with prefix and ends with suffix
                    path.starts_with(prefix) && path.ends_with(suffix)
                };
                
                tracing::debug!("🔍 PATTERN: ** pattern '{}' -> prefix: '{}', suffix: '{}', matches: {}", 
                    pattern, prefix, suffix, matches);
                return matches;
            }
        } else if pattern.contains("*") {
            // Handle single * patterns
            let parts: Vec<&str> = pattern.split("*").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                let matches = path.starts_with(prefix) && path.ends_with(suffix);
                tracing::debug!("🔍 PATTERN: * pattern '{}' -> prefix: '{}', suffix: '{}', matches: {}", 
                    pattern, prefix, suffix, matches);
                return matches;
            }
        } else {
            // Exact match or contains match
            let matches = path.contains(pattern);
            tracing::debug!("🔍 PATTERN: exact pattern '{}' matches: {}", pattern, matches);
            return matches;
        }
        
        tracing::debug!("🔍 PATTERN: no match for pattern '{}'", pattern);
        false
    }
}
