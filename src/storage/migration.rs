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
        
        // Keep legacy files for safety - user can delete manually
        info!("ℹ️  Legacy files kept in: {}", backup_path.display());
        info!("   You can safely delete them after verifying the migration");
        
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
        let backup_dir = self.data_dir.join(format!("backup_before_migration_{}", timestamp));
        
        // Create backup directory
        fs::create_dir_all(&backup_dir)
            .map_err(|e| VectorizerError::Io(e))?;
        
        // Copy legacy data directory
        let legacy_data = self.find_legacy_data_dir()?;
        let backup_data = backup_dir.join("data");
        
        self.copy_dir_recursive(&legacy_data, &backup_data)?;
        
        Ok(backup_dir)
    }
    
    /// Find the legacy data directory
    fn find_legacy_data_dir(&self) -> Result<PathBuf> {
        // Try collections subdirectory first
        let collections_dir = self.data_dir.join("collections");
        if collections_dir.exists() {
            return Ok(collections_dir);
        }
        
        // Try parent data directory
        let parent_data = self.data_dir.parent()
            .unwrap_or(&self.data_dir)
            .join("data");
        
        if parent_data.exists() {
            return Ok(parent_data);
        }
        
        // Use current directory
        Ok(self.data_dir.clone())
    }
    
    /// Count legacy collections
    fn count_legacy_collections(&self) -> Result<usize> {
        let legacy_dir = self.find_legacy_data_dir()?;
        
        let count = fs::read_dir(&legacy_dir)
            .map_err(|e| VectorizerError::Io(e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .count();
        
        Ok(count)
    }
    
    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)
            .map_err(|e| VectorizerError::Io(e))?;
        
        for entry in fs::read_dir(src).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            let file_name = path.file_name()
                .ok_or_else(|| VectorizerError::Storage("Invalid file name".to_string()))?;
            let dest_path = dst.join(file_name);
            
            if path.is_dir() {
                self.copy_dir_recursive(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)
                    .map_err(|e| VectorizerError::Io(e))?;
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
        let collections_dir = data_dir.join("collections");
        let collection_dir = collections_dir.join("test_collection");
        fs::create_dir_all(&collection_dir).unwrap();
        
        let test_file = collection_dir.join("test.bin");
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

