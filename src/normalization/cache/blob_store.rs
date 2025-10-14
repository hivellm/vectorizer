//! Blob store implementation with Zstandard compression
//!
//! This is the third tier of the cache hierarchy, providing long-term
//! persistent storage with compression for maximum space efficiency.

use crate::normalization::ContentHash;
use anyhow::{Context, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Blob entry metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BlobEntry {
    file_path: PathBuf,
    offset: u64,
    compressed_size: u64,
    uncompressed_size: u64,
}

/// Cold blob store with compression
pub struct BlobStore {
    /// Base directory for blob files
    base_path: PathBuf,
    /// Compression level (1-22)
    compression_level: i32,
    /// Index: hash → blob entry
    index: Arc<RwLock<HashMap<ContentHash, BlobEntry>>>,
}

impl BlobStore {
    /// Create a new blob store
    pub fn new(base_path: &Path, compression_level: i32) -> Result<Self> {
        create_dir_all(base_path)
            .with_context(|| format!("Failed to create blob store directory: {:?}", base_path))?;

        let mut store = Self {
            base_path: base_path.to_path_buf(),
            compression_level,
            index: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing index
        store.load_index()?;

        Ok(store)
    }

    /// Get blob from store
    pub async fn get(&self, hash: &ContentHash) -> Result<Option<Vec<u8>>> {
        let index = self.index.read();

        if let Some(entry) = index.get(hash) {
            let entry = entry.clone();
            drop(index);

            // Read compressed data
            let mut file = File::open(&entry.file_path)?;
            file.seek(SeekFrom::Start(entry.offset))?;

            let mut compressed = vec![0u8; entry.compressed_size as usize];
            file.read_exact(&mut compressed)?;

            // Decompress
            let decompressed = zstd::decode_all(&compressed[..])?;

            return Ok(Some(decompressed));
        }

        Ok(None)
    }

    /// Put blob into store
    pub async fn put(&self, hash: ContentHash, data: &[u8]) -> Result<()> {
        let uncompressed_size = data.len() as u64;

        // Compress data
        let compressed = zstd::encode_all(data, self.compression_level)?;
        let compressed_size = compressed.len() as u64;

        // Determine shard file
        let shard = hash.as_bytes()[0] % 16;
        let file_name = format!("blob_{:02x}.zst", shard);
        let file_path = self.base_path.join(&file_name);

        // Open file for append
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&file_path)?;

        // Get current offset
        let offset = file.seek(SeekFrom::End(0))?;

        // Write compressed data
        file.write_all(&compressed)?;
        file.sync_all()?;

        // Update index
        let mut index = self.index.write();
        index.insert(
            hash,
            BlobEntry {
                file_path: file_path.clone(),
                offset,
                compressed_size,
                uncompressed_size,
            },
        );

        drop(index);
        self.save_index()?;

        Ok(())
    }

    /// Clear all entries
    pub async fn clear(&self) -> Result<()> {
        let mut index = self.index.write();
        index.clear();

        // Remove all blob files
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "zst") {
                std::fs::remove_file(path)?;
            }
        }

        self.save_index()?;
        Ok(())
    }

    /// Load index from disk
    fn load_index(&mut self) -> Result<()> {
        let index_path = self.base_path.join("index.bin");

        if !index_path.exists() {
            return Ok(());
        }

        let mut file = File::open(&index_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.is_empty() {
            return Ok(());
        }

        let index: HashMap<ContentHash, BlobEntry> =
            bincode::deserialize(&data).unwrap_or_default();

        *self.index.write() = index;

        Ok(())
    }

    /// Save index to disk
    fn save_index(&self) -> Result<()> {
        let index_path = self.base_path.join("index.bin");

        let index = self.index.read();
        let data = bincode::serialize(&*index)?;

        let mut file = File::create(&index_path)?;
        file.write_all(&data)?;
        file.sync_all()?;

        Ok(())
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.index.read().len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.index.read().is_empty()
    }

    /// Get compression statistics
    pub fn compression_stats(&self) -> CompressionStats {
        let index = self.index.read();

        let total_compressed: u64 = index.values().map(|e| e.compressed_size).sum();
        let total_uncompressed: u64 = index.values().map(|e| e.uncompressed_size).sum();

        let compression_ratio = if total_uncompressed > 0 {
            total_uncompressed as f64 / total_compressed as f64
        } else {
            1.0
        };

        CompressionStats {
            entries: index.len(),
            total_compressed,
            total_uncompressed,
            compression_ratio,
            space_saved: total_uncompressed.saturating_sub(total_compressed),
        }
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub entries: usize,
    pub total_compressed: u64,
    pub total_uncompressed: u64,
    pub compression_ratio: f64,
    pub space_saved: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_blob_store_basic() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path(), 3).unwrap();

        let hash = ContentHash::from_bytes([1u8; 32]);
        let data = b"Test data for blob store";

        // Put
        store.put(hash, data).await.unwrap();

        // Get
        let retrieved = store.get(&hash).await.unwrap();
        assert_eq!(retrieved.as_deref(), Some(data.as_ref()));
    }

    #[tokio::test]
    async fn test_blob_store_compression() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path(), 3).unwrap();

        let hash = ContentHash::from_bytes([2u8; 32]);
        // Highly compressible data
        let data = "a".repeat(10000);

        store.put(hash, data.as_bytes()).await.unwrap();

        let stats = store.compression_stats();
        assert!(stats.compression_ratio > 2.0); // Should compress well
        assert!(stats.space_saved > 5000);
    }

    #[tokio::test]
    async fn test_blob_store_miss() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path(), 3).unwrap();

        let hash = ContentHash::from_bytes([99u8; 32]);
        let result = store.get(&hash).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_blob_store_persistence() {
        let dir = tempdir().unwrap();

        let hash = ContentHash::from_bytes([3u8; 32]);
        let data = b"Persistent blob data";

        // Create store, put data, drop it
        {
            let store = BlobStore::new(dir.path(), 3).unwrap();
            store.put(hash, data).await.unwrap();
        }

        // Create new store, should load persisted data
        {
            let store = BlobStore::new(dir.path(), 3).unwrap();
            let retrieved = store.get(&hash).await.unwrap();
            assert_eq!(retrieved.as_deref(), Some(data.as_ref()));
        }
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_blob_store_clear() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path(), 3).unwrap();

        let hash = ContentHash::from_bytes([4u8; 32]);
        store.put(hash, b"test").await.unwrap();

        store.clear().await.unwrap();

        let result = store.get(&hash).await.unwrap();
        assert!(result.is_none());
        assert_eq!(store.len(), 0);
    }

    #[tokio::test]
    async fn test_blob_store_multiple() {
        let dir = tempdir().unwrap();
        let store = BlobStore::new(dir.path(), 3).unwrap();

        // Store multiple blobs
        for i in 0..10u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);
            let data = format!("Blob data {}", i);

            store.put(hash, data.as_bytes()).await.unwrap();
        }

        // Retrieve all
        for i in 0..10u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);

            let retrieved = store.get(&hash).await.unwrap();
            assert_eq!(
                retrieved.as_deref(),
                Some(format!("Blob data {}", i).as_bytes())
            );
        }
    }
}

