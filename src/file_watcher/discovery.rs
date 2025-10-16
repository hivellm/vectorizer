//! File Discovery System for initial file scanning and indexing
//!
//! This module provides functionality to discover and index existing files
//! in the workspace during startup, ensuring all relevant files are indexed
//! without requiring manual modification.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::VectorStore;
use crate::file_watcher::{EmbeddingManager, FileWatcherConfig, VectorOperations};

/// File discovery system for initial scanning
pub struct FileDiscovery {
    config: FileWatcherConfig,
    vector_operations: Arc<VectorOperations>,
    vector_store: Arc<VectorStore>,
}

/// Statistics for file discovery operations
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    pub total_files_scanned: usize,
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub files_errors: usize,
    pub directories_scanned: usize,
    pub processing_time_ms: u64,
}

/// Result of file discovery operation
#[derive(Debug)]
pub struct DiscoveryResult {
    pub stats: DiscoveryStats,
    pub indexed_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub error_files: Vec<(PathBuf, String)>,
}

impl FileDiscovery {
    /// Create a new file discovery system
    pub fn new(
        config: FileWatcherConfig,
        vector_operations: Arc<VectorOperations>,
        vector_store: Arc<VectorStore>,
    ) -> Self {
        Self {
            config,
            vector_operations,
            vector_store,
        }
    }

    /// Discover and index all existing files in configured paths
    pub async fn discover_existing_files(
        &self,
    ) -> Result<DiscoveryResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        info!("🔍 Starting file discovery for existing files...");

        // Add timeout to prevent infinite hanging
        const DISCOVERY_TIMEOUT_SECONDS: u64 = 15; // Reduced timeout for faster recovery
        let discovery_result = tokio::time::timeout(
            std::time::Duration::from_secs(DISCOVERY_TIMEOUT_SECONDS),
            self.perform_discovery(),
        )
        .await;

