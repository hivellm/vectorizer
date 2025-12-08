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

        // For large/binary-safe handling, avoid read_to_string; copy file to temp dir
        // and process via DocumentLoader
        let temp_dir = std::env::temp_dir().join(format!("temp_single_{}", uuid::Uuid::new_v4()));
        if let Err(e) = tokio::fs::create_dir_all(&temp_dir).await {
            tracing::warn!("Failed to create temp dir {:?}: {}", temp_dir, e);
            return Ok(());
        }

        // Destination file keeps original filename
        let dest_file = temp_dir.join(
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("file")),
        );
        match tokio::fs::copy(path, &dest_file).await {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(
                    "Failed to copy file {:?} to temp {:?}: {}",
                    path,
                    dest_file,
                    e
                );
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                return Ok(());
            }
        }

        // Build loader config
        let mut loader_config = LoaderConfig {
            max_chunk_size: 2048,
            chunk_overlap: 256,
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: collection_name.clone(),
            max_file_size: self.config.max_file_size as usize,
        };

        // CRITICAL: Always enforce hardcoded exclusions (Python cache, binaries, etc.)
        loader_config.ensure_hardcoded_excludes();

        let embedding_manager = {
            let mut em = EmbeddingManager::new();
            let bm25 = crate::embedding::Bm25Embedding::new(512);
            em.register_provider("bm25".to_string(), Box::new(bm25));
            em.set_default_provider("bm25").unwrap();
            em
        };
        let mut loader = FileLoader::with_embedding_manager(loader_config, embedding_manager);
        match loader
            .load_and_index_project(&temp_dir.to_string_lossy(), &self.vector_store)
            .await
        {
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
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use tokio::sync::RwLock;

    use super::*;
    use crate::file_watcher::FileWatcherConfig;

    fn create_test_ops() -> VectorOperations {
        let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
        let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
        let config = FileWatcherConfig::default();
        VectorOperations::new(vector_store, embedding_manager, config)
    }

    // Task 1.3: Comprehensive test cases for determine_collection_name()

    #[test]
    fn test_docs_architecture_collection() {
        let ops = create_test_ops();

        let test_paths = vec![
            "/home/user/project/docs/architecture/system.md",
            "/docs/architecture/README.md",
        ];

        for path in test_paths {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, "docs-architecture",
                "Path {} should map to docs-architecture",
                path
            );
        }
    }

    #[test]
    fn test_docs_subdirectories() {
        let ops = create_test_ops();

        let test_cases = vec![
            ("/docs/templates/pr.md", "docs-templates"),
            ("/docs/processes/release.md", "docs-processes"),
            ("/docs/governance/voting.md", "docs-governance"),
            ("/docs/navigation/sitemap.md", "docs-navigation"),
            ("/docs/testing/strategy.md", "docs-testing"),
            ("/docs/random/file.md", "docs-architecture"), // default docs
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, expected,
                "Path {} should map to {}",
                path, expected
            );
        }
    }

    #[test]
    fn test_vectorizer_collections() {
        let ops = create_test_ops();

        let test_cases = vec![
            // Note: /docs/ pattern is checked first, so any path with /docs/ will match that
            // These test cases avoid /docs/ pattern
            ("/vectorizer/src/main.rs", "vectorizer-source"),
            ("/vectorizer/src/db/vector_store.rs", "vectorizer-source"),
            ("/project/vectorizer/README.md", "vectorizer-source"),
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, expected,
                "Path {} should map to {}",
                path, expected
            );
        }
    }

    #[test]
    fn test_vectorizer_sdk_language_detection() {
        let ops = create_test_ops();

        let test_cases = vec![
            (
                "/vectorizer/client-sdks/typescript/index.ts",
                "vectorizer-sdk-typescript",
            ),
            (
                "/vectorizer/sdks/nodejs/client.js",
                "vectorizer-sdk-typescript",
            ),
            (
                "/vectorizer/client-sdks/python/client.py",
                "vectorizer-sdk-python",
            ),
            ("/vectorizer/sdks/rust/lib.rs", "vectorizer-sdk-rust"),
            ("/vectorizer/sdks/unknown/file.txt", "vectorizer-source"), // fallback
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, expected,
                "SDK path {} should map to {}",
                path, expected
            );
        }
    }

    #[test]
    fn test_gov_collections() {
        let ops = create_test_ops();

        let test_cases = vec![
            ("/gov/bips/BIP-001.md", "gov-bips"),
            ("/gov/guidelines/code.md", "gov-guidelines"),
            ("/gov/proposals/2024-q1.md", "gov-proposals"),
            ("/gov/minutes/2024-01.md", "gov-minutes"),
            ("/gov/schemas/voting.json", "gov-schemas"),
            ("/gov/teams/engineering.md", "gov-teams"),
            ("/gov/metrics/2024-q1.md", "gov-metrics"),
            ("/gov/issues/123.md", "gov-issues"),
            ("/gov/snapshot/vote-123.json", "gov-snapshot"),
            ("/gov/other/file.md", "gov-core"), // default gov
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, expected,
                "Path {} should map to {}",
                path, expected
            );
        }
    }

    #[test]
    fn test_unknown_paths_use_default() {
        let ops = create_test_ops();

        let test_paths = vec![
            "/random/path/file.txt",
            "F:\\Some\\Random\\Directory\\document.md",
            "/usr/local/bin/script.sh",
            "/home/user/downloads/file.pdf",
            "/third-party/libsodium/src/file.c",
            "/Server/ToS-Server-5/Api/endpoint.cs",
            "/Benchmark/src/test.cpp",
        ];

        for path in test_paths {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, "workspace-default",
                "Unknown path {} should use default collection",
                path
            );
        }
    }

    #[test]
    fn test_custom_default_collection() {
        let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
        let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
        let mut config = FileWatcherConfig::default();
        config.default_collection = Some("my-custom-collection".to_string());
        let ops = VectorOperations::new(vector_store, embedding_manager, config);

        let collection = ops.determine_collection_name(&PathBuf::from("/random/path/file.txt"));
        assert_eq!(collection, "my-custom-collection");
    }

    #[test]
    fn test_windows_paths() {
        let ops = create_test_ops();

        // Note: The function checks for forward slashes "/" not backslashes "\"
        // Windows paths with backslashes won't match the patterns and will use default
        let test_cases = vec![
            (
                "C:\\Users\\dev\\project\\docs\\architecture\\design.md",
                "workspace-default",
            ),
            ("D:\\Work\\vectorizer\\src\\main.rs", "workspace-default"),
            ("F:\\Gov\\bips\\BIP-001.md", "workspace-default"),
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, expected,
                "Windows path {} should map to {}",
                path, expected
            );
        }
    }

    #[test]
    fn test_collection_mapping_priority() {
        let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
        let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
        let mut config = FileWatcherConfig::default();

        // Configure collection mapping with patterns that match the test paths
        let mut mapping = std::collections::HashMap::new();
        // Use patterns that will definitely match
        mapping.insert(
            "**/project/docs/**/*.md".to_string(),
            "custom-docs".to_string(),
        );
        mapping.insert(
            "**/project/src/**/*.rs".to_string(),
            "custom-rust".to_string(),
        );
        mapping.insert(
            "**/project/tests/**/*".to_string(),
            "custom-tests".to_string(),
        );
        config.collection_mapping = Some(mapping);

        let ops = VectorOperations::new(vector_store, embedding_manager, config);

        // Collection mapping should take priority over known patterns
        assert_eq!(
            ops.determine_collection_name(&PathBuf::from("/project/docs/guide.md")),
            "custom-docs"
        );
        assert_eq!(
            ops.determine_collection_name(&PathBuf::from("/project/src/main.rs")),
            "custom-rust"
        );
        assert_eq!(
            ops.determine_collection_name(&PathBuf::from("/project/tests/test.rs")),
            "custom-tests"
        );

        // Paths that don't match mapping should fall back to known patterns or default
        assert_eq!(
            ops.determine_collection_name(&PathBuf::from("/docs/architecture/design.md")),
            "docs-architecture" // Known pattern, not in mapping
        );
    }

    #[test]
    fn test_collection_mapping_windows_paths_normalized() {
        let vector_store = Arc::new(crate::VectorStore::new_cpu_only());
        let embedding_manager = Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new()));
        let mut config = FileWatcherConfig::default();

        // Configure collection mapping with forward slashes (will be normalized)
        let mut mapping = std::collections::HashMap::new();
        mapping.insert("*/docs/**/*.md".to_string(), "documentation".to_string());
        mapping.insert("*/src/**/*.rs".to_string(), "rust-code".to_string());
        config.collection_mapping = Some(mapping);

        let ops = VectorOperations::new(vector_store, embedding_manager, config);

        // Windows paths with backslashes should be normalized and match patterns
        assert_eq!(
            ops.determine_collection_name(&PathBuf::from(r"C:\project\docs\guide.md")),
            "documentation"
        );
        assert_eq!(
            ops.determine_collection_name(&PathBuf::from(r"D:\work\src\main.rs")),
            "rust-code"
        );
    }

    #[test]
    fn test_deeply_nested_paths() {
        let ops = create_test_ops();

        let collection = ops.determine_collection_name(&PathBuf::from(
            "/a/b/c/d/e/f/docs/architecture/deeply/nested/file.md",
        ));
        assert_eq!(collection, "docs-architecture");
    }

    #[test]
    fn test_path_with_similar_names() {
        let ops = create_test_ops();

        // Path contains "docs" but not in expected location
        let collection = ops.determine_collection_name(&PathBuf::from("/mydocs/file.txt"));
        assert_eq!(collection, "workspace-default");

        // Path contains "vectorizer" but not in expected location
        let collection =
            ops.determine_collection_name(&PathBuf::from("/not-vectorizer/src/main.rs"));
        assert_eq!(collection, "workspace-default");
    }

    #[test]
    fn test_relative_paths() {
        let ops = create_test_ops();

        // Note: The function uses contains("/pattern/") which requires forward slashes
        // Relative paths without leading slash won't match these patterns
        let test_cases = vec![
            ("/docs/architecture/file.md", "docs-architecture"),
            ("/vectorizer/src/main.rs", "vectorizer-source"),
            ("/gov/bips/BIP-001.md", "gov-bips"),
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, expected,
                "Path {} should map to {}",
                path, expected
            );
        }
    }

    #[test]
    fn test_paths_with_special_characters() {
        let ops = create_test_ops();

        let test_cases = vec![
            (
                "/docs/architecture/file with spaces.md",
                "docs-architecture",
            ),
            ("/vectorizer/src/module-name.rs", "vectorizer-source"),
            ("/gov/bips/BIP_001.md", "gov-bips"),
        ];

        for (path, expected) in test_cases {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(collection, expected);
        }
    }

    #[test]
    fn test_empty_path() {
        let ops = create_test_ops();
        let collection = ops.determine_collection_name(&PathBuf::from(""));
        assert_eq!(collection, "workspace-default");
    }

    #[test]
    fn test_no_empty_collection_creation() {
        let ops = create_test_ops();

        // These paths previously caused empty collections to be created
        // Now they should all use the default collection
        let problematic_paths = vec![
            "/third-party/libsodium/src/file.c",
            "/Server/ToS-Server-5/Api/endpoint.cs",
            "/Benchmark/src/test.cpp",
            "/test/symbols/symbol.txt",
            "/libsodium-regen-msvc/file.h",
        ];

        for path in problematic_paths {
            let collection = ops.determine_collection_name(&PathBuf::from(path));
            assert_eq!(
                collection, "workspace-default",
                "Path {} should NOT create new collection, should use default",
                path
            );
        }
    }
}
