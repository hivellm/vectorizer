//! File hashing utilities for change detection

use std::fs;
use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

/// Calculate SHA256 hash of a file's content
pub fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Get file modification time
pub fn get_file_modified_time(file_path: &Path) -> Result<chrono::DateTime<chrono::Utc>> {
    let metadata = fs::metadata(file_path)?;
    let system_time = metadata.modified()?;
    Ok(system_time.into())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_calculate_file_hash_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();

        let hash = calculate_file_hash(temp_file.path());
        assert!(hash.is_ok());

        // SHA256 of empty file is known
        let empty_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash.unwrap(), empty_hash);
    }

    #[test]
    fn test_calculate_file_hash_with_content() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();
        temp_file.flush().unwrap();

        let hash = calculate_file_hash(temp_file.path());
        assert!(hash.is_ok());

        let hash_str = hash.unwrap();
        assert_eq!(hash_str.len(), 64); // SHA256 hash is 64 hex chars
    }

    #[test]
    fn test_calculate_file_hash_consistency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "same content").unwrap();
        temp_file.flush().unwrap();

        let hash1 = calculate_file_hash(temp_file.path()).unwrap();
        let hash2 = calculate_file_hash(temp_file.path()).unwrap();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash_different_content() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        write!(temp_file1, "content 1").unwrap();
        temp_file1.flush().unwrap();

        let mut temp_file2 = NamedTempFile::new().unwrap();
        write!(temp_file2, "content 2").unwrap();
        temp_file2.flush().unwrap();

        let hash1 = calculate_file_hash(temp_file1.path()).unwrap();
        let hash2 = calculate_file_hash(temp_file2.path()).unwrap();

        // Different content should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash_nonexistent() {
        let nonexistent = Path::new("/nonexistent/file.txt");
        let result = calculate_file_hash(nonexistent);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_modified_time() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test").unwrap();
        temp_file.flush().unwrap();

        let modified_time = get_file_modified_time(temp_file.path());
        assert!(modified_time.is_ok());

        let time = modified_time.unwrap();
        // Should be a recent timestamp
        let now = chrono::Utc::now();
        assert!(time <= now);
    }

    #[test]
    fn test_get_file_modified_time_nonexistent() {
        let nonexistent = Path::new("/nonexistent/file.txt");
        let result = get_file_modified_time(nonexistent);

        assert!(result.is_err());
    }

    #[test]
    fn test_file_hash_binary_content() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let binary_data = vec![0u8, 1, 2, 3, 255, 128, 64];
        temp_file.write_all(&binary_data).unwrap();
        temp_file.flush().unwrap();

        let hash = calculate_file_hash(temp_file.path());
        assert!(hash.is_ok());
        assert_eq!(hash.unwrap().len(), 64);
    }
}
