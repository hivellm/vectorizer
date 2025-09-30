//! Collection metadata structures for tracking indexed files and statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Metadata for a single indexed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File path relative to project root
    pub path: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Number of chunks created from this file
    pub chunk_count: usize,
    /// Number of vectors generated from this file
    pub vector_count: usize,
    /// When this file was indexed
    pub indexed_at: DateTime<Utc>,
    /// File modification time at indexing
    pub file_modified_at: DateTime<Utc>,
    /// Hash of file content for change detection
    pub content_hash: String,
}

/// Collection-level metadata stored in .vectorizer/metadata.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadataFile {
    /// Collection name
    pub collection_name: String,
    /// Project path
    pub project_path: String,
    /// When the collection was first created
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub last_updated: DateTime<Utc>,
    /// Total number of files indexed
    pub total_files: usize,
    /// Total number of vectors in the collection
    pub total_vectors: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Configuration used for indexing
    pub config: CollectionIndexingConfig,
    /// Map of file paths to their metadata
    pub files: HashMap<String, FileMetadata>,
    /// Embedding model information
    pub embedding_model: EmbeddingModelInfo,
}

/// Configuration used for indexing the collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndexingConfig {
    /// Chunk size used
    pub chunk_size: usize,
    /// Chunk overlap used
    pub chunk_overlap: usize,
    /// Include patterns used
    pub include_patterns: Vec<String>,
    /// Exclude patterns used
    pub exclude_patterns: Vec<String>,
    /// Allowed extensions
    pub allowed_extensions: Vec<String>,
    /// Maximum file size
    pub max_file_size: usize,
}

/// Information about the embedding model used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelInfo {
    /// Model type (bm25, tfidf, etc.)
    pub model_type: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Model-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl CollectionMetadataFile {
    /// Create a new metadata file
    pub fn new(
        collection_name: String,
        project_path: String,
        config: CollectionIndexingConfig,
        embedding_model: EmbeddingModelInfo,
    ) -> Self {
        Self {
            collection_name,
            project_path,
            created_at: Utc::now(),
            last_updated: Utc::now(),
            total_files: 0,
            total_vectors: 0,
            total_chunks: 0,
            config,
            files: HashMap::new(),
            embedding_model,
        }
    }

    /// Add or update file metadata
    pub fn add_file(&mut self, file_metadata: FileMetadata) {
        self.files.insert(file_metadata.path.clone(), file_metadata);
        self.update_totals();
        self.last_updated = Utc::now();
    }

    /// Remove file metadata
    pub fn remove_file(&mut self, file_path: &str) {
        self.files.remove(file_path);
        self.update_totals();
        self.last_updated = Utc::now();
    }

    /// Update total counts
    fn update_totals(&mut self) {
        self.total_files = self.files.len();
        self.total_vectors = self.files.values().map(|f| f.vector_count).sum();
        self.total_chunks = self.files.values().map(|f| f.chunk_count).sum();
    }

    /// Get file metadata by path
    pub fn get_file(&self, path: &str) -> Option<&FileMetadata> {
        self.files.get(path)
    }

    /// Check if file exists and is up to date
    pub fn is_file_current(&self, path: &str, current_modified: DateTime<Utc>, current_hash: &str) -> bool {
        if let Some(file_meta) = self.files.get(path) {
            file_meta.file_modified_at == current_modified && file_meta.content_hash == current_hash
        } else {
            false
        }
    }

    /// Get files that need re-indexing
    pub fn get_files_to_reindex(&self, current_files: &HashMap<String, (DateTime<Utc>, String)>) -> Vec<String> {
        let mut to_reindex = Vec::new();
        
        for (path, (modified, hash)) in current_files {
            if !self.is_file_current(path, *modified, hash) {
                to_reindex.push(path.clone());
            }
        }

        // Also check for files that were removed
        for existing_path in self.files.keys() {
            if !current_files.contains_key(existing_path) {
                to_reindex.push(existing_path.clone());
            }
        }

        to_reindex
    }
}
