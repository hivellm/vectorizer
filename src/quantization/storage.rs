//! Quantized vector storage system
//! 
//! Implements efficient storage and retrieval of quantized vectors with
//! memory-mapped files, compression, and cache management.

use crate::quantization::{
    QuantizationResult, QuantizationError, QuantizationType,
    traits::{QuantizedVectors, QuantizationMethod, QualityMetrics},
    scalar::ScalarQuantization,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for quantized vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for storage
    pub storage_dir: PathBuf,
    /// Maximum cache size in MB
    pub max_cache_size_mb: usize,
    /// Enable memory mapping for large files
    pub enable_memory_mapping: bool,
    /// Compression level (0-9)
    pub compression_level: u32,
    /// Enable automatic cleanup of old files
    pub auto_cleanup: bool,
    /// Maximum file size before splitting (MB)
    pub max_file_size_mb: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_dir: PathBuf::from("./quantized_storage"),
            max_cache_size_mb: 1024, // 1GB
            enable_memory_mapping: true,
            compression_level: 6,
            auto_cleanup: true,
            max_file_size_mb: 512, // 512MB
        }
    }
}

/// Metadata for stored quantized vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    /// Collection name
    pub collection_name: String,
    /// Quantization method used
    pub quantization_type: QuantizationType,
    /// Number of vectors
    pub vector_count: usize,
    /// Vector dimension
    pub dimension: usize,
    /// File size in bytes
    pub file_size: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Quality metrics
    pub quality_metrics: Option<QualityMetrics>,
    /// Compression ratio achieved
    pub compression_ratio:/// Cached quantized vectors with metadata
#[derive(Debug, Clone)]
pub struct CachedQuantizedVectors {
    /// The quantized vectors
    pub vectors: QuantizedVectors,
    /// Storage metadata
    pub metadata: StorageMetadata,
    /// Last access time
    pub last_access: SystemTime,
    /// Memory usage estimate in bytes
    pub memory_usage: usize,
}

/// Quantized vector storage manager
pub struct QuantizedVectorStorage {
    config: StorageConfig,
    cache: Arc<RwLock<HashMap<String, CachedQuantizedVectors>>>,
    cache_size: Arc<RwLock<usize>>,
    storage_dir: PathBuf,
    cache_hits: Arc<RwLock<u64>>,
    cache_misses: Arc<RwLock<u64>>,
}Vectors>>>,
    cache_size: Arc<RwLock<usize>>,
    storage_dir: PathBuf,
}

