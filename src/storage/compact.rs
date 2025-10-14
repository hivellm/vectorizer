//! Storage compaction logic

use crate::error::{Result, VectorizerError};
use crate::storage::{StorageWriter, StorageReader, StorageIndex};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Storage compactor for managing .vecdb archives
pub struct StorageCompactor {
    /// Data directory
    data_dir: PathBuf,
    
    /// Compression level
    compression_level: i32,
    
    /// Batch size for incremental updates
    batch_size: usize,
    
    /// Pending operations counter
    pending_operations: usize,
}

impl StorageCompactor {
    /// Create a new storage compactor
    pub fn new(data_dir: impl AsRef<Path>, compression_level: i32, batch_size: usize) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            compression_level,
            batch_size,
            pending_operations: 0,
        }
    }
    
    /// Compact all collections into .vecdb archive
    pub fn compact_all(&self) -> Result<StorageIndex> {
        let collections_dir = self.data_dir.join("collections");
        
        if !collections_dir.exists() {
            // Try legacy 'data' directory structure
            let legacy_data = self.data_dir.parent()
                .unwrap_or(&self.data_dir)
                .join("data");
            
            if legacy_data.exists() {
                return self.compact_directory(&legacy_data);
            }
            
            return Err(VectorizerError::Storage(
                format!("Collections directory not found: {}", collections_dir.display())
            ));
        }
        
        self.compact_directory(&collections_dir)
    }
    
    /// Compact a specific directory
    fn compact_directory(&self, source_dir: &Path) -> Result<StorageIndex> {
        info!("🗜️  Starting compaction of directory: {}", source_dir.display());
        
        let writer = StorageWriter::new(&self.data_dir, self.compression_level);
        let index = writer.write_archive(source_dir)?;
        
        info!("✅ Compaction complete:");
        info!("   Collections: {}", index.collection_count());
        info!("   Total vectors: {}", index.total_vectors());
        info!("   Original size: {} MB", index.total_size / 1_048_576);
        info!("   Compressed size: {} MB", index.compressed_size / 1_048_576);
        info!("   Compression ratio: {:.2}%", index.compression_ratio * 100.0);
        
        Ok(index)
    }
    
    /// Record a new operation (insert, update, delete)
    pub fn record_operation(&mut self) {
        self.pending_operations += 1;
    }
    
    /// Check if compaction should be triggered
    pub fn should_compact(&self) -> bool {
        self.pending_operations >= self.batch_size
    }
    
    /// Trigger incremental compaction if threshold reached
    pub fn maybe_compact(&mut self) -> Result<Option<StorageIndex>> {
        if self.should_compact() {
            info!("🔄 Triggering incremental compaction ({} pending operations)", self.pending_operations);
            let index = self.compact_all()?;
            self.pending_operations = 0;
            Ok(Some(index))
        } else {
            Ok(None)
        }
    }
    
    /// Force compaction regardless of pending operations
    pub fn force_compact(&mut self) -> Result<StorageIndex> {
        let index = self.compact_all()?;
        self.pending_operations = 0;
        Ok(index)
    }
    
    /// Get the number of pending operations
    pub fn pending_operations(&self) -> usize {
        self.pending_operations
    }
    
    /// Reset pending operations counter
    pub fn reset_counter(&mut self) {
        self.pending_operations = 0;
    }
    
    /// Verify archive integrity
    pub fn verify_integrity(&self) -> Result<bool> {
        info!("🔍 Verifying archive integrity...");
        
        let reader = StorageReader::new(&self.data_dir)?;
        let index = reader.index()?;
        
        let collections = reader.list_collections()?;
        
        if collections.len() != index.collection_count() {
            warn!("⚠️  Collection count mismatch!");
            return Ok(false);
        }
        
        // Verify we can read all files
        for collection_name in &collections {
            let collection = reader.get_collection(collection_name)?
                .ok_or_else(|| VectorizerError::Storage("Collection not found in index".to_string()))?;
            
            for file_entry in &collection.files {
                reader.read_file(&file_entry.path)?;
            }
        }
        
        info!("✅ Archive integrity verified");
        Ok(true)
    }
}

/// Get estimated compression ratio for a file
pub fn estimate_compression_ratio(data: &[u8]) -> f64 {
    // Simple heuristic based on data characteristics
    let mut zero_count = 0;
    let mut repeat_count = 0;
    let mut last_byte = 0u8;
    
    for (i, &byte) in data.iter().enumerate() {
        if byte == 0 {
            zero_count += 1;
        }
        if i > 0 && byte == last_byte {
            repeat_count += 1;
        }
        last_byte = byte;
    }
    
    let compressibility = (zero_count + repeat_count) as f64 / data.len() as f64;
    
    // Estimate ratio (higher compressibility = better compression)
    0.3 + (0.5 * (1.0 - compressibility))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};
    use std::io::Write;

    fn create_test_collection(data_dir: &Path) {
        let collections_dir = data_dir.join("collections");
        let collection_dir = collections_dir.join("test_collection");
        fs::create_dir_all(&collection_dir).unwrap();
        
        let test_file = collection_dir.join("test.bin");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"test vector data").unwrap();
    }

    #[test]
    fn test_compactor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let compactor = StorageCompactor::new(temp_dir.path(), 3, 1000);
        
        assert_eq!(compactor.compression_level, 3);
        assert_eq!(compactor.batch_size, 1000);
        assert_eq!(compactor.pending_operations, 0);
    }

    #[test]
    fn test_record_operation() {
        let temp_dir = TempDir::new().unwrap();
        let mut compactor = StorageCompactor::new(temp_dir.path(), 3, 5);
        
        assert!(!compactor.should_compact());
        
        for _ in 0..4 {
            compactor.record_operation();
        }
        assert!(!compactor.should_compact());
        
        compactor.record_operation();
        assert!(compactor.should_compact());
    }

    #[test]
    fn test_compact_all() {
        let temp_dir = TempDir::new().unwrap();
        create_test_collection(temp_dir.path());
        
        let compactor = StorageCompactor::new(temp_dir.path(), 3, 1000);
        let result = compactor.compact_all();
        
        assert!(result.is_ok());
        let index = result.unwrap();
        assert!(index.collections.len() > 0);
    }

    #[test]
    fn test_estimate_compression_ratio() {
        let data = vec![0u8; 1000]; // Highly compressible
        let ratio = estimate_compression_ratio(&data);
        assert!(ratio < 0.5); // Should compress well
        
        let random_data = (0..1000).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        let ratio2 = estimate_compression_ratio(&random_data);
        assert!(ratio2 > ratio); // Less compressible
    }

    #[test]
    fn test_reset_counter() {
        let temp_dir = TempDir::new().unwrap();
        let mut compactor = StorageCompactor::new(temp_dir.path(), 3, 5);
        
        compactor.record_operation();
        compactor.record_operation();
        assert_eq!(compactor.pending_operations(), 2);
        
        compactor.reset_counter();
        assert_eq!(compactor.pending_operations(), 0);
    }
}

