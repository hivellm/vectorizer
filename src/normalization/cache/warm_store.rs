//! Warm store implementation using memory-mapped files
//!
//! This is the second tier of the cache hierarchy, providing persistent
//! storage for frequently accessed normalized text using mmap for fast access.

use std::collections::HashMap;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use memmap2::{Mmap, MmapMut};
use parking_lot::RwLock;

use crate::normalization::ContentHash;

/// Warm store for medium-term caching with mmap
pub struct WarmStore {
    /// Base directory for store files
    base_path: PathBuf,
    /// Index: hash → (file_path, offset, length)
    index: Arc<RwLock<HashMap<ContentHash, (PathBuf, u64, u64)>>>,
    /// Memory-mapped files cache
    mmap_cache: Arc<RwLock<HashMap<PathBuf, Arc<Mmap>>>>,
}

impl WarmStore {
    /// Create a new warm store
    pub fn new(base_path: &Path) -> Result<Self> {
        create_dir_all(base_path)
            .with_context(|| format!("Failed to create warm store directory: {:?}", base_path))?;

        let mut store = Self {
            base_path: base_path.to_path_buf(),
            index: Arc::new(RwLock::new(HashMap::new())),
            mmap_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing index
        store.load_index()?;

        Ok(store)
    }

    /// Get text from warm store
    pub async fn get(&self, hash: &ContentHash) -> Result<Option<String>> {
        let index = self.index.read();

        if let Some((file_path, offset, length)) = index.get(hash) {
            // Clone values before dropping the lock
            let file_path = file_path.clone();
            let offset = *offset;
            let length = *length;
            drop(index);

            // Get or create mmap
            let mmap = self.get_or_create_mmap(&file_path)?;

            // Read from mmap
            let start = offset as usize;
            let end = start + length as usize;

            if end <= mmap.len() {
                let data = &mmap[start..end];
                let text = String::from_utf8(data.to_vec())?;
                return Ok(Some(text));
            }
        }

        Ok(None)
    }

    /// Put text into warm store
    pub async fn put(&self, hash: ContentHash, text: &str) -> Result<()> {
        let data = text.as_bytes();
        let length = data.len() as u64;

        // Determine shard file (based on hash prefix)
        let shard = hash.as_bytes()[0] % 16;
        let file_name = format!("shard_{:02x}.bin", shard);
        let file_path = self.base_path.join(&file_name);

        // Open file for append
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&file_path)?;

        // Get current offset (end of file)
        let offset = file.seek(SeekFrom::End(0))?;

        // Write data
        file.write_all(data)?;
        file.sync_all()?;

        // Update index
        let mut index = self.index.write();
        index.insert(hash, (file_path.clone(), offset, length));

        // Invalidate mmap cache for this file
        self.mmap_cache.write().remove(&file_path);

        // Persist index
        drop(index);
        self.save_index()?;

        Ok(())
    }

    /// Clear all entries
    pub async fn clear(&self) -> Result<()> {
        let mut index = self.index.write();
        index.clear();

        let mut mmap_cache = self.mmap_cache.write();
        mmap_cache.clear();

        // Remove all shard files
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "bin") {
                std::fs::remove_file(path)?;
            }
        }

        self.save_index()?;
        Ok(())
    }

    /// Get or create memory-mapped file
    fn get_or_create_mmap(&self, file_path: &Path) -> Result<Arc<Mmap>> {
        {
            let cache = self.mmap_cache.read();
            if let Some(mmap) = cache.get(file_path) {
                return Ok(Arc::clone(mmap));
            }
        }

        // Create new mmap
        let file = File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let mmap = Arc::new(mmap);

        // Cache it
        let mut cache = self.mmap_cache.write();
        cache.insert(file_path.to_path_buf(), Arc::clone(&mmap));

        Ok(mmap)
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

        let index: HashMap<ContentHash, (PathBuf, u64, u64)> =
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
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_warm_store_basic() {
        let dir = tempdir().unwrap();
        let store = WarmStore::new(dir.path()).unwrap();

        let hash = ContentHash::from_bytes([1u8; 32]);
        let text = "Test text for warm store";

        // Put
        store.put(hash, text).await.unwrap();

        // Get
        let retrieved = store.get(&hash).await.unwrap();
        assert_eq!(retrieved.as_deref(), Some(text));
    }

    #[tokio::test]
    async fn test_warm_store_miss() {
        let dir = tempdir().unwrap();
        let store = WarmStore::new(dir.path()).unwrap();

        let hash = ContentHash::from_bytes([99u8; 32]);
        let result = store.get(&hash).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_warm_store_multiple() {
        let dir = tempdir().unwrap();
        let store = WarmStore::new(dir.path()).unwrap();

        // Store multiple entries
        for i in 0..10u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);
            let text = format!("Text number {}", i);

            store.put(hash, &text).await.unwrap();
        }

        // Retrieve all
        for i in 0..10u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);

            let retrieved = store.get(&hash).await.unwrap();
            let expected = format!("Text number {}", i);
            assert_eq!(retrieved.as_deref(), Some(expected.as_str()));
        }
    }

    #[tokio::test]
    async fn test_warm_store_persistence() {
        let dir = tempdir().unwrap();

        let hash = ContentHash::from_bytes([2u8; 32]);
        let text = "Persistent text";

        // Create store, put data, drop it
        {
            let store = WarmStore::new(dir.path()).unwrap();
            store.put(hash, text).await.unwrap();
        }

        // Create new store, should load persisted data
        {
            let store = WarmStore::new(dir.path()).unwrap();
            let retrieved = store.get(&hash).await.unwrap();
            assert_eq!(retrieved.as_deref(), Some(text));
        }
    }

}
