//! Storage migration from legacy format to .vecdb

use crate::error::{Result, VectorizerError};
use crate::storage::{StorageCompactor, StorageFormat, detect_format};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};
use chrono::Utc;

/// Storage migrator for converting legacy format to .vecdb
pub struct StorageMigrator {
    /// Data directory
    data_dir: PathBuf,
    
    /// Compression level for migration
    compression_level: i32,
}

impl StorageMigrator {
    /// Create a new storage migrator
    pub fn new(data_dir: impl AsRef<Path>, compression_level: i32) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            compression_level,
        }
    }
    
    /// Check if migration is needed
    pub fn needs_migration(&self) -> bool {
        detect_format(&self.data_dir) == StorageFormat::Legacy
    }
    
    /// Perform automatic migration
    pub fn migrate(&self) -> Result<MigrationResult> {
        info!("🔄 Starting storage migration to .vecdb format...");
        
        // Check current format
        if !self.needs_migration() {
            info!("✅ Already using .vecdb format, no migration needed");
            return Ok(MigrationResult {
                success: true,
                collections_migrated: 0,
                backup_path: None,
                message: "Already using .vecdb format".to_string(),
            });
        }
        
        // Create backup
        let backup_path = self.create_backup()?;
        info!("📦 Backup created: {}", backup_path.display());
        
        // Count collections before migration
        let collections_count = self.count_legacy_collections()?;
        
        // Perform compaction (migration)
        let compactor = StorageCompactor::new(&self.data_dir, self.compression_level, 1000);
        let index = compactor.compact_all()?;
        
        // Verify migration
        if !compactor.verify_integrity()? {
            error!("❌ Migration verification failed!");
            return Err(VectorizerError::Storage("Migration verification failed".to_string()));
        }
        
        info!("✅ Migration completed successfully!");
        info!("   Migrated {} collections", index.collection_count());
        info!("   Total vectors: {}", index.total_vectors());
        info!("   Space saved: {} MB", 
            (index.total_size.saturating_sub(index.compressed_size)) / 1_048_576);
        
        // Remove legacy files after successful migration
        info!("🗑️  Removing legacy files from data directory...");
        let removed_count = self.remove_legacy_files()?;
        info!("✅ Removed {} legacy files", removed_count);
        
        info!("ℹ️  Backup saved to: {}", backup_path.display());
        info!("   You can delete the backup after verifying the migration");
        
        Ok(MigrationResult {
            success: true,
            collections_migrated: index.collection_count(),
            backup_path: Some(backup_path),
            message: format!("Successfully migrated {} collections", index.collection_count()),
        })
    }
    
    /// Create backup of current data
    fn create_backup(&self) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        // Use shorter path to avoid "File name too long" errors
        // Place backup inside data directory
        let backup_dir = self.data_dir.join(format!(".bak.{}", timestamp));
        
        // Create backup directory
        fs::create_dir_all(&backup_dir)
            .map_err(|e| VectorizerError::Io(e))?;
        
        // Copy legacy data directory
        let legacy_data = self.find_legacy_data_dir()?;
        
        self.copy_dir_recursive(&legacy_data, &backup_dir)?;
        
        Ok(backup_dir)
    }
    
    /// Find the legacy data directory
    fn find_legacy_data_dir(&self) -> Result<PathBuf> {
        // Current structure: files directly in data/ directory
        // Pattern: collection-name_vector_store.bin
        Ok(self.data_dir.clone())
    }
    
    /// Count legacy collections
    fn count_legacy_collections(&self) -> Result<usize> {
        let legacy_dir = self.find_legacy_data_dir()?;
        
        // Count unique collection names (group by prefix before last _)
        use std::collections::HashSet;
        let mut collection_names = HashSet::new();
        
        for entry in fs::read_dir(&legacy_dir).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with("_vector_store.bin") {
                        if let Some(pos) = name.rfind('_') {
                            collection_names.insert(name[..pos].to_string());
                        }
                    }
                }
            }
        }
        
        Ok(collection_names.len())
    }
    
    /// Remove legacy files after successful migration
    fn remove_legacy_files(&self) -> Result<usize> {
        let mut removed_count = 0;
        
        for entry in fs::read_dir(&self.data_dir).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Remove files with legacy patterns
                    if name.ends_with("_vector_store.bin") 
                        || name.ends_with("_tokenizer.json")
                        || name.ends_with("_metadata.json") {
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
    
    /// Copy directory recursively (with error tolerance for long file names)
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let entries = match fs::read_dir(src) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("⚠️ Failed to read directory {:?}: {}", src, e);
                return Ok(()); // Continue with migration even if we can't backup this directory
            }
        };
        
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("⚠️ Failed to read directory entry: {}", e);
                    continue;
                }
            };
            
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => name,
                None => {
                    warn!("⚠️ Invalid file name for path: {:?}", path);
                    continue;
                }
            };
            
            // Skip backup directories and .vecdb files
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with(".bak") 
                    || name_str == "vectorizer.vecdb" 
                    || name_str == "vectorizer.vecidx" {
                    continue;
                }
            }
            
            let dest_path = dst.join(file_name);
            
            if path.is_dir() {
                // Try to copy directory, but don't fail if it can't be copied
                if let Err(e) = self.copy_dir_recursive(&path, &dest_path) {
                    warn!("⚠️ Failed to backup directory {:?}: {}", path, e);
                }
            } else {
                // Try to copy file, but don't fail if it can't be copied
                if let Err(e) = fs::copy(&path, &dest_path) {
                    warn!("⚠️ Failed to backup file {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Rollback migration (restore from backup)
    pub fn rollback(&self, backup_path: &Path) -> Result<()> {
        warn!("🔙 Rolling back migration...");
        
        if !backup_path.exists() {
            return Err(VectorizerError::Storage(
                format!("Backup not found: {}", backup_path.display())
            ));
        }
        
        // Remove .vecdb and .vecidx files
        let vecdb_path = self.data_dir.join(crate::storage::VECDB_FILE);
        let vecidx_path = self.data_dir.join(crate::storage::VECIDX_FILE);
        
        if vecdb_path.exists() {
            fs::remove_file(&vecdb_path)
                .map_err(|e| VectorizerError::Io(e))?;
        }
        
        if vecidx_path.exists() {
            fs::remove_file(&vecidx_path)
                .map_err(|e| VectorizerError::Io(e))?;
        }
        
        // Restore from backup
        let backup_data = backup_path.join("data");
        if backup_data.exists() {
            self.copy_dir_recursive(&backup_data, &self.data_dir)?;
        }
        
        info!("✅ Rollback completed");
        Ok(())
    }
}

/// Result of a migration operation
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Whether migration was successful
    pub success: bool,
    
    /// Number of collections migrated
    pub collections_migrated: usize,
    
    /// Path to backup directory
    pub backup_path: Option<PathBuf>,
    
    /// Result message
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_legacy_structure(data_dir: &Path) {
        fs::create_dir_all(&data_dir).unwrap();
        
        // Create legacy format: flat files directly in data_dir with collection_name_vector_store.bin pattern
        let test_file = data_dir.join("test_collection_vector_store.bin");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"test vector data").unwrap();
    }

    #[test]
    fn test_migrator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let migrator = StorageMigrator::new(temp_dir.path(), 3);
        
        assert_eq!(migrator.compression_level, 3);
    }

    #[test]
    fn test_needs_migration() {
        let temp_dir = TempDir::new().unwrap();
        create_legacy_structure(temp_dir.path());
        
        let migrator = StorageMigrator::new(temp_dir.path(), 3);
        assert!(migrator.needs_migration());
    }

    #[test]
    fn test_count_legacy_collections() {
        let temp_dir = TempDir::new().unwrap();
        create_legacy_structure(temp_dir.path());
        
        let migrator = StorageMigrator::new(temp_dir.path(), 3);
        let count = migrator.count_legacy_collections().unwrap();
        
        assert!(count > 0);
    }

    #[test]
    fn test_migration() {
        let temp_dir = TempDir::new().unwrap();
        create_legacy_structure(temp_dir.path());
        
        let migrator = StorageMigrator::new(temp_dir.path(), 3);
        let result = migrator.migrate();
        
        assert!(result.is_ok());
        let migration_result = result.unwrap();
        assert!(migration_result.success);
        assert!(migration_result.collections_migrated > 0);
    }
}

