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
