//! Storage index management (.vecidx files)

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, VectorizerError};

/// Storage index containing metadata about all collections in the .vecdb archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageIndex {
    /// Format version
    pub version: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Collections in the archive
    pub collections: Vec<CollectionIndex>,

    /// Total uncompressed size in bytes
    pub total_size: u64,

    /// Total compressed size in bytes
    pub compressed_size: u64,

    /// Compression ratio (compressed_size / total_size)
    pub compression_ratio: f64,
}

/// Index for a single collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionIndex {
    /// Collection name
    pub name: String,

    /// Files in this collection
    pub files: Vec<FileEntry>,

    /// Number of vectors in this collection
    pub vector_count: usize,

    /// Vector dimension
    pub dimension: usize,

    /// Collection metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Individual file entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative path within the archive
    pub path: String,

    /// Original uncompressed size
    pub size: u64,

    /// Compressed size in archive
    pub compressed_size: u64,

    /// SHA-256 checksum of uncompressed content
    pub checksum: String,

    /// File type (vectors, metadata, config, etc.)
    #[serde(rename = "type")]
    pub file_type: FileType,
}

/// Type of file in the storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    /// Vector data files
    Vectors,
    /// Metadata files
    Metadata,
    /// Configuration files
    Config,
    /// Index files
    Index,
    /// Tokenizer files (BM25 vocabulary)
    Tokenizer,
    /// Other files
    Other,
}

impl StorageIndex {
    /// Create a new storage index
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            version: crate::storage::STORAGE_VERSION.to_string(),
            created_at: now,
            updated_at: now,
            collections: Vec::new(),
            total_size: 0,
            compressed_size: 0,
            compression_ratio: 0.0,
        }
    }

    /// Add a collection to the index
    pub fn add_collection(&mut self, collection: CollectionIndex) {
        // Update totals
        for file in &collection.files {
            self.total_size += file.size;
            self.compressed_size += file.compressed_size;
        }

        self.collections.push(collection);
        self.update_compression_ratio();
        self.updated_at = Utc::now();
    }

    /// Update compression ratio
    fn update_compression_ratio(&mut self) {
        if self.total_size > 0 {
            self.compression_ratio = self.compressed_size as f64 / self.total_size as f64;
        }
    }

    /// Find a collection by name
    pub fn find_collection(&self, name: &str) -> Option<&CollectionIndex> {
        self.collections.iter().find(|c| c.name == name)
    }

    /// Find a collection by name (mutable)
    pub fn find_collection_mut(&mut self, name: &str) -> Option<&mut CollectionIndex> {
        self.collections.iter_mut().find(|c| c.name == name)
    }

    /// Remove a collection from the index
    pub fn remove_collection(&mut self, name: &str) -> bool {
        if let Some(pos) = self.collections.iter().position(|c| c.name == name) {
            let collection = self.collections.remove(pos);

            // Update totals
            for file in &collection.files {
                self.total_size = self.total_size.saturating_sub(file.size);
                self.compressed_size = self.compressed_size.saturating_sub(file.compressed_size);
            }

            self.update_compression_ratio();
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Save the index to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;

        fs::write(path, json).map_err(|e| VectorizerError::Io(e))?;

        Ok(())
    }

    /// Load the index from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| VectorizerError::Io(e))?;

        let index: Self = serde_json::from_str(&content)
            .map_err(|e| VectorizerError::Deserialization(e.to_string()))?;

        Ok(index)
    }

    /// Get total number of vectors across all collections
    pub fn total_vectors(&self) -> usize {
        self.collections.iter().map(|c| c.vector_count).sum()
    }

    /// Get total number of collections
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }
}

impl Default for StorageIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectionIndex {
    /// Create a new collection index
    pub fn new(name: String) -> Self {
        Self {
            name,
            files: Vec::new(),
            vector_count: 0,
            dimension: 0,
            metadata: HashMap::new(),
        }
    }

    /// Add a file to the collection
    pub fn add_file(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    /// Get total uncompressed size
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Get total compressed size
    pub fn compressed_size(&self) -> u64 {
        self.files.iter().map(|f| f.compressed_size).sum()
    }
}

impl FileEntry {
    /// Create a new file entry
    pub fn new(
        path: String,
        size: u64,
        compressed_size: u64,
        checksum: String,
        file_type: FileType,
    ) -> Self {
        Self {
            path,
            size,
            compressed_size,
            checksum,
            file_type,
        }
    }
}

/// Detect file type from path
pub fn detect_file_type(path: &str) -> FileType {
    let path_lower = path.to_lowercase();

    if path_lower.ends_with(".bin") || path_lower.ends_with(".bin.gz") {
        FileType::Vectors
    } else if path_lower.ends_with("_metadata.json") {
        FileType::Metadata
    } else if path_lower.ends_with("_tokenizer.json") {
        FileType::Tokenizer
    } else if path_lower.ends_with(".json")
        || path_lower.ends_with(".yaml")
        || path_lower.ends_with(".yml")
    {
        FileType::Config
    } else if path_lower.contains("index") {
        FileType::Index
    } else {
        FileType::Other
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_storage_index_new() {
        let index = StorageIndex::new();
        assert_eq!(index.version, crate::storage::STORAGE_VERSION);
        assert_eq!(index.collections.len(), 0);
        assert_eq!(index.total_size, 0);
    }

    #[test]
    fn test_add_collection() {
        let mut index = StorageIndex::new();
        let mut collection = CollectionIndex::new("test".to_string());
        collection.add_file(FileEntry::new(
            "test.bin".to_string(),
            1000,
            500,
            "checksum".to_string(),
            FileType::Vectors,
        ));

        index.add_collection(collection);
        assert_eq!(index.collections.len(), 1);
        assert_eq!(index.total_size, 1000);
        assert_eq!(index.compressed_size, 500);
        assert_eq!(index.compression_ratio, 0.5);
    }

    #[test]
    fn test_find_collection() {
        let mut index = StorageIndex::new();
        index.add_collection(CollectionIndex::new("test1".to_string()));
        index.add_collection(CollectionIndex::new("test2".to_string()));

        assert!(index.find_collection("test1").is_some());
        assert!(index.find_collection("test2").is_some());
        assert!(index.find_collection("test3").is_none());
    }

    #[test]
    fn test_remove_collection() {
        let mut index = StorageIndex::new();
        let mut collection = CollectionIndex::new("test".to_string());
        collection.add_file(FileEntry::new(
            "test.bin".to_string(),
            1000,
            500,
            "checksum".to_string(),
            FileType::Vectors,
        ));

        index.add_collection(collection);
        assert_eq!(index.collections.len(), 1);

        assert!(index.remove_collection("test"));
        assert_eq!(index.collections.len(), 0);
        assert_eq!(index.total_size, 0);
        assert_eq!(index.compressed_size, 0);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test.vecidx");

        let mut index = StorageIndex::new();
        index.add_collection(CollectionIndex::new("test".to_string()));

        index.save(&index_path).unwrap();
        assert!(index_path.exists());

        let loaded = StorageIndex::load(&index_path).unwrap();
        assert_eq!(loaded.version, index.version);
        assert_eq!(loaded.collections.len(), 1);
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type("data.bin"), FileType::Vectors);
        assert_eq!(detect_file_type("data.bin.gz"), FileType::Vectors);
        assert_eq!(
            detect_file_type("collection_metadata.json"),
            FileType::Metadata
        );
        assert_eq!(detect_file_type("config.yaml"), FileType::Config);
        assert_eq!(detect_file_type("unknown.txt"), FileType::Other);
    }
}
