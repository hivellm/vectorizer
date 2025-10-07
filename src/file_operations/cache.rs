use super::types::{CachedFile, FileSummary};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// File-level cache system
pub struct FileLevelCache {
    /// LRU cache for complete files (max 100 files)
    file_content_cache: Arc<RwLock<LruCache<String, CachedFile>>>,
    
    /// LRU cache for summaries (max 500 summaries)
    summary_cache: Arc<RwLock<LruCache<String, CachedSummary>>>,
    
    /// File list cache per collection (with TTL)
    file_list_cache: Arc<RwLock<lru::LruCache<String, CachedFileList>>>,
}

#[derive(Debug, Clone)]
struct CachedSummary {
    summary: FileSummary,
    cached_at: Instant,
}

#[derive(Debug, Clone)]
struct CachedFileList {
    files: Vec<super::types::FileInfo>,
    total_chunks: usize,
    cached_at: Instant,
}

impl FileLevelCache {
    pub fn new() -> Self {
        Self {
            file_content_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(100).unwrap())
            )),
            summary_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(500).unwrap())
            )),
            file_list_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(50).unwrap())
            )),
        }
    }

    /// Get cached file content
    pub async fn get_file_content(&self, key: &str) -> Option<CachedFile> {
        let mut cache = self.file_content_cache.write().await;
        cache.get(key).cloned()
    }

    /// Put file content in cache
    pub async fn put_file_content(&self, key: String, file: CachedFile) {
        let mut cache = self.file_content_cache.write().await;
        cache.put(key, file);
    }

    /// Get cached summary
    pub async fn get_summary(&self, key: &str, max_age: Duration) -> Option<FileSummary> {
        let mut cache = self.summary_cache.write().await;
        if let Some(cached) = cache.get(key) {
            if cached.cached_at.elapsed() < max_age {
                return Some(cached.summary.clone());
            }
        }
        None
    }

    /// Put summary in cache
    pub async fn put_summary(&self, key: String, summary: FileSummary) {
        let mut cache = self.summary_cache.write().await;
        cache.put(key, CachedSummary {
            summary,
            cached_at: Instant::now(),
        });
    }

    /// Get cached file list
    pub async fn get_file_list(
        &self, 
        collection: &str, 
        max_age: Duration
    ) -> Option<(Vec<super::types::FileInfo>, usize)> {
        let mut cache = self.file_list_cache.write().await;
        if let Some(cached) = cache.get(collection) {
            if cached.cached_at.elapsed() < max_age {
                return Some((cached.files.clone(), cached.total_chunks));
            }
        }
        None
    }

    /// Put file list in cache
    pub async fn put_file_list(
        &self,
        collection: String,
        files: Vec<super::types::FileInfo>,
        total_chunks: usize,
    ) {
        let mut cache = self.file_list_cache.write().await;
        cache.put(collection, CachedFileList {
            files,
            total_chunks,
            cached_at: Instant::now(),
        });
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.file_content_cache.write().await.clear();
        self.summary_cache.write().await.clear();
        self.file_list_cache.write().await.clear();
    }

    /// Clear cache for specific collection
    pub async fn clear_collection(&self, collection: &str) {
        // Clear file content cache entries for this collection
        {
            let mut cache = self.file_content_cache.write().await;
            let keys_to_remove: Vec<String> = cache
                .iter()
                .filter(|(k, _)| k.starts_with(&format!("{}:", collection)))
                .map(|(k, _)| k.clone())
                .collect();
            
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // Clear summary cache entries for this collection
        {
            let mut cache = self.summary_cache.write().await;
            let keys_to_remove: Vec<String> = cache
                .iter()
                .filter(|(k, _)| k.starts_with(&format!("{}:", collection)))
                .map(|(k, _)| k.clone())
                .collect();
            
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // Clear file list cache for this collection
        self.file_list_cache.write().await.pop(collection);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            file_content_entries: self.file_content_cache.read().await.len(),
            summary_entries: self.summary_cache.read().await.len(),
            file_list_entries: self.file_list_cache.read().await.len(),
        }
    }
}

impl Default for FileLevelCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub file_content_entries: usize,
    pub summary_entries: usize,
    pub file_list_entries: usize,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = FileLevelCache::new();
        let stats = cache.stats().await;
        
        assert_eq!(stats.file_content_entries, 0);
        assert_eq!(stats.summary_entries, 0);
        assert_eq!(stats.file_list_entries, 0);
    }

    #[tokio::test]
    async fn test_file_content_cache() {
        let cache = FileLevelCache::new();
        let key = "test:file.rs".to_string();
        
        // Initially empty
        assert!(cache.get_file_content(&key).await.is_none());
        
        // Put file
        let cached_file = CachedFile {
            path: "file.rs".to_string(),
            content: "fn main() {}".to_string(),
            chunks: vec!["chunk1".to_string()],
            summary: None,
            metadata: super::types::FileMetadata {
                file_type: "rs".to_string(),
                size_kb: 1.0,
                chunk_count: 1,
                last_indexed: chrono::Utc::now(),
                language: Some("rust".to_string()),
            },
            cached_at: Instant::now(),
        };
        
        cache.put_file_content(key.clone(), cached_file.clone()).await;
        
        // Retrieve
        let retrieved = cache.get_file_content(&key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "fn main() {}");
    }

    #[tokio::test]
    async fn test_clear_collection() {
        let cache = FileLevelCache::new();
        
        // Add entries for two collections
        cache.put_file_content(
            "coll1:file1.rs".to_string(),
            create_test_file("file1.rs"),
        ).await;
        cache.put_file_content(
            "coll2:file2.rs".to_string(),
            create_test_file("file2.rs"),
        ).await;
        
        // Clear coll1
        cache.clear_collection("coll1").await;
        
        // coll1 should be gone, coll2 should remain
        assert!(cache.get_file_content("coll1:file1.rs").await.is_none());
        assert!(cache.get_file_content("coll2:file2.rs").await.is_some());
    }

    fn create_test_file(name: &str) -> CachedFile {
        CachedFile {
            path: name.to_string(),
            content: "test".to_string(),
            chunks: vec![],
            summary: None,
            metadata: super::types::FileMetadata {
                file_type: "rs".to_string(),
                size_kb: 1.0,
                chunk_count: 1,
                last_indexed: chrono::Utc::now(),
                language: Some("rust".to_string()),
            },
            cached_at: Instant::now(),
        }
    }
}

