//! Cache metadata structures and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use super::CacheResult;

/// Cache metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Cache version
    pub version: String,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    
    /// Collections cache information
    pub collections: HashMap<String, CollectionCacheInfo>,
    
    /// Global cache configuration
    pub global_config: GlobalCacheConfig,
    
    /// Cache statistics
    pub stats: CacheStats,
}

/// Collection cache information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionCacheInfo {
    /// Collection name
    pub name: String,
    
    /// Last indexed timestamp
    pub last_indexed: DateTime<Utc>,
    
    /// Number of files in collection
    pub file_count: usize,
    
    /// Number of vectors in collection
    pub vector_count: usize,
    
    /// File hash information
    pub file_hashes: HashMap<PathBuf, FileHashInfo>,
    
    /// Embedding model used
    pub embedding_model: String,
    
    /// Embedding model version
    pub embedding_version: String,
    
    /// Indexing strategy used
    pub indexing_strategy: IndexingStrategy,
    
    /// Collection-specific configuration hash
    pub config_hash: String,
}

/// File hash information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashInfo {
    /// Content hash (SHA-256)
    pub content_hash: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// File modification time
    pub modified_time: DateTime<Utc>,
    
    /// Number of processed chunks
    pub processed_chunks: usize,
    
    /// Vector IDs associated with this file
    pub vector_ids: Vec<String>,
    
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
}

/// Indexing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexingStrategy {
    /// Complete reindexing
    Full,
    /// Only changed files
    Incremental,
    /// Smart combination
    Hybrid,
}

/// Global cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCacheConfig {
    /// Default embedding model
    pub default_embedding_model: String,
    
    /// Default chunk size
    pub default_chunk_size: usize,
    
    /// Default chunk overlap
    pub default_chunk_overlap: usize,
    
    /// Cache compression enabled
    pub compression_enabled: bool,
    
    /// Cache validation enabled
    pub validation_enabled: bool,
}

impl Default for GlobalCacheConfig {
    fn default() -> Self {
        Self {
            default_embedding_model: "bm25".to_string(),
            default_chunk_size: 512,
            default_chunk_overlap: 50,
            compression_enabled: true,
            validation_enabled: true,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache size in bytes
    pub total_size_bytes: u64,
    
    /// Number of cache entries
    pub entry_count: usize,
    
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f32,
    
    /// Number of cache hits
    pub hits: u64,
    
    /// Number of cache misses
    pub misses: u64,
    
    /// Last cleanup timestamp
    pub last_cleanup: Option<DateTime<Utc>>,
    
    /// Cache creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,
}

impl Default for CacheStats {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            total_size_bytes: 0,
            entry_count: 0,
            hit_rate: 0.0,
            hits: 0,
            misses: 0,
            last_cleanup: None,
            created_at: now,
            last_accessed: now,
        }
    }
}

impl CacheMetadata {
    /// Create new cache metadata
    pub fn new(version: String) -> Self {
        let now = Utc::now();
        Self {
            version,
            created_at: now,
            last_updated: now,
            collections: HashMap::new(),
            global_config: GlobalCacheConfig::default(),
            stats: CacheStats::default(),
        }
    }
    
    /// Update last accessed timestamp
    pub fn update_access(&mut self) {
        self.stats.last_accessed = Utc::now();
    }
    
    /// Update last updated timestamp
    pub fn update_modified(&mut self) {
        self.last_updated = Utc::now();
    }
    
    /// Add or update collection cache info
    pub fn update_collection(&mut self, collection_info: CollectionCacheInfo) {
        self.collections.insert(collection_info.name.clone(), collection_info);
        self.update_modified();
    }
    
    /// Get collection cache info
    pub fn get_collection(&self, name: &str) -> Option<&CollectionCacheInfo> {
        self.collections.get(name)
    }
    
