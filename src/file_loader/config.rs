//! Configuration for file loading and indexing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/dist/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/.git/**".to_string(),
                "**/data/**".to_string(),
                "**/*.bin".to_string(),
                "**/*.exe".to_string(),
                "**/*.dll".to_string(),
                "**/*.so".to_string(),
            ],
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

