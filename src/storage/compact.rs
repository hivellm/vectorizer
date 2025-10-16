//! Storage compaction logic

use crate::error::{Result, VectorizerError};
use crate::storage::{StorageWriter, StorageReader, StorageIndex};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

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
        // Use data directory directly (flat structure with collection-name_*.bin files)
        if !self.data_dir.exists() {
            return Err(VectorizerError::Storage(
                format!("Data directory not found: {}", self.data_dir.display())
            ));
        }
        
        self.compact_directory(&self.data_dir)
    }
    
    /// Compact all collections from memory (no raw files created/used)
    pub fn compact_from_memory(&self, store: &crate::db::VectorStore) -> Result<StorageIndex> {
        info!("🗜️  Starting compaction from memory (no raw files)");
        
        // SAFETY: Create backup of existing .vecdb before overwriting
        let vecdb_path = self.data_dir.join(crate::storage::VECDB_FILE);
        if vecdb_path.exists() {
            let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
            match std::fs::copy(&vecdb_path, &backup_path) {
                Ok(_) => {
                    info!("💾 Created backup: {}", backup_path.display());
                }
                Err(e) => {
                    warn!("⚠️  Failed to create backup: {} - proceeding with caution", e);
                }
            }
        }
        
        // Get all collections from store
        let collection_names = store.list_collections();
        
        if collection_names.is_empty() {
            error!("❌ No collections in memory to compact");
            return Err(VectorizerError::Storage("No collections to compact".to_string()));
        }
        
        info!("📦 Found {} collections in memory", collection_names.len());
        
        let mut persisted_collections = Vec::new();
        
        for name in &collection_names {
            match store.get_collection(name) {
                Ok(collection_ref) => {
                    // Get all vectors from collection
                    use crate::db::CollectionType;
                    let vectors = match collection_ref.deref() {
                        CollectionType::Cpu(c) => c.get_all_vectors(),
                        #[cfg(feature = "wgpu-gpu")]
                        _ => {
                            warn!("⚠️  GPU collections not yet supported for memory compaction, skipping '{}'", name);
                            continue;
                        }
                    };
                    
                    info!("   Collection '{}': {} vectors", name, vectors.len());
                    
                    // Convert to persisted format
                    let persisted_vectors: Vec<crate::persistence::PersistedVector> = vectors
                        .into_iter()
                        .map(|v| crate::persistence::PersistedVector::from(v))
                        .collect();
                    
                    let config = match collection_ref.deref() {
                        CollectionType::Cpu(c) => c.config().clone(),
                        #[cfg(feature = "wgpu-gpu")]
                        _ => {
                            warn!("⚠️  GPU collections not yet supported for memory compaction, skipping '{}'", name);
                            continue;
                        }
                    };
                    
                    let persisted = crate::persistence::PersistedCollection {
                        name: name.clone(),
                        config: Some(config),
                        vectors: persisted_vectors,
                        hnsw_dump_basename: None,
                    };
                    
                    persisted_collections.push(persisted);
                }
                Err(e) => {
                    warn!("⚠️  Failed to get collection '{}': {}", name, e);
                    continue;
                }
            }
        }
        
        if persisted_collections.is_empty() {
            error!("❌ CRITICAL: No collections could be serialized from memory!");
            error!("   Refusing to create empty archive");
            
            // Restore from backup if it exists
            let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
            if backup_path.exists() {
                warn!("🔄 Restoring from backup...");
                if let Ok(_) = std::fs::copy(&backup_path, &vecdb_path) {
                    info!("✅ Restored from backup - data preserved");
                }
            }
            
            return Err(VectorizerError::Storage(
                "No collections could be serialized - data loss prevented".to_string()
            ));
        }
        
        // CRITICAL SAFETY CHECK: Calculate total vectors BEFORE overwriting .vecdb
        let total_vectors: usize = persisted_collections.iter()
            .map(|c| c.vectors.len())
            .sum();
        
        if total_vectors == 0 {
            error!("❌ CRITICAL: All {} collections have ZERO vectors!", persisted_collections.len());
            error!("   Refusing to overwrite vectorizer.vecdb with empty collections!");
            
            // Check if .vecdb exists and has data
            if vecdb_path.exists() {
                if let Ok(metadata) = std::fs::metadata(&vecdb_path) {
                    let size_mb = metadata.len() / 1_048_576;
                    error!("   Existing vectorizer.vecdb size: {} MB", size_mb);
                    if size_mb > 0 {
                        error!("   REFUSING to overwrite {} MB of existing data with empty collections!", size_mb);
                    }
                }
            }
            
            // Restore from backup if it exists
            let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
            if backup_path.exists() {
                if let Ok(_) = std::fs::remove_file(&backup_path) {
                    info!("🗑️  Removed backup since no changes will be made");
                }
            }
            
            return Err(VectorizerError::Storage(
                format!("Refusing to overwrite vectorizer.vecdb - {} collections but 0 total vectors", persisted_collections.len())
            ));
        }
        
        info!("✅ Safety check passed: {} total vectors across {} collections", total_vectors, persisted_collections.len());
        
        // Write from memory (no disk files)
        let writer = StorageWriter::new(&self.data_dir, self.compression_level);
        let index = writer.write_from_memory(persisted_collections)?;
        
        info!("✅ Compaction from memory complete:");
        info!("   Collections: {}", index.collection_count());
        info!("   Total vectors: {}", index.total_vectors());
        info!("   Original size: {} MB", index.total_size / 1_048_576);
        info!("   Compressed size: {} MB", index.compressed_size / 1_048_576);
        info!("   Compression ratio: {:.2}%", index.compression_ratio * 100.0);
        
        // Success! Remove backup since new .vecdb is verified
        let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
        if backup_path.exists() {
            if let Ok(_) = std::fs::remove_file(&backup_path) {
                debug!("🗑️  Removed backup file after successful compaction");
            }
        }
        
        Ok(index)
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
    
    /// Compact all collections and optionally remove raw files
    pub fn compact_all_with_cleanup(&self, remove_source_files: bool) -> Result<StorageIndex> {
        info!("🗜️  Starting compaction (remove_source_files: {})", remove_source_files);
        
        // SAFETY: Create backup of existing .vecdb before overwriting
        let vecdb_path = self.data_dir.join(crate::storage::VECDB_FILE);
        if vecdb_path.exists() {
            let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
            match std::fs::copy(&vecdb_path, &backup_path) {
                Ok(_) => {
                    info!("💾 Created backup: {}", backup_path.display());
                }
                Err(e) => {
                    warn!("⚠️  Failed to create backup: {} - proceeding with caution", e);
                }
            }
        }
        
        // First, compact everything
        let index = self.compact_all()?;
        
        // CRITICAL SAFETY CHECK: Verify we're not creating an empty archive
        if index.collection_count() == 0 {
            error!("❌ CRITICAL: Compaction resulted in ZERO collections!");
            error!("   Refusing to overwrite existing .vecdb with empty archive");
            error!("   This indicates a serious problem with the compaction process");
            
            // Restore from backup if it exists
            let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
            if backup_path.exists() {
                warn!("🔄 Restoring from backup...");
                if let Ok(_) = std::fs::copy(&backup_path, &vecdb_path) {
                    info!("✅ Restored from backup - data preserved");
                }
            }
            
            return Err(VectorizerError::Storage(
                "Compaction resulted in empty archive - data loss prevented".to_string()
            ));
        }
        
        // Verify integrity before removing anything
        if !self.verify_integrity()? {
            error!("❌ Integrity check failed - restoring from backup");
            let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
            if backup_path.exists() {
                if let Ok(_) = std::fs::copy(&backup_path, &vecdb_path) {
                    info!("✅ Restored from backup after integrity failure");
                }
            }
            return Err(VectorizerError::Storage("Integrity check failed after compaction".to_string()));
        }
        
        // If requested and verification passed, remove raw files
        if remove_source_files {
            info!("🗑️  Removing raw collection files...");
            let removed_count = self.remove_raw_files()?;
            info!("✅ Removed {} raw files", removed_count);
        }
        
        // Success! Remove backup since new .vecdb is verified
        let backup_path = self.data_dir.join(format!("{}.backup", crate::storage::VECDB_FILE));
        if backup_path.exists() {
            if let Ok(_) = std::fs::remove_file(&backup_path) {
                debug!("🗑️  Removed backup file after successful compaction");
            }
        }
        
        Ok(index)
    }
    
    /// Remove raw collection files (*.bin, *.json, *.checksum) from data directory
    pub fn remove_raw_files(&self) -> Result<usize> {
        use std::fs;
        
        let mut removed_count = 0;
        
        for entry in fs::read_dir(&self.data_dir).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip .vecdb and .vecidx files
                    if name == crate::storage::VECDB_FILE || name == crate::storage::VECIDX_FILE {
                        continue;
                    }
                    
                    // Remove legacy collection files
                    if name.ends_with("_vector_store.bin") 
                        || name.ends_with("_tokenizer.json")
                        || name.ends_with("_metadata.json")
                        || name.ends_with("_checksums.json") {
                        match fs::remove_file(&path) {
                            Ok(_) => {
                                info!("   Removed: {}", name);
                                removed_count += 1;
                            }
                            Err(e) => {
                                warn!("   Failed to remove {}: {}", name, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(removed_count)
    }
    
    /// Compact only if changes detected (checks timestamps and sizes)
    pub fn compact_if_changed(&mut self) -> Result<Option<StorageIndex>> {
        use std::fs;
        use std::time::SystemTime;
        
        let vecdb_path = self.data_dir.join(crate::storage::VECDB_FILE);
        
        // If no .vecdb exists, always compact
        if !vecdb_path.exists() {
            info!("💾 No existing archive found, performing full compaction");
            return self.compact_all().map(Some);
        }
        
        // Get .vecdb modification time
        let vecdb_modified = fs::metadata(&vecdb_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        
        // Check if any raw files are newer than .vecdb
        let mut changes_detected = false;
        
        for entry in fs::read_dir(&self.data_dir).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check if it's a collection file
                    if name.ends_with("_vector_store.bin") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if modified > vecdb_modified {
                                    info!("📝 Detected change in: {}", name);
                                    changes_detected = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if changes_detected {
            info!("💾 Changes detected, starting compaction...");
            let index = self.compact_all()?;
            self.reset_counter();
            Ok(Some(index))
        } else {
            info!("⏭️  No changes detected, skipping compaction");
            Ok(None)
        }
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
        fs::create_dir_all(&data_dir).unwrap();
        
        // Create test collection file directly in data_dir (flat structure)
        let test_file = data_dir.join("test_collection_vector_store.bin");
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

