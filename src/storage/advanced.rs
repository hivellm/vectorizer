//! Advanced storage options for vector storage
//!
//! This module provides:
//! - On-disk vector storage with memory-mapped files
//! - Memory-mapped storage for efficient access
//! - Storage optimization (compaction, defragmentation)
//! - Storage logging and metrics

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use memmap2::{Mmap, MmapMut, MmapOptions};
use parking_lot::RwLock as ParkingRwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::error::{Result, VectorizerError};
use crate::models::Vector;
use crate::storage::config::StorageConfig;

/// Advanced storage manager for on-disk vector storage
pub struct AdvancedStorage {
    /// Base directory for storage files
    base_path: PathBuf,
    /// Storage configuration
    config: StorageConfig,
    /// Memory-mapped file cache
    mmap_cache: Arc<ParkingRwLock<HashMap<String, Arc<Mmap>>>>,
    /// Storage statistics
    stats: Arc<ParkingRwLock<StorageStats>>,
    /// Logging enabled flag
    logging_enabled: bool,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total vectors stored
    pub total_vectors: usize,
    /// Total storage size in bytes
    pub total_size_bytes: usize,
    /// Memory-mapped files count
    pub mmap_files_count: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Read operations
    pub read_ops: usize,
    /// Write operations
    pub write_ops: usize,
    /// Compaction operations
    pub compaction_ops: usize,
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            total_vectors: 0,
            total_size_bytes: 0,
            mmap_files_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            read_ops: 0,
            write_ops: 0,
            compaction_ops: 0,
        }
    }
}

impl AdvancedStorage {
    /// Create a new advanced storage manager
    pub fn new(base_path: impl AsRef<Path>, config: StorageConfig) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Ensure base directory exists
        std::fs::create_dir_all(&base_path).map_err(|e| VectorizerError::Io(e))?;

        info!(
            "📦 Initializing advanced storage at: {}",
            base_path.display()
        );

