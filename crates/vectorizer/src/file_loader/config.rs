//! Configuration for file loading and indexing

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// File loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Overlap between chunks in characters
    pub chunk_overlap: usize,
    /// Glob patterns for files to include
    pub include_patterns: Vec<String>,
    /// Glob patterns for files/directories to exclude
    pub exclude_patterns: Vec<String>,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Embedding type to use
    pub embedding_type: String,
    /// Collection name for documents
    pub collection_name: String,
    /// Maximum file size in bytes
    pub max_file_size: usize,
}

impl LoaderConfig {
    /// Ensures that hardcoded exclusions are always present, merging with user-provided patterns
    pub fn ensure_hardcoded_excludes(&mut self) {
        let hardcoded = Self::get_hardcoded_excludes();

        // Combine user patterns with hardcoded patterns, removing duplicates
        let mut all_excludes: Vec<String> = self.exclude_patterns.clone();
        for pattern in hardcoded {
            if !all_excludes.contains(&pattern) {
                all_excludes.push(pattern);
            }
        }

        self.exclude_patterns = all_excludes;
    }

    /// Get hardcoded exclusion patterns that should NEVER be indexed
    /// These are critical patterns for Python cache, binaries, and other artifacts
    /// that can cause performance issues and bloat the index
    pub fn get_hardcoded_excludes() -> Vec<String> {
        vec![
            // Version control
            "**/.git/**".to_string(),
            // Build artifacts and dependencies
            "**/target/**".to_string(),       // Rust build
            "**/node_modules/**".to_string(), // Node.js dependencies
            "**/dist/**".to_string(),         // Distribution builds
            "**/build/**".to_string(),        // Generic build directory
            "**/.fingerprint/**".to_string(), // Rust incremental
            // Vectorizer specific (CRITICAL - prevents recursive indexing)
            "**/data/**".to_string(),            // Vectorizer data directory
            "**/*.vecdb".to_string(),            // Vectorizer database
            "**/*.vecidx".to_string(),           // Vectorizer index
            "**/*_metadata.json".to_string(),    // Legacy metadata
            "**/*_tokenizer.json".to_string(),   // Legacy tokenizer
            "**/*_vector_store.bin".to_string(), // Legacy vector store
            // Python artifacts (CRITICAL - prevents ~400MB binary indexing)
            "**/__pycache__/**".to_string(), // Python bytecode cache
            "**/*.pyc".to_string(),          // Compiled bytecode
            "**/*.pyo".to_string(),          // Optimized bytecode
            "**/*.pyd".to_string(),          // Python dynamic module
            "**/*.whl".to_string(),          // Wheel packages
            "**/*.egg".to_string(),          // Egg packages
            "**/*.egg-info/**".to_string(),  // Egg metadata
            "**/site-packages/**".to_string(), // Installed packages
            "**/.venv/**".to_string(),       // Virtual environments
            "**/venv/**".to_string(),
            "**/env/**".to_string(),
            "**/.pytest_cache/**".to_string(), // Pytest cache
            "**/.tox/**".to_string(),          // Tox environments
            // Binary and compiled files (CRITICAL - prevents memory issues)
            "**/*.bin".to_string(),
            "**/*.exe".to_string(),
            "**/*.dll".to_string(),
            "**/*.so".to_string(),    // Linux shared libraries
            "**/*.dylib".to_string(), // macOS shared libraries
            "**/*.a".to_string(),     // Static libraries
            "**/*.o".to_string(),     // Object files
            "**/*.obj".to_string(),   // Windows object files
            "**/*.lib".to_string(),   // Windows libraries
            // Temporary and editor files
            "**/*.tmp".to_string(),
            "**/*.log".to_string(),
            "**/*.part".to_string(),
            "**/*.lock".to_string(),
            "**/~*".to_string(),         // Backup files
            "**/.#*".to_string(),        // Emacs lock files
            "**/*.swp".to_string(),      // Vim swap
            "**/*.swo".to_string(),      // Vim swap
            "**/Cargo.lock".to_string(), // Lock files (not source)
            "**/package-lock.json".to_string(),
            "**/pnpm-lock.yaml".to_string(),
            "**/yarn.lock".to_string(),
            // OS specific
            "**/.DS_Store".to_string(), // macOS metadata
            "**/Thumbs.db".to_string(), // Windows thumbnails
            // IDE and editor directories
            "**/.vscode/**".to_string(),
            "**/.idea/**".to_string(),
            "**/.eclipse/**".to_string(),
            // Hidden files (generally)
            "**/.*/**".to_string(), // Hidden directories
        ]
    }
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 2048,
            chunk_overlap: 256,
            include_patterns: vec![
                "**/*.md".to_string(),
                "**/*.txt".to_string(),
                "**/*.json".to_string(),
                "**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.js".to_string(),
            ],
            exclude_patterns: Self::get_hardcoded_excludes(),
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: "documents".to_string(),
            max_file_size: 1024 * 1024, // 1MB
        }
    }
}

