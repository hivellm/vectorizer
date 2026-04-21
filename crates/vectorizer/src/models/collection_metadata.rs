//! Collection metadata structures for tracking indexed files

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Metadata for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadataFile {
    /// Collection name
    pub name: String,
    /// Files indexed in this collection
    pub files: Vec<FileMetadata>,
    /// Collection configuration
    pub config: CollectionIndexingConfig,
    /// Embedding model information
    pub embedding_model: EmbeddingModelInfo,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Metadata for an indexed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File path
    pub path: String,
    /// File hash
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Number of chunks created from this file
    pub chunk_count: usize,
    /// Indexed timestamp
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration for collection indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndexingConfig {
    /// Chunk size
    pub chunk_size: usize,
    /// Chunk overlap
    pub chunk_overlap: usize,
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Maximum file size
    pub max_file_size: usize,
}

/// Embedding model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelInfo {
    /// Model name
    pub name: String,
    /// Model version
    pub version: Option<String>,
    /// Vector dimension
    pub dimension: usize,
    /// Model parameters
    pub parameters: HashMap<String, serde_json::Value>,
}