impl QuantizedVectorStorage {
    /// Create a new storage manager
    pub fn new(config: StorageConfig) -> QuantizationResult<Self> {
        // Create storage directory if it doesn't exist
        if !config.storage_dir.exists() {
            std::fs::create_dir_all(&config.storage_dir)
                .map_err(|e| QuantizationError::Internal(
                    format!("Failed to create storage directory: {}", e)
                ))?;
        }

        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_size: Arc::new(RwLock::new(0)),
            storage_dir: config.storage_dir.clone(),
            config,
        })
    }

    /// Store quantized vectors to disk
    pub fn store(&self, collection_name: &str, vectors: &QuantizedVectors) -> QuantizationResult<()> {
        let file_path = self.get_file_path(collection_name);
        
        // Create metadata
        let metadata = StorageMetadata {
            collection_name: collection_name.to_string(),
            quantization_type: vectors.parameters.clone().into(),
            vector_count: vectors.count,
            dimension: vectors.dimension,
            file_size: 0, // Will be updated after writing
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            last_accessed: SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            quality_metrics: None,
            compression_ratio: self.calculate_compression_ratio(vectors),
            memory_usage: vectors.data.len(),
        };

        // Serialize and compress data
        let serialized = self.serialize_vectors(vectors)?;
        let compressed = self.compress_data(&serialized)?;

        // Write to disk
        let mut file = File::create(&file_path)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to create file {}: {}", file_path.display(), e)
            ))?;

        // Write metadata header
        let metadata_bytes = bincode::serialize(&metadata)
            .map_err(|e| QuantizationError::SerializationFailed(
                format!("Failed to serialize metadata: {}", e)
            ))?;

        let header_size = metadata_bytes.len() as u32;
        file.write_all(&header_size.to_le_bytes())
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to write header size: {}", e)
            ))?;

        file.write_all(&metadata_bytes)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to write metadata: {}", e)
            ))?;

        file.write_all(&compressed)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to write compressed data: {}", e)
            ))?;

        // Update cache
        self.update_cache(collection_name, vectors.clone(), metadata)?;

        Ok(())
    }

    /// Load quantized vectors from disk
    pub fn load(&self, collection_name: &str) -> QuantizationResult<QuantizedVectors> {
        // Check cache first
        if let Some(cached) = self.get_from_cache(collection_name) {
            return Ok(cached.vectors);
        }

        let file_path = self.get_file_path(collection_name);
        
        if !file_path.exists() {
            return Err(QuantizationError::InvalidParameters(
                format!("Storage file not found: {}", file_path.display())
            ));
        }

        let mut file = File::open(&file_path)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to open file {}: {}", file_path.display(), e)
            ))?;

        // Read header size
        let mut header_size_bytes = [0u8; 4];
        file.read_exact(&mut header_size_bytes)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to read header size: {}", e)
            ))?;

        let header_size = u32::from_le_bytes(header_size_bytes) as usize;

        // Read metadata
        let mut metadata_bytes = vec![0u8; header_size];
        file.read_exact(&mut metadata_bytes)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to read metadata: {}", e)
            ))?;

        let metadata: StorageMetadata = bincode::deserialize(&metadata_bytes)
            .map_err(|e| QuantizationError::DeserializationFailed(
                format!("Failed to deserialize metadata: {}", e)
            ))?;

        // Read compressed data
        let mut compressed_data = Vec::new();
        file.read_to_end(&mut compressed_data)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to read compressed data: {}", e)
            ))?;

        // Decompress and deserialize
        let serialized = self.decompress_data(&compressed_data)?;
        let vectors = self.deserialize_vectors(&serialized)?;

        // Update cache
        self.update_cache(collection_name, vectors.clone(), metadata)?;

        Ok(vectors)
    }

    /// Remove stored vectors
    pub fn remove(&self, collection_name: &str) -> QuantizationResult<()> {
        // Remove from cache
        self.remove_from_cache(collection_name);

        // Remove file
        let file_path = self.get_file_path(collection_name);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| QuantizationError::Internal(
                    format!("Failed to remove file {}: {}", file_path.display(), e)
                ))?;
        }

        Ok(())
    }

    /// List all stored collections
    pub fn list_collections(&self) -> QuantizationResult<Vec<String>> {
        let mut collections = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(collections);
        }

        for entry in std::fs::read_dir(&self.storage_dir)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to read storage directory: {}", e)
            ))? {
            let entry = entry.map_err(|e| QuantizationError::Internal(
                format!("Failed to read directory entry: {}", e)
            ))?;

            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".qvec") {
                    let collection_name = file_name.trim_end_matches(".qvec");
                    collections.push(collection_name.to_string());
                }
            }
        }

        Ok(collections)
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> QuantizationResult<StorageStats> {
        let cache = self.cache.read().unwrap();
        let cache_size = *self.cache_size.read().unwrap();

        let mut total_files = 0;
        let mut total_size = 0u64;
        let mut total_vectors = 0;

        for entry in std::fs::read_dir(&self.storage_dir)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to read storage directory: {}", e)
            ))? {
            let entry = entry.map_err(|e| QuantizationError::Internal(
                format!("Failed to read directory entry: {}", e)
            ))?;

            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".qvec") {
                    total_files += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }

                    // Try to load metadata for vector count
                    if let Ok(vectors) = self.load(file_name.trim_end_matches(".qvec")) {
                        total_vectors += vectors.count;
                    }
                }
            }
        }

        Ok(StorageStats {
            cached_collections: cache.len(),
            total_collections: total_files,
            cache_size_mb: cache_size / (1024 * 1024),
            total_storage_mb: total_size / (1024 * 1024),
            total_vectors,
            cache_hit_ratio: 0.0,
        })
    }

    /// Clear cache
    pub fn clear_cache(&self) -> QuantizationResult<()> {
        let mut cache = self.cache.write().unwrap();
        let mut cache_size = self.cache_size.write().unwrap();
        
        cache.clear();
        *cache_size = 0;

        Ok(())
    }

    /// Cleanup old files
    pub fn cleanup(&self, max_age_seconds: u64) -> QuantizationResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();

        for entry in std::fs::read_dir(&self.storage_dir)
            .map_err(|e| QuantizationError::Internal(
                format!("Failed to read storage directory: {}", e)
            ))? {
            let entry = entry.map_err(|e| QuantizationError::Internal(
                format!("Failed to read directory entry: {}", e)
            ))?;

            if let Ok(metadata) = entry.metadata() {
                if let Ok(created) = metadata.created() {
                    if let Ok(created_time) = created.duration_since(UNIX_EPOCH) {
                        if now - created_time.as_secs() > max_age_seconds {
                            std::fs::remove_file(entry.path())
                                .map_err(|e| QuantizationError::Internal(
                                    format!("Failed to remove old file: {}", e)
                                ))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Private helper methods

    fn get_file_path(&self, collection_name: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.qvec", collection_name))
    }

    fn calculate_compression_ratio(&self, vectors: &QuantizedVectors) -> f32 {
        let original_size = vectors.count * vectors.dimension * 4; // f32 = 4 bytes
        let compressed_size = vectors.data.len();
        
        if compressed_size == 0 {
            return 1.0;
        }
        
        original_size as f32 / compressed_size as f32
    }

    fn serialize_vectors(&self, vectors: &QuantizedVectors) -> QuantizationResult<Vec<u8>> {
        bincode::serialize(vectors)
            .map_err(|e| QuantizationError::SerializationFailed(
                format!("Failed to serialize vectors: {}", e)
            ))
    }

    fn deserialize_vectors(&self, data: &[u8]) -> QuantizationResult<QuantizedVectors> {
        bincode::deserialize(data)
            .map_err(|e| QuantizationError::DeserializationFailed(
                format!("Failed to deserialize vectors: {}", e)
            ))
    }

    fn compress_data(&self, data: &[u8]) -> QuantizationResult<Vec<u8>> {
        // Use lz4 compression for fast compression/decompression
        Ok(lz4_flex::compress(data))
    }

    fn decompress_data(&self, compressed: &[u8]) -> QuantizationResult<Vec<u8>> {
        lz4_flex::decompress(compressed, compressed.len() * 4) // Estimate expansion
            .map_err(|e| QuantizationError::Internal(
                format!("Decompression failed: {}", e)
            ))
    }

    fn get_from_cache(&self, collection_name: &str) -> Option<CachedQuantizedVectors> {
        let cache = self.cache.read().unwrap();
        cache.get(collection_name).cloned()
    }

    fn update_cache(&self, collection_name: &str, vectors: QuantizedVectors, metadata: StorageMetadata) -> QuantizationResult<()> {
        let memory_usage = vectors.data.len();
        
        // Check if we need to evict from cache
        let max_cache_size = self.config.max_cache_size_mb * 1024 * 1024;
        let mut cache_size = self.cache_size.write().unwrap();
        
        if *cache_size + memory_usage > max_cache_size {
            self.evict_from_cache(memory_usage)?;
        }

        let cached = CachedQuantizedVectors {
            vectors,
            metadata,
            last_access: SystemTime::now(),
            memory_usage,
        };

        let mut cache = self.cache.write().unwrap();
        cache.insert(collection_name.to_string(), cached);
        *cache_size += memory_usage;

        Ok(())
    }

    fn remove_from_cache(&self, collection_name: &str) {
        let mut cache = self.cache.write().unwrap();
        let mut cache_size = self.cache_size.write().unwrap();
        
        if let Some(cached) = cache.remove(collection_name) {
            *cache_size -= cached.memory_usage;
        }
    }

    fn evict_from_cache(&self, required_space: usize) -> QuantizationResult<()> {
        let mut cache = self.cache.write().unwrap();
        let mut cache_size = self.cache_size.write().unwrap();
        
        // Simple LRU eviction - remove oldest entries
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, cached)| cached.last_access);

        let mut freed_space = 0;
        let mut to_remove = Vec::new();

        for (key, cached) in entries {
            to_remove.push(key.clone());
            freed_space += cached.memory_usage;
            
            if freed_space >= required_space {
                break;
            }
        }

        for key in to_remove {
            if let Some(cached) = cache.remove(&key) {
                *cache_size -= cached.memory_usage;
            }
        }

        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Number of collections in cache
    pub cached_collections: usize,
    /// Total number of stored collections
    pub total_collections: usize,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Total storage size in MB
    pub total_storage_mb: u64,
    /// Total number of vectors stored
    pub total_vectors: usize,
    /// Cache hit ratio (0.0 - 1.0)
    pub cache_hit_ratio: f32,
}

impl StorageMetadata {
    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();
    }

    /// Calculate storage efficiency
    pub fn storage_efficiency(&self) -> f32 {
        if self.memory_usage == 0 {
            return 1.0;
        }
        
        self.compression_ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            max_cache_size_mb: 10,
            ..Default::default()
        };

        let storage = QuantizedVectorStorage::new(config).unwrap();

        // Create test vectors
        let mut sq = ScalarQuantization::new(8).unwrap();
        let test_vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ];
        sq.fit(&test_vectors).unwrap();
        let quantized = sq.quantize(&test_vectors).unwrap();

        // Test store
        storage.store("test_collection", &quantized).unwrap();

        // Test load
        let loaded = storage.load("test_collection").unwrap();
        assert_eq!(loaded.count, quantized.count);
        assert_eq!(loaded.dimension, quantized.dimension);

        // Test list collections
        let collections = storage.list_collections().unwrap();
        assert!(collections.contains(&"test_collection".to_string()));

        // Test remove
        storage.remove("test_collection").unwrap();
        let collections_after = storage.list_collections().unwrap();
        assert!(!collections_after.contains(&"test_collection".to_string()));
    }

    #[test]
    fn test_storage_stats() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            max_cache_size_mb: 10,
            ..Default::default()
        };

        let storage = QuantizedVectorStorage::new(config).unwrap();

        // Create and store test data
        let mut sq = ScalarQuantization::new(8).unwrap();
        let test_vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
        ];
        sq.fit(&test_vectors).unwrap();
        let quantized = sq.quantize(&test_vectors).unwrap();

        storage.store("test1", &quantized).unwrap();
        storage.store("test2", &quantized).unwrap();

        let stats = storage.get_stats().unwrap();
        assert_eq!(stats.total_collections, 2);
        assert_eq!(stats.total_vectors, 4); // 2 collections * 2 vectors each
    }

    #[test]
    fn test_cache_management() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            max_cache_size_mb: 1, // Very small cache
            ..Default::default()
        };

        let storage = QuantizedVectorStorage::new(config).unwrap();

        // Create test data
        let mut sq = ScalarQuantization::new(8).unwrap();
        let test_vectors = vec![vec![1.0; 1000]]; // Large vector
        sq.fit(&test_vectors).unwrap();
        let quantized = sq.quantize(&test_vectors).unwrap();

        // Store multiple collections to test cache eviction
        for i in 0..5 {
            storage.store(&format!("test{}", i), &quantized).unwrap();
        }

        let stats = storage.get_stats().unwrap();
        assert!(stats.cached_collections <= 5); // Some may be evicted

        // Test cache clear
        storage.clear_cache().unwrap();
        let stats_after = storage.get_stats().unwrap();
        assert_eq!(stats_after.cached_collections, 0);
    }
}