        Ok(Self {
            base_path,
            config,
            mmap_cache: Arc::new(ParkingRwLock::new(HashMap::new())),
            stats: Arc::new(ParkingRwLock::new(StorageStats::default())),
            logging_enabled: true,
        })
    }

    /// Store vectors on disk with memory-mapped access
    pub fn store_vectors(&self, collection_name: &str, vectors: &[Vector]) -> Result<()> {
        if self.logging_enabled {
            info!(
                "💾 Storing {} vectors for collection '{}'",
                vectors.len(),
                collection_name
            );
        }

        let file_path = self
            .base_path
            .join(format!("{}_vectors.bin", collection_name));

        // Create or open file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| VectorizerError::Io(e))?;

        // Serialize vectors
        let serialized = bincode::serialize(vectors).map_err(|e| {
            VectorizerError::Serialization(format!("Failed to serialize vectors: {}", e))
        })?;

        // Write to file
        let mut writer = std::io::BufWriter::new(&file);
        writer
            .write_all(&serialized)
            .map_err(|e| VectorizerError::Io(e))?;
        writer.flush().map_err(|e| VectorizerError::Io(e))?;

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_vectors += vectors.len();
            stats.total_size_bytes += serialized.len();
            stats.write_ops += 1;
        }

        if self.logging_enabled {
            info!(
                "✅ Stored {} vectors ({} bytes) for collection '{}'",
                vectors.len(),
                serialized.len(),
                collection_name
            );
        }

        Ok(())
    }

    /// Load vectors from disk using memory-mapped file
    pub fn load_vectors(&self, collection_name: &str) -> Result<Vec<Vector>> {
        if self.logging_enabled {
            debug!("📖 Loading vectors for collection '{}'", collection_name);
        }

        let file_path = self
            .base_path
            .join(format!("{}_vectors.bin", collection_name));

        if !file_path.exists() {
            return Err(VectorizerError::Storage(format!(
                "Vector file not found: {}",
                file_path.display()
            )));
        }

        // Check cache first
        {
            let cache = self.mmap_cache.read();
            if let Some(mmap) = cache.get(collection_name) {
                // Update statistics
                {
                    let mut stats = self.stats.write();
                    stats.cache_hits += 1;
                    stats.read_ops += 1;
                }

                // Deserialize from memory-mapped file
                let vectors: Vec<Vector> = bincode::deserialize(&**mmap).map_err(|e| {
                    VectorizerError::Deserialization(format!(
                        "Failed to deserialize vectors: {}",
                        e
                    ))
                })?;

                if self.logging_enabled {
                    debug!(
                        "✅ Loaded {} vectors from cache for collection '{}'",
                        vectors.len(),
                        collection_name
                    );
                }

                return Ok(vectors);
            }
        }

        // Cache miss - load from disk and create memory map
        {
            let mut stats = self.stats.write();
            stats.cache_misses += 1;
            stats.read_ops += 1;
        }

        let file = File::open(&file_path).map_err(|e| VectorizerError::Io(e))?;

        // Create memory-mapped file
        let mmap = unsafe {
            MmapOptions::new().map(&file).map_err(|e| {
                VectorizerError::Storage(format!("Failed to create memory map: {}", e))
            })?
        };

        // Deserialize from memory-mapped file
        let vectors: Vec<Vector> = bincode::deserialize(&*mmap).map_err(|e| {
            VectorizerError::Deserialization(format!("Failed to deserialize vectors: {}", e))
        })?;

        // Cache the memory map
        {
            let mut cache = self.mmap_cache.write();
            cache.insert(collection_name.to_string(), Arc::new(mmap));

            let mut stats = self.stats.write();
            stats.mmap_files_count = cache.len();
        }

        if self.logging_enabled {
            info!(
                "✅ Loaded {} vectors from disk for collection '{}'",
                vectors.len(),
                collection_name
            );
        }

        Ok(vectors)
    }

    /// Optimize storage (compaction and defragmentation)
    pub fn optimize_storage(&self, collection_name: &str) -> Result<StorageOptimizationResult> {
        if self.logging_enabled {
            info!("🔧 Optimizing storage for collection '{}'", collection_name);
        }

        let file_path = self
            .base_path
            .join(format!("{}_vectors.bin", collection_name));

        if !file_path.exists() {
            return Err(VectorizerError::Storage(format!(
                "Vector file not found: {}",
                file_path.display()
            )));
        }

        // Load vectors
        let vectors = self.load_vectors(collection_name)?;

        // Get original file size
        let original_size = std::fs::metadata(&file_path)
            .map_err(|e| VectorizerError::Io(e))?
            .len() as usize;

        // Clear cache BEFORE re-storing to close memory-mapped file
        // This prevents "file with a user-mapped section open" error
        self.clear_cache(collection_name)?;

        // Re-store vectors (this will compact the file)
        self.store_vectors(collection_name, &vectors)?;

        // Get new file size
        let new_size = std::fs::metadata(&file_path)
            .map_err(|e| VectorizerError::Io(e))?
            .len() as usize;

        let space_saved = original_size.saturating_sub(new_size);
        let compression_ratio = if original_size > 0 {
            (space_saved as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.compaction_ops += 1;
        }

        let result = StorageOptimizationResult {
            collection_name: collection_name.to_string(),
            original_size_bytes: original_size,
            new_size_bytes: new_size,
            space_saved_bytes: space_saved,
            compression_ratio_percent: compression_ratio,
            vectors_count: vectors.len(),
        };

        if self.logging_enabled {
            info!(
                "✅ Storage optimization complete for '{}': saved {} bytes ({:.1}%)",
                collection_name, space_saved, compression_ratio
            );
        }

        Ok(result)
    }

    /// Clear cache for a collection
    pub fn clear_cache(&self, collection_name: &str) -> Result<()> {
        let mut cache = self.mmap_cache.write();
        cache.remove(collection_name);

        let mut stats = self.stats.write();
        stats.mmap_files_count = cache.len();

        if self.logging_enabled {
            debug!("🗑️  Cleared cache for collection '{}'", collection_name);
        }

        Ok(())
    }

    /// Clear all caches
    pub fn clear_all_caches(&self) -> Result<()> {
        let mut cache = self.mmap_cache.write();
        cache.clear();

        let mut stats = self.stats.write();
        stats.mmap_files_count = 0;

        if self.logging_enabled {
            info!("🗑️  Cleared all caches");
        }

        Ok(())
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> StorageStats {
        self.stats.read().clone()
    }

    /// Enable or disable logging
    pub fn set_logging(&mut self, enabled: bool) {
        self.logging_enabled = enabled;
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.mmap_cache.read();
        let stats = self.stats.read();

        CacheStats {
            cached_collections: cache.len(),
            total_cache_size_bytes: cache.values().map(|mmap| mmap.len()).sum(),
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            hit_rate: if stats.cache_hits + stats.cache_misses > 0 {
                (stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Storage optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimizationResult {
    /// Collection name
    pub collection_name: String,
    /// Original file size in bytes
    pub original_size_bytes: usize,
    /// New file size in bytes
    pub new_size_bytes: usize,
    /// Space saved in bytes
    pub space_saved_bytes: usize,
    /// Compression ratio percentage
    pub compression_ratio_percent: f64,
    /// Number of vectors
    pub vectors_count: usize,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached collections
    pub cached_collections: usize,
    /// Total cache size in bytes
    pub total_cache_size_bytes: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Cache hit rate percentage
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::models::Vector;
    use crate::storage::config::StorageConfig;

    fn create_test_vectors(count: usize) -> Vec<Vector> {
        (0..count)
            .map(|i| Vector {
                id: format!("vec_{}", i),
                data: vec![i as f32; 128],
                sparse: None,
                payload: None,
            })
            .collect()
    }

    #[test]
    fn test_store_and_load_vectors() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig::default();
        let storage = AdvancedStorage::new(temp_dir.path(), config).unwrap();

        let vectors = create_test_vectors(10);
        storage.store_vectors("test", &vectors).unwrap();

        let loaded = storage.load_vectors("test").unwrap();
        assert_eq!(loaded.len(), 10);
        assert_eq!(loaded[0].id, "vec_0");
    }

    #[test]
    fn test_memory_mapped_access() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig::default();
        let storage = AdvancedStorage::new(temp_dir.path(), config).unwrap();

        let vectors = create_test_vectors(100);
        storage.store_vectors("test", &vectors).unwrap();

        // First load - should be cache miss
        let loaded1 = storage.load_vectors("test").unwrap();
        assert_eq!(loaded1.len(), 100);

        // Second load - should be cache hit
        let loaded2 = storage.load_vectors("test").unwrap();
        assert_eq!(loaded2.len(), 100);

        let stats = storage.cache_stats();
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
    }

    #[test]
    fn test_storage_optimization() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig::default();
        let storage = AdvancedStorage::new(temp_dir.path(), config).unwrap();

        let vectors = create_test_vectors(50);
        storage.store_vectors("test", &vectors).unwrap();

        let result = storage.optimize_storage("test").unwrap();
        assert_eq!(result.vectors_count, 50);
        assert!(result.new_size_bytes > 0);
    }

    #[test]
    fn test_cache_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig::default();
        let storage = AdvancedStorage::new(temp_dir.path(), config).unwrap();

        let vectors = create_test_vectors(10);
        storage.store_vectors("test1", &vectors).unwrap();
        storage.store_vectors("test2", &vectors).unwrap();

        // Load to populate cache
        storage.load_vectors("test1").unwrap();
        storage.load_vectors("test2").unwrap();

        let cache_stats = storage.cache_stats();
        assert_eq!(cache_stats.cached_collections, 2);

        storage.clear_cache("test1").unwrap();
        let cache_stats = storage.cache_stats();
        assert_eq!(cache_stats.cached_collections, 1);

        storage.clear_all_caches().unwrap();
        let cache_stats = storage.cache_stats();
        assert_eq!(cache_stats.cached_collections, 0);
    }

    #[test]
    fn test_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig::default();
        let storage = AdvancedStorage::new(temp_dir.path(), config).unwrap();

        let vectors = create_test_vectors(20);
        storage.store_vectors("test", &vectors).unwrap();
        storage.load_vectors("test").unwrap();

        let stats = storage.get_stats();
        assert_eq!(stats.total_vectors, 20);
        assert!(stats.total_size_bytes > 0);
        assert!(stats.write_ops > 0);
        assert!(stats.read_ops > 0);
    }
}
