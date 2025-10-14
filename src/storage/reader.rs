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