/// Document chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Unique identifier for the chunk
    pub id: String,
    /// Text content of the chunk
    pub content: String,
    /// Source file path
    pub file_path: String,
    /// Chunk index within the document
    pub chunk_index: usize,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_config_default() {
        let config = LoaderConfig::default();

        assert_eq!(config.max_chunk_size, 2048);
        assert_eq!(config.chunk_overlap, 256);
        assert_eq!(config.embedding_dimension, 512);
        assert_eq!(config.embedding_type, "bm25");
        assert_eq!(config.collection_name, "documents");
        assert_eq!(config.max_file_size, 1024 * 1024);

        // Should have default include patterns
        assert!(!config.include_patterns.is_empty());
        assert!(config.include_patterns.contains(&"**/*.md".to_string()));

        // Should have hardcoded exclude patterns
        assert!(!config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_get_hardcoded_excludes() {
        let excludes = LoaderConfig::get_hardcoded_excludes();

        // Should include critical patterns
        assert!(excludes.contains(&"**/.git/**".to_string()));
        assert!(excludes.contains(&"**/target/**".to_string()));
        assert!(excludes.contains(&"**/node_modules/**".to_string()));
        assert!(excludes.contains(&"**/__pycache__/**".to_string()));
        assert!(excludes.contains(&"**/*.vecdb".to_string()));

        // Should be a reasonable number of patterns
        assert!(excludes.len() > 30);
    }

    #[test]
    fn test_ensure_hardcoded_excludes() {
        let mut config = LoaderConfig {
            max_chunk_size: 2048,
            chunk_overlap: 256,
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec!["**/custom_exclude/**".to_string()],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: "test".to_string(),
            max_file_size: 1024 * 1024,
        };

        config.ensure_hardcoded_excludes();

        // Should have custom pattern
        assert!(
            config
                .exclude_patterns
                .contains(&"**/custom_exclude/**".to_string())
        );

        // Should have hardcoded patterns
        assert!(config.exclude_patterns.contains(&"**/.git/**".to_string()));
        assert!(
            config
                .exclude_patterns
                .contains(&"**/target/**".to_string())
        );

        // Should not have duplicates
        let unique_count = config.exclude_patterns.len();
        let mut deduped = config.exclude_patterns.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(unique_count, deduped.len());
    }

    #[test]
    fn test_document_chunk_creation() {
        let chunk = DocumentChunk {
            id: "chunk_1".to_string(),
            content: "This is a test chunk".to_string(),
            file_path: "/path/to/file.txt".to_string(),
            chunk_index: 0,
            metadata: HashMap::new(),
        };

        assert_eq!(chunk.id, "chunk_1");
        assert_eq!(chunk.content, "This is a test chunk");
        assert_eq!(chunk.file_path, "/path/to/file.txt");
        assert_eq!(chunk.chunk_index, 0);
    }

    #[test]
    fn test_document_chunk_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::json!("test.txt"));
        metadata.insert("language".to_string(), serde_json::json!("rust"));

        let chunk = DocumentChunk {
            id: "chunk_1".to_string(),
            content: "fn main() {}".to_string(),
            file_path: "/test.rs".to_string(),
            chunk_index: 0,
            metadata: metadata.clone(),
        };

        assert_eq!(chunk.metadata.len(), 2);
        assert_eq!(chunk.metadata["source"], "test.txt");
        assert_eq!(chunk.metadata["language"], "rust");
    }

    #[test]
    fn test_loader_config_custom() {
        let config = LoaderConfig {
            max_chunk_size: 4096,
            chunk_overlap: 512,
            include_patterns: vec!["**/*.py".to_string()],
            exclude_patterns: vec!["**/tests/**".to_string()],
            embedding_dimension: 768,
            embedding_type: "bert".to_string(),
            collection_name: "python_docs".to_string(),
            max_file_size: 5 * 1024 * 1024,
        };

        assert_eq!(config.max_chunk_size, 4096);
        assert_eq!(config.embedding_dimension, 768);
        assert_eq!(config.embedding_type, "bert");
        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
    }
}