        match discovery_result {
            Ok(result) => result,
            Err(_) => {
                warn!(
                    "⚠️ File discovery timed out after {} seconds",
                    DISCOVERY_TIMEOUT_SECONDS
                );
                Ok(DiscoveryResult {
                    stats: DiscoveryStats {
                        total_files_scanned: 0,
                        files_indexed: 0,
                        files_skipped: 0,
                        files_errors: 1,
                        directories_scanned: 0,
                        processing_time_ms: DISCOVERY_TIMEOUT_SECONDS * 1000,
                    },
                    indexed_files: Vec::new(),
                    skipped_files: Vec::new(),
                    error_files: vec![(std::path::PathBuf::new(), "Discovery timeout".to_string())],
                })
            }
        }
    }

    /// Perform the actual discovery without timeout
    async fn perform_discovery(
        &self,
    ) -> Result<DiscoveryResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Get watch paths from config or discover from workspace
        let watch_paths = self.get_watch_paths().await?;

        if watch_paths.is_empty() {
            warn!("No watch paths found for file discovery");
            return Ok(DiscoveryResult {
                stats: DiscoveryStats {
                    total_files_scanned: 0,
                    files_indexed: 0,
                    files_skipped: 0,
                    files_errors: 0,
                    directories_scanned: 0,
                    processing_time_ms: 0,
                },
                indexed_files: Vec::new(),
                skipped_files: Vec::new(),
                error_files: Vec::new(),
            });
        }

        info!(
            "📁 Scanning {} directories for existing files",
            watch_paths.len()
        );

        let mut stats = DiscoveryStats {
            total_files_scanned: 0,
            files_indexed: 0,
            files_skipped: 0,
            files_errors: 0,
            directories_scanned: watch_paths.len(),
            processing_time_ms: 0,
        };

        let mut indexed_files = Vec::new();
        let mut skipped_files = Vec::new();
        let mut error_files = Vec::new();

        // Process each watch path
        for watch_path in &watch_paths {
            info!("📂 Scanning directory: {:?}", watch_path);

            match self.scan_directory(watch_path).await {
                Ok(mut dir_result) => {
                    stats.total_files_scanned += dir_result.stats.total_files_scanned;
                    stats.files_indexed += dir_result.stats.files_indexed;
                    stats.files_skipped += dir_result.stats.files_skipped;
                    stats.files_errors += dir_result.stats.files_errors;

                    indexed_files.append(&mut dir_result.indexed_files);
                    skipped_files.append(&mut dir_result.skipped_files);
                    error_files.append(&mut dir_result.error_files);
                }
                Err(e) => {
                    error!("Failed to scan directory {:?}: {}", watch_path, e);
                    stats.files_errors += 1;
                    error_files.push((watch_path.clone(), e.to_string()));
                }
            }
        }

        stats.processing_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "✅ File discovery completed in {}ms",
            stats.processing_time_ms
        );
        info!(
            "📊 Discovery stats: {} scanned, {} indexed, {} skipped, {} errors",
            stats.total_files_scanned, stats.files_indexed, stats.files_skipped, stats.files_errors
        );

        Ok(DiscoveryResult {
            stats,
            indexed_files,
            skipped_files,
            error_files,
        })
    }

    /// Scan a single directory for files
    async fn scan_directory(
        &self,
        path: &Path,
    ) -> Result<DiscoveryResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut stats = DiscoveryStats {
            total_files_scanned: 0,
            files_indexed: 0,
            files_skipped: 0,
            files_errors: 0,
            directories_scanned: 1,
            processing_time_ms: 0,
        };

        let mut indexed_files = Vec::new();
        let mut skipped_files = Vec::new();
        let mut error_files = Vec::new();

        // Collect all files in the directory
        let mut files = self.collect_files_recursive(path).await?;

        // Limit the number of files to process to avoid overwhelming the system
        const MAX_FILES_PER_DISCOVERY: usize = 20; // Very conservative limit
        if files.len() > MAX_FILES_PER_DISCOVERY {
            warn!(
                "⚠️ Found {} files, limiting to {} for discovery to avoid system overload",
                files.len(),
                MAX_FILES_PER_DISCOVERY
            );
            files.truncate(MAX_FILES_PER_DISCOVERY);
        }

        stats.total_files_scanned = files.len();
        info!("📄 Found {} files in {:?}", files.len(), path);

        // Process files sequentially to avoid overwhelming the system
        // TODO: Re-enable batch processing once stability is confirmed
        for (index, file_path) in files.iter().enumerate() {
            info!(
                "📄 Processing file {}/{}: {:?}",
                index + 1,
                files.len(),
                file_path
            );

            match Self::process_single_file(file_path, &self.config, &self.vector_operations).await
            {
                Ok(ProcessResult::Indexed) => {
                    stats.files_indexed += 1;
                    indexed_files.push(file_path.clone());
                    info!(
                        "✅ Indexed file {}/{}: {:?}",
                        index + 1,
                        files.len(),
                        file_path
                    );
                }
                Ok(ProcessResult::Skipped(reason)) => {
                    stats.files_skipped += 1;
                    skipped_files.push(file_path.clone());
                    info!(
                        "⏭️ Skipped file {}/{}: {:?} - {}",
                        index + 1,
                        files.len(),
                        file_path,
                        reason
                    );
                }
                Err(e) => {
                    stats.files_errors += 1;
                    error_files.push((file_path.clone(), e.to_string()));
                    warn!(
                        "❌ Error processing file {}/{}: {:?} - {}",
                        index + 1,
                        files.len(),
                        file_path,
                        e
                    );
                }
            }
        }

        Ok(DiscoveryResult {
            stats,
            indexed_files,
            skipped_files,
            error_files,
        })
    }

    /// Collect all files recursively from a directory
    async fn collect_files_recursive(
        &self,
        path: &Path,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let mut files = Vec::new();

        if !path.exists() {
            return Ok(files);
        }

        if !path.is_dir() {
            // Single file
            if self.config.should_process_file(path) {
                files.push(path.to_path_buf());
            }
            return Ok(files);
        }

        // Recursively collect files
        let mut dir_queue = vec![path.to_path_buf()];

        while let Some(current_dir) = dir_queue.pop() {
            let mut entries = tokio::fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    if self.config.recursive {
                        // Check if directory should be excluded
                        let should_scan = !self.is_directory_excluded(&entry_path);
                        if should_scan {
                            dir_queue.push(entry_path);
                        }
                    }
                } else if entry_path.is_file() {
                    if self.config.should_process_file(&entry_path) {
                        files.push(entry_path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Check if a directory should be excluded from scanning
    fn is_directory_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude patterns
        for pattern in &self.config.exclude_patterns {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }

    /// Process a single file for indexing
    async fn process_single_file(
        file_path: &Path,
        config: &FileWatcherConfig,
        vector_operations: &VectorOperations,
    ) -> Result<ProcessResult, Box<dyn std::error::Error + Send + Sync>> {
        // Check if file should be processed
        if !config.should_process_file(file_path) {
            return Ok(ProcessResult::Skipped(
                "Does not match include patterns".to_string(),
            ));
        }

        // Check file size
        if let Ok(metadata) = std::fs::metadata(file_path) {
            if metadata.len() > config.max_file_size {
                return Ok(ProcessResult::Skipped(format!(
                    "File too large: {} bytes",
                    metadata.len()
                )));
            }
        }

        // Check if file is readable
        if let Err(e) = std::fs::File::open(file_path) {
            return Ok(ProcessResult::Skipped(format!("Cannot read file: {}", e)));
        }

        // Try to index the file
        match vector_operations.index_file_from_path(file_path).await {
            Ok(_) => Ok(ProcessResult::Indexed),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Get watch paths from configuration or discover from workspace
    async fn get_watch_paths(
        &self,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        // If watch_paths is explicitly configured, use it
        if let Some(paths) = &self.config.watch_paths {
            let mut valid_paths = Vec::new();
            for path in paths {
                if path.exists() {
                    valid_paths.push(path.clone());
                } else {
                    warn!("Watch path does not exist: {:?}", path);
                }
            }
            return Ok(valid_paths);
        }

        // Otherwise, try to discover from workspace configuration
        self.discover_workspace_paths().await
    }

    /// Discover watch paths from workspace configuration
    async fn discover_workspace_paths(
        &self,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let workspace_file = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("vectorize-workspace.yml");

        if !workspace_file.exists() {
            // Fallback to current directory
            return Ok(vec![
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            ]);
        }

        let content = tokio::fs::read_to_string(&workspace_file).await?;
        let workspace: serde_yaml::Value = serde_yaml::from_str(&content)?;

        let mut watch_paths = Vec::new();

        // Extract watch paths from global_settings
        if let Some(global_settings) = workspace.get("global_settings") {
            if let Some(file_watcher) = global_settings.get("file_watcher") {
                if let Some(paths) = file_watcher.get("watch_paths") {
                    if let Some(paths_array) = paths.as_sequence() {
                        for path in paths_array {
                            if let Some(path_str) = path.as_str() {
                                let path_buf = std::env::current_dir()
                                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                                    .join(path_str);
                                if path_buf.exists() {
                                    watch_paths.push(path_buf);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Extract project paths
        if let Some(projects) = workspace.get("projects") {
            if let Some(projects_array) = projects.as_sequence() {
                for project in projects_array {
                    if let Some(path) = project.get("path") {
                        if let Some(path_str) = path.as_str() {
                            let project_path = std::env::current_dir()
                                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                                .join(path_str);
                            if project_path.exists() {
                                watch_paths.push(project_path);
                            }
                        }
                    }
                }
            }
        }

        // If no paths found, use current directory
        if watch_paths.is_empty() {
            watch_paths
                .push(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
        }

        info!(
            "🔍 Discovered {} watch paths from workspace config",
            watch_paths.len()
        );
        Ok(watch_paths)
    }

    /// Sync with existing collections to remove orphaned files
    pub async fn sync_with_existing_collections(
        &self,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔄 Starting comprehensive synchronization with existing collections...");

        let start_time = std::time::Instant::now();
        let mut stats = SyncStats {
            collections_checked: 0,
            orphaned_files_removed: 0,
            files_verified: 0,
            sync_time_ms: 0,
        };

        // Get all collections from vector store
        let collections = self.vector_store.list_collections();
        stats.collections_checked = collections.len();

        info!("📊 Found {} collections to synchronize", collections.len());

        for collection_name in collections {
            info!("🔍 Synchronizing collection: {}", collection_name);

            // Get all vectors in the collection
            let collection = self.vector_store.get_collection(&collection_name)?;
            let vectors = collection.get_all_vectors();

            info!(
                "📄 Checking {} vectors in collection '{}'",
                vectors.len(),
                collection_name
            );

            let mut orphaned_files = Vec::new();

            for vector in vectors {
                stats.files_verified += 1;

                // Check if the file still exists
                if let Some(payload) = &vector.payload {
                    if let Some(file_path) = payload.data.get("file_path") {
                        if let Some(path_str) = file_path.as_str() {
                            let path = PathBuf::from(path_str);

                            if !path.exists() {
                                // File no longer exists, mark for removal
                                orphaned_files.push((vector.id.clone(), path_str.to_string()));
                                debug!("🗑️ Found orphaned file: {}", path_str);
                            } else {
                                debug!("✅ File exists: {}", path_str);
                            }
                        }
                    }
                }
            }

            // Remove orphaned files in batch
            if !orphaned_files.is_empty() {
                info!(
                    "🗑️ Removing {} orphaned files from collection '{}'",
                    orphaned_files.len(),
                    collection_name
                );

                for (vector_id, file_path) in orphaned_files {
                    if let Err(e) = collection.delete_vector(&vector_id) {
                        warn!("Failed to remove orphaned file {}: {}", file_path, e);
                    } else {
                        stats.orphaned_files_removed += 1;
                        info!("✅ Removed orphaned file: {}", file_path);
                    }
                }
            } else {
                info!(
                    "✅ No orphaned files found in collection '{}'",
                    collection_name
                );
            }
        }

        stats.sync_time_ms = start_time.elapsed().as_millis() as u64;

        info!("✅ Collection sync completed in {}ms", stats.sync_time_ms);
        info!(
            "📊 Sync results: {} collections checked, {} files verified, {} orphaned files removed",
            stats.collections_checked, stats.files_verified, stats.orphaned_files_removed
        );

        Ok(SyncResult { stats })
    }

    /// Detect files that exist in the filesystem but are not indexed
    pub async fn detect_unindexed_files(
        &self,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 Detecting unindexed files in watch paths...");

        let watch_paths = self.get_watch_paths().await?;
        let mut unindexed_files = Vec::new();

        for watch_path in watch_paths {
            info!("📂 Scanning for unindexed files in: {:?}", watch_path);

            let files = self.collect_files_recursive(&watch_path).await?;
            info!(
                "📄 Found {} files to check in {:?}",
                files.len(),
                watch_path
            );

            for file_path in files {
                // Check if file is indexed in any collection
                let mut is_indexed = false;

                let collections = self.vector_store.list_collections();
                for collection_name in collections {
                    if let Ok(collection) = self.vector_store.get_collection(&collection_name) {
                        let vectors = collection.get_all_vectors();

                        for vector in vectors {
                            if let Some(payload) = &vector.payload {
                                if let Some(indexed_path) = payload.data.get("file_path") {
                                    if let Some(path_str) = indexed_path.as_str() {
                                        if PathBuf::from(path_str) == file_path {
                                            is_indexed = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if is_indexed {
                        break;
                    }
                }

                if !is_indexed {
                    unindexed_files.push(file_path.clone());
                    debug!("📄 Unindexed file found: {:?}", file_path);
                }
            }
        }

        info!("📊 Found {} unindexed files", unindexed_files.len());
        Ok(unindexed_files)
    }

    /// Perform comprehensive synchronization (orphaned + unindexed)
    pub async fn comprehensive_sync(
        &self,
    ) -> Result<(SyncResult, Vec<PathBuf>), Box<dyn std::error::Error + Send + Sync>> {
        info!("🔄 Starting comprehensive synchronization...");

        // First, remove orphaned files
        let sync_result = self.sync_with_existing_collections().await?;

        // Then, detect unindexed files
        let unindexed_files = self.detect_unindexed_files().await?;

        info!(
            "✅ Comprehensive sync completed: {} orphaned files removed, {} unindexed files detected",
            sync_result.stats.orphaned_files_removed,
            unindexed_files.len()
        );

        Ok((sync_result, unindexed_files))
    }
}

/// Result of processing a single file
#[derive(Debug)]
enum ProcessResult {
    Indexed,
    Skipped(String),
}

/// Statistics for collection sync operations
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub collections_checked: usize,
    pub orphaned_files_removed: usize,
    pub files_verified: usize,
    pub sync_time_ms: u64,
}

/// Result of collection sync operation
#[derive(Debug)]
pub struct SyncResult {
    pub stats: SyncStats,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_file_discovery_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");

        // Create test file
        fs::write(&test_file, "# Test").unwrap();

        // Create mock config without problematic exclude patterns
        let config = FileWatcherConfig {
            watch_paths: Some(vec![temp_dir.path().to_path_buf()]),
            include_patterns: vec!["*.md".to_string()],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/*.tmp".to_string(),
                "**/*.log".to_string(),
            ],
            ..FileWatcherConfig::default()
        };

        // Note: This test would need proper mocking of VectorOperations and VectorStore
        // For now, just test the file collection logic
        let discovery = FileDiscovery {
            config: config.clone(),
            vector_operations: Arc::new(VectorOperations::new(
                Arc::new(VectorStore::new_auto()),
                Arc::new(RwLock::new(EmbeddingManager::new())),
                config.clone(),
            )),
            vector_store: Arc::new(VectorStore::new_auto()),
        };

        let files = discovery
            .collect_files_recursive(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[tokio::test]
    async fn test_directory_exclusion() {
        let mut config = FileWatcherConfig::default();
        config.exclude_patterns = vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            ".git".to_string(),
        ];
        let discovery = FileDiscovery {
            config: config.clone(),
            vector_operations: Arc::new(VectorOperations::new(
                Arc::new(VectorStore::new_auto()),
                Arc::new(RwLock::new(EmbeddingManager::new())),
                config.clone(),
            )),
            vector_store: Arc::new(VectorStore::new_auto()),
        };

        // Test excluded directories
        assert!(discovery.is_directory_excluded(Path::new("target/debug")));
        assert!(discovery.is_directory_excluded(Path::new("node_modules/test")));
        assert!(discovery.is_directory_excluded(Path::new(".git")));

        // Test non-excluded directories
        assert!(!discovery.is_directory_excluded(Path::new("src")));
        assert!(!discovery.is_directory_excluded(Path::new("docs")));
    }
}
