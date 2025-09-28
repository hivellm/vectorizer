//! Content hash validation for file changes

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Sha256, Digest};

/// Hash validator for file content changes
pub struct HashValidator {
    /// Cache of file hashes
    file_hashes: Arc<RwLock<HashMap<PathBuf, String>>>,
    /// Enable hash validation
    enabled: bool,
}

impl HashValidator {
    /// Create a new hash validator
    pub fn new() -> Self {
        Self {
            file_hashes: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    /// Create a new hash validator with enabled flag
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            file_hashes: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }

    /// Calculate hash for a file
    pub async fn calculate_hash(&self, path: &std::path::Path) -> Result<String, std::io::Error> {
        if !self.enabled {
            return Ok("disabled".to_string());
        }

        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash)
    }

    /// Check if file content has changed
    pub async fn has_content_changed(&self, path: &std::path::Path) -> Result<bool, std::io::Error> {
        if !self.enabled {
            return Ok(true); // Always consider changed if validation is disabled
        }

        let current_hash = self.calculate_hash(path).await?;
        
        let mut hashes = self.file_hashes.write().await;
        let previous_hash = hashes.get(path).cloned();
        
        let changed = previous_hash.as_ref() != Some(&current_hash);
        
        if changed {
            hashes.insert(path.to_path_buf(), current_hash);
        }
        
        Ok(changed)
    }

    /// Update hash for a file
    pub async fn update_hash(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        if !self.enabled {
            return Ok(());
        }

        let hash = self.calculate_hash(path).await?;
        let mut hashes = self.file_hashes.write().await;
        hashes.insert(path.to_path_buf(), hash);
        Ok(())
    }

    /// Remove hash for a file
    pub async fn remove_hash(&self, path: &std::path::Path) {
        let mut hashes = self.file_hashes.write().await;
        hashes.remove(path);
    }

    /// Clear all hashes
    pub async fn clear_hashes(&self) {
        let mut hashes = self.file_hashes.write().await;
        hashes.clear();
    }

    /// Get hash for a file
    pub async fn get_hash(&self, path: &std::path::Path) -> Option<String> {
        let hashes = self.file_hashes.read().await;
        hashes.get(path).cloned()
    }

    /// Check if hash validation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable hash validation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get number of cached hashes
    pub async fn cached_hashes_count(&self) -> usize {
        let hashes = self.file_hashes.read().await;
        hashes.len()
    }

    /// Initialize hashes for a directory
    pub async fn initialize_directory_hashes(&self, dir_path: &std::path::Path) -> Result<usize, std::io::Error> {
        if !self.enabled {
            return Ok(0);
        }

        let mut count = 0;
        let mut hashes = self.file_hashes.write().await;

        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Ok(hash) = self.calculate_hash(path).await {
                    hashes.insert(path.to_path_buf(), hash);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Get all cached file paths
    pub async fn get_cached_paths(&self) -> Vec<PathBuf> {
        let hashes = self.file_hashes.read().await;
        hashes.keys().cloned().collect()
    }

    /// Validate hash for a file without updating cache
    pub async fn validate_hash(&self, path: &std::path::Path, expected_hash: &str) -> Result<bool, std::io::Error> {
        if !self.enabled {
            return Ok(true);
        }

        let current_hash = self.calculate_hash(path).await?;
        Ok(current_hash == expected_hash)
    }
}

impl Default for HashValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_hash_validator_creation() {
        let validator = HashValidator::new();
        assert!(validator.is_enabled());
        assert_eq!(validator.cached_hashes_count().await, 0);
    }

    #[tokio::test]
    async fn test_hash_validator_disabled() {
        let validator = HashValidator::with_enabled(false);
        assert!(!validator.is_enabled());
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let hash = validator.calculate_hash(&file_path).await.unwrap();
        assert_eq!(hash, "disabled");
        
        let changed = validator.has_content_changed(&file_path).await.unwrap();
        assert!(changed); // Always true when disabled
    }

    #[tokio::test]
    async fn test_hash_calculation() {
        let validator = HashValidator::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        fs::write(&file_path, "test content").unwrap();
        let hash1 = validator.calculate_hash(&file_path).await.unwrap();
        
        fs::write(&file_path, "different content").unwrap();
        let hash2 = validator.calculate_hash(&file_path).await.unwrap();
        
        assert_ne!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_content_change_detection() {
        let validator = HashValidator::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // First check - should be changed (no previous hash)
        fs::write(&file_path, "test content").unwrap();
        let changed1 = validator.has_content_changed(&file_path).await.unwrap();
        assert!(changed1);
        
        // Second check - should not be changed (same content)
        let changed2 = validator.has_content_changed(&file_path).await.unwrap();
        assert!(!changed2);
        
        // Third check - should be changed (different content)
        fs::write(&file_path, "different content").unwrap();
        let changed3 = validator.has_content_changed(&file_path).await.unwrap();
        assert!(changed3);
    }

    #[tokio::test]
    async fn test_hash_operations() {
        let validator = HashValidator::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        fs::write(&file_path, "test content").unwrap();
        
        // Update hash
        validator.update_hash(&file_path).await.unwrap();
        assert_eq!(validator.cached_hashes_count().await, 1);
        
        // Get hash
        let hash = validator.get_hash(&file_path).await.unwrap();
        assert!(!hash.is_empty());
        
        // Remove hash
        validator.remove_hash(&file_path).await;
        assert_eq!(validator.cached_hashes_count().await, 0);
        
        // Clear all hashes
        validator.update_hash(&file_path).await.unwrap();
        assert_eq!(validator.cached_hashes_count().await, 1);
        validator.clear_hashes().await;
        assert_eq!(validator.cached_hashes_count().await, 0);
    }

    #[tokio::test]
    async fn test_directory_initialization() {
        let validator = HashValidator::new();
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::write(temp_dir.path().join("file3.txt"), "content3").unwrap();
        
        // Initialize directory hashes
        let count = validator.initialize_directory_hashes(temp_dir.path()).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(validator.cached_hashes_count().await, 3);
        
        // Get cached paths
        let paths = validator.get_cached_paths().await;
        assert_eq!(paths.len(), 3);
    }

    #[tokio::test]
    async fn test_hash_validation() {
        let validator = HashValidator::new();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        fs::write(&file_path, "test content").unwrap();
        let correct_hash = validator.calculate_hash(&file_path).await.unwrap();
        
        // Validate with correct hash
        let valid1 = validator.validate_hash(&file_path, &correct_hash).await.unwrap();
        assert!(valid1);
        
        // Validate with incorrect hash
        let valid2 = validator.validate_hash(&file_path, "wrong_hash").await.unwrap();
        assert!(!valid2);
    }
}