    /// Remove collection cache info
    pub fn remove_collection(&mut self, name: &str) -> Option<CollectionCacheInfo> {
        let result = self.collections.remove(name);
        if result.is_some() {
            self.update_modified();
        }
        result
    }
    
    /// Check if collection exists in cache
    pub fn has_collection(&self, name: &str) -> bool {
        self.collections.contains_key(name)
    }
    
    /// Get all collection names
    pub fn collection_names(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }
    
    /// Update cache statistics
    pub fn update_stats(&mut self, stats: CacheStats) {
        self.stats = stats;
        self.update_modified();
    }
    
    /// Calculate total cache size
    pub fn calculate_total_size(&self) -> u64 {
        self.collections.values()
            .map(|info| {
                info.file_hashes.values()
                    .map(|file_info| file_info.size)
                    .sum::<u64>()
            })
            .sum()
    }
    
    /// Get cache age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = Utc::now();
        now.signed_duration_since(self.created_at).num_seconds() as u64
    }
    
    /// Check if cache is stale based on TTL
    pub fn is_stale(&self, ttl_seconds: u64) -> bool {
        self.age_seconds() > ttl_seconds
    }
}

impl CollectionCacheInfo {
    /// Create new collection cache info
    pub fn new(name: String, embedding_model: String, embedding_version: String) -> Self {
        let now = Utc::now();
        Self {
            name,
            last_indexed: now,
            file_count: 0,
            vector_count: 0,
            file_hashes: HashMap::new(),
            embedding_model,
            embedding_version,
            indexing_strategy: IndexingStrategy::Incremental,
            config_hash: String::new(),
        }
    }
    
    /// Update file hash info
    pub fn update_file_hash(&mut self, file_path: PathBuf, file_info: FileHashInfo) {
        self.file_hashes.insert(file_path, file_info);
        self.file_count = self.file_hashes.len();
    }
    
    /// Remove file hash info
    pub fn remove_file_hash(&mut self, file_path: &PathBuf) -> Option<FileHashInfo> {
        let result = self.file_hashes.remove(file_path);
        if result.is_some() {
            self.file_count = self.file_hashes.len();
        }
        result
    }
    
    /// Get file hash info
    pub fn get_file_hash(&self, file_path: &PathBuf) -> Option<&FileHashInfo> {
        self.file_hashes.get(file_path)
    }
    
    /// Check if file exists in cache
    pub fn has_file(&self, file_path: &PathBuf) -> bool {
        self.file_hashes.contains_key(file_path)
    }
    
    /// Update last indexed timestamp
    pub fn update_indexed(&mut self) {
        self.last_indexed = Utc::now();
    }
    
    /// Calculate collection size
    pub fn calculate_size(&self) -> u64 {
        self.file_hashes.values().map(|info| info.size).sum()
    }
    
    /// Get collection age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = Utc::now();
        now.signed_duration_since(self.last_indexed).num_seconds() as u64
    }
    
    /// Check if collection is stale based on TTL
    pub fn is_stale(&self, ttl_seconds: u64) -> bool {
        self.age_seconds() > ttl_seconds
    }
}

impl FileHashInfo {
    /// Create new file hash info
    pub fn new(
        content_hash: String,
        size: u64,
        modified_time: DateTime<Utc>,
        processed_chunks: usize,
        vector_ids: Vec<String>,
    ) -> Self {
        Self {
            content_hash,
            size,
            modified_time,
            processed_chunks,
            vector_ids,
            processed_at: Utc::now(),
        }
    }
    
    /// Check if file has been modified since last processing
    pub fn is_modified(&self, current_modified_time: DateTime<Utc>) -> bool {
        current_modified_time > self.modified_time
    }
    
    /// Update processing timestamp
    pub fn update_processed(&mut self) {
        self.processed_at = Utc::now();
    }
    
    /// Get processing age in seconds
    pub fn processing_age_seconds(&self) -> u64 {
        let now = Utc::now();
        now.signed_duration_since(self.processed_at).num_seconds() as u64
    }
}
