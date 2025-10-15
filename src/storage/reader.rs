//! Storage reader for reading from .vecdb archives

use crate::error::{Result, VectorizerError};
use crate::storage::{StorageIndex, CollectionIndex};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use zip::ZipArchive;

/// Reader for accessing .vecdb archives
pub struct StorageReader {
    /// Path to .vecdb file
    vecdb_path: PathBuf,
    
    /// Cached storage index
    index: Arc<RwLock<StorageIndex>>,
    
    /// Cache for decompressed files
    file_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    
    /// Maximum cache size in bytes
    max_cache_size: usize,
}

impl StorageReader {
    /// Create a new storage reader
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let vecdb_path = data_dir.as_ref().join(crate::storage::VECDB_FILE);
        let vecidx_path = data_dir.as_ref().join(crate::storage::VECIDX_FILE);
        
        if !vecdb_path.exists() {
            return Err(VectorizerError::Storage(
                format!("Archive not found: {}", vecdb_path.display())
            ));
        }
        
        if !vecidx_path.exists() {
            return Err(VectorizerError::Storage(
                format!("Index not found: {}", vecidx_path.display())
            ));
        }
        
        let index = StorageIndex::load(&vecidx_path)?;
        
        Ok(Self {
            vecdb_path,
            index: Arc::new(RwLock::new(index)),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size: 100 * 1024 * 1024, // 100MB default cache
        })
    }
    
    /// Get the storage index
    pub fn index(&self) -> Result<StorageIndex> {
        self.index.read()
            .map(|guard| guard.clone())
            .map_err(|_| VectorizerError::Storage("Failed to read index".to_string()))
    }
    
    /// List all collection names
    pub fn list_collections(&self) -> Result<Vec<String>> {
        let index = self.index.read()
            .map_err(|_| VectorizerError::Storage("Failed to read index".to_string()))?;
        
        Ok(index.collections.iter().map(|c| c.name.clone()).collect())
    }
    
    /// Get collection metadata
    pub fn get_collection(&self, name: &str) -> Result<Option<CollectionIndex>> {
        let index = self.index.read()
            .map_err(|_| VectorizerError::Storage("Failed to read index".to_string()))?;
        
        Ok(index.find_collection(name).cloned())
    }
    
    /// Read a file from the archive
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // Check cache first
        {
            let cache = self.file_cache.read()
                .map_err(|_| VectorizerError::Storage("Failed to read cache".to_string()))?;
            
            if let Some(data) = cache.get(path) {
                return Ok(data.clone());
            }
        }
        
        // Read from archive
        let data = self.read_file_from_archive(path)?;
        
        // Cache the result if not too large
        if data.len() < self.max_cache_size / 10 {
            let mut cache = self.file_cache.write()
                .map_err(|_| VectorizerError::Storage("Failed to write cache".to_string()))?;
            
            cache.insert(path.to_string(), data.clone());
            
            // Simple cache eviction if too large
            let total_size: usize = cache.values().map(|v| v.len()).sum();
            if total_size > self.max_cache_size {
                cache.clear();
            }
        }
        
        Ok(data)
    }
    
    /// Read file directly from archive without caching
    fn read_file_from_archive(&self, path: &str) -> Result<Vec<u8>> {
        let file = File::open(&self.vecdb_path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let mut archive = ZipArchive::new(file)
            .map_err(|e| VectorizerError::Storage(format!("Failed to open archive: {}", e)))?;
        
        let mut zip_file = archive.by_name(path)
            .map_err(|e| VectorizerError::Storage(format!("File not found in archive: {}", e)))?;
        
        let mut buffer = Vec::new();
        zip_file.read_to_end(&mut buffer)
            .map_err(|e| VectorizerError::Io(e))?;
        
        Ok(buffer)
    }
    
    /// Read all files for a collection
    pub fn read_collection_files(&self, collection_name: &str) -> Result<HashMap<String, Vec<u8>>> {
        let collection = self.get_collection(collection_name)?
            .ok_or_else(|| VectorizerError::Storage(format!("Collection not found: {}", collection_name)))?;
        
        let mut files = HashMap::new();
        
        for file_entry in &collection.files {
            let data = self.read_file(&file_entry.path)?;
            files.insert(file_entry.path.clone(), data);
        }
        
        Ok(files)
    }
    
    /// Clear the file cache
    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self.file_cache.write()
            .map_err(|_| VectorizerError::Storage("Failed to write cache".to_string()))?;
        
        cache.clear();
        Ok(())
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<CacheStats> {
        let cache = self.file_cache.read()
            .map_err(|_| VectorizerError::Storage("Failed to read cache".to_string()))?;
        
        let entry_count = cache.len();
        let total_size: usize = cache.values().map(|v| v.len()).sum();
        
        Ok(CacheStats {
            entry_count,
            total_size_bytes: total_size,
            max_size_bytes: self.max_cache_size,
        })
    }
    
    /// Extract all collections from archive in memory (no temp files)
    pub fn extract_all_collections(&self) -> Result<Vec<crate::persistence::PersistedCollection>> {
        use tracing::info;
        
        info!("📦 Extracting all collections from compressed archive in memory...");
        
        let collections_list = self.list_collections()?;
        let mut persisted_collections = Vec::new();
        
        for collection_name in &collections_list {
            if let Some(collection) = self.read_collection_in_memory(collection_name)? {
                persisted_collections.push(collection);
            }
        }
        
        info!("✅ Extracted {} collections from archive (no temp files created)", persisted_collections.len());
        
        Ok(persisted_collections)
    }
    
    /// Read a collection directly in memory without creating temp files
    pub fn read_collection_in_memory(&self, collection_name: &str) -> Result<Option<crate::persistence::PersistedCollection>> {
        use tracing::{debug, warn};
        
        let collection_index = match self.get_collection(collection_name)? {
            Some(c) => c,
            None => return Ok(None),
        };
        
        debug!("Reading collection '{}' from archive", collection_name);
        
        // Read all files for this collection into memory
        let files = self.read_collection_files(collection_name)?;
        
        // Find the vector_store.bin file
        let vector_store_path = format!("{}_vector_store.bin", collection_name);
        let vector_data = files.get(&vector_store_path)
            .ok_or_else(|| VectorizerError::Storage(format!("Vector store file not found for collection: {}", collection_name)))?;
        
        // Files are saved as JSON, not bincode
        let json_str = std::str::from_utf8(vector_data)
            .map_err(|e| VectorizerError::Deserialization(format!("Invalid UTF-8 in vector store: {}", e)))?;
        
        // Deserialize PersistedVectorStore (which contains Vec<PersistedCollection>)
        let persisted_store: crate::persistence::PersistedVectorStore = serde_json::from_str(json_str)
            .map_err(|e| VectorizerError::Deserialization(format!("Failed to deserialize collection: {}", e)))?;
        
        // Extract the first collection (files are saved as PersistedVectorStore with one collection)
        let mut persisted = persisted_store.collections.into_iter().next()
            .ok_or_else(|| VectorizerError::Storage(format!("No collection found in vector store file: {}", vector_store_path)))?;
        
        // BACKWARD COMPATIBILITY: If name is empty, infer from filename
        if persisted.name.is_empty() {
            debug!("⚠️  Collection name missing in persisted data, inferring from filename: '{}'", collection_name);
            persisted.name = collection_name.to_string();
        }
        
        // BACKWARD COMPATIBILITY: If config is missing, load from metadata file
        if persisted.config.is_none() {
            debug!("⚠️  Collection config missing, attempting to load from metadata file...");
            let metadata_path = format!("{}_metadata.json", collection_name);
            
            if let Some(metadata_data) = files.get(&metadata_path) {
                let metadata_str = std::str::from_utf8(metadata_data)
                    .map_err(|e| VectorizerError::Deserialization(format!("Invalid UTF-8 in metadata: {}", e)))?;
                
                // Metadata file structure
                #[derive(serde::Deserialize)]
                struct Metadata {
                    config: crate::models::CollectionConfig,
                }
                
                match serde_json::from_str::<Metadata>(metadata_str) {
                    Ok(metadata) => {
                        debug!("✅ Loaded config from metadata file");
                        persisted.config = Some(metadata.config);
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to load config from metadata: {}", e);
                        // Create default config as fallback
                        debug!("⚠️  Using default config as fallback");
                        let dimension = if let Some(first_vec) = persisted.vectors.first() {
                            // Try to get dimension from first vector using into_runtime
                            match first_vec.clone().into_runtime() {
                                Ok(v) => v.data.len(),
                                Err(_) => 384 // Default fallback
                            }
                        } else {
                            384 // Default dimension
                        };
                        
                        persisted.config = Some(crate::models::CollectionConfig {
                            dimension,
                            metric: crate::models::DistanceMetric::Cosine,
                            hnsw_config: crate::models::HnswConfig {
                                m: 32,
                                ef_construction: 100,
                                ef_search: 50,
                                seed: None,
                            },
                            quantization: crate::models::QuantizationConfig::None,
                            compression: crate::models::CompressionConfig {
                                enabled: false,
                                threshold_bytes: 1024,
                                algorithm: crate::models::CompressionAlgorithm::Lz4,
                            },
                            normalization: None,
                        });
                    }
                }
            } else {
                warn!("⚠️  Metadata file not found, using default config");
                let dimension = if let Some(first_vec) = persisted.vectors.first() {
                    // Try to get dimension from first vector using into_runtime
                    match first_vec.clone().into_runtime() {
                        Ok(v) => v.data.len(),
                        Err(_) => 384 // Default fallback
                    }
                } else {
                    384
                };
                
                persisted.config = Some(crate::models::CollectionConfig {
                    dimension,
                    metric: crate::models::DistanceMetric::Cosine,
                    hnsw_config: crate::models::HnswConfig {
                        m: 32,
                        ef_construction: 100,
                        ef_search: 50,
                        seed: None,
                    },
                    quantization: crate::models::QuantizationConfig::None,
                    compression: crate::models::CompressionConfig {
                        enabled: false,
                        threshold_bytes: 1024,
                        algorithm: crate::models::CompressionAlgorithm::Lz4,
                    },
                    normalization: None,
                });
            }
        }
        
        debug!("✅ Collection '{}' loaded with {} vectors", persisted.name, persisted.vectors.len());
        
        Ok(Some(persisted))
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub entry_count: usize,
    
    /// Total size of cached data in bytes
    pub total_size_bytes: usize,
    
    /// Maximum cache size in bytes
    pub max_size_bytes: usize,
}

impl CacheStats {
    /// Get cache utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.max_size_bytes == 0 {
            0.0
        } else {
            (self.total_size_bytes as f64 / self.max_size_bytes as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::storage::StorageWriter;
    use std::fs;
    use std::io::Write;

    fn create_test_archive(data_dir: &Path) -> Result<()> {
        let collections_dir = data_dir.join("collections");
        let collection_dir = collections_dir.join("test_collection");
        fs::create_dir_all(&collection_dir)?;
        
        let test_file = collection_dir.join("test.bin");
        let mut file = File::create(&test_file)?;
        file.write_all(b"test vector data")?;
        
        let writer = StorageWriter::new(data_dir, 3);
        writer.write_archive(&collections_dir)?;
        
        Ok(())
    }

    #[test]
    fn test_reader_creation() {
        let temp_dir = TempDir::new().unwrap();
        create_test_archive(temp_dir.path()).unwrap();
        
        let reader = StorageReader::new(temp_dir.path());
        assert!(reader.is_ok());
    }

    #[test]
    fn test_list_collections() {
        let temp_dir = TempDir::new().unwrap();
        create_test_archive(temp_dir.path()).unwrap();
        
        let reader = StorageReader::new(temp_dir.path()).unwrap();
        let collections = reader.list_collections().unwrap();
        
        assert!(collections.len() > 0);
    }

    #[test]
    fn test_get_collection() {
        let temp_dir = TempDir::new().unwrap();
        create_test_archive(temp_dir.path()).unwrap();
        
        let reader = StorageReader::new(temp_dir.path()).unwrap();
        let collection = reader.get_collection("test_collection").unwrap();
        
        assert!(collection.is_some());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        create_test_archive(temp_dir.path()).unwrap();
        
        let reader = StorageReader::new(temp_dir.path()).unwrap();
        let stats = reader.cache_stats().unwrap();
        
        assert_eq!(stats.entry_count, 0); // Initially empty
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_test_archive(temp_dir.path()).unwrap();
        
        let reader = StorageReader::new(temp_dir.path()).unwrap();
        assert!(reader.clear_cache().is_ok());
    }
}

