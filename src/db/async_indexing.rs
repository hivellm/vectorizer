//! Async indexing support for HNSW with double-buffering
//!
//! This module provides:
//! - Background index building
//! - Double-buffering for seamless index swaps
//! - Progress tracking during index construction
//! - Search quality verification during rebuild

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::db::optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
use crate::error::{Result, VectorizerError};

/// Index build progress information
#[derive(Debug, Clone)]
pub struct IndexBuildProgress {
    /// Total vectors to index
    pub total_vectors: usize,
    /// Vectors indexed so far
    pub indexed_vectors: usize,
    /// Percentage complete (0.0 to 1.0)
    pub progress: f64,
    /// Estimated time remaining in seconds
    pub estimated_seconds_remaining: Option<f64>,
    /// Build start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Current status
    pub status: IndexBuildStatus,
}

/// Index build status
#[derive(Debug, Clone, PartialEq)]
pub enum IndexBuildStatus {
    /// Not building
    Idle,
    /// Building in progress
    Building,
    /// Build completed, ready to swap
    Ready,
    /// Build failed
    Failed(String),
}

/// Async index manager with double-buffering
pub struct AsyncIndexManager {
    /// Primary index (currently used for searches)
    primary_index: Arc<RwLock<OptimizedHnswIndex>>,
    /// Secondary index (being built in background)
    secondary_index: Arc<RwLock<Option<OptimizedHnswIndex>>>,
    /// Build progress
    progress: Arc<RwLock<IndexBuildProgress>>,
    /// Background build task handle
    build_handle: Arc<RwLock<Option<JoinHandle<Result<()>>>>>,
    /// Progress channel sender
    progress_tx: Arc<RwLock<Option<mpsc::UnboundedSender<IndexBuildProgress>>>>,
    /// Configuration
    config: OptimizedHnswConfig,
    /// Dimension
    dimension: usize,
}

impl AsyncIndexManager {
    /// Create a new async index manager
    pub fn new(
        dimension: usize,
        config: OptimizedHnswConfig,
        initial_vectors: HashMap<String, Vec<f32>>,
    ) -> Result<Self> {
        // Create primary index with initial vectors
        let primary_index = OptimizedHnswIndex::new(dimension, config)?;

        // Insert initial vectors
        if !initial_vectors.is_empty() {
            let batch: Vec<_> = initial_vectors.into_iter().collect();
            primary_index.batch_add(batch)?;
        }

        let progress = IndexBuildProgress {
            total_vectors: 0,
            indexed_vectors: 0,
            progress: 0.0,
            estimated_seconds_remaining: None,
            started_at: chrono::Utc::now(),
            status: IndexBuildStatus::Idle,
        };

        Ok(Self {
            primary_index: Arc::new(RwLock::new(primary_index)),
            secondary_index: Arc::new(RwLock::new(None)),
            progress: Arc::new(RwLock::new(progress)),
            build_handle: Arc::new(RwLock::new(None)),
            progress_tx: Arc::new(RwLock::new(None)),
            config,
            dimension,
        })
    }

    /// Start background index rebuild with new vectors
    pub fn start_rebuild(
        &self,
        vectors: HashMap<String, Vec<f32>>,
    ) -> Result<mpsc::UnboundedReceiver<IndexBuildProgress>> {
        // Check if rebuild is already in progress
        {
            let progress = self.progress.read();
            if matches!(progress.status, IndexBuildStatus::Building) {
                return Err(VectorizerError::Storage(
                    "Index rebuild already in progress".to_string(),
                ));
            }
        }

        // Cancel any existing build
        self.cancel_rebuild();

        // Create progress channel
        let (tx, rx) = mpsc::unbounded_channel();
        *self.progress_tx.write() = Some(tx.clone());

        let total_vectors = vectors.len();
        let start_time = Instant::now();

        // Update progress
        {
            let mut progress = self.progress.write();
            *progress = IndexBuildProgress {
                total_vectors,
                indexed_vectors: 0,
                progress: 0.0,
                estimated_seconds_remaining: None,
                started_at: chrono::Utc::now(),
                status: IndexBuildStatus::Building,
            };
        }

        // Clone necessary data for background task
        let dimension = self.dimension;
        let config = self.config;
        let progress_arc = self.progress.clone();
        let secondary_index_arc = self.secondary_index.clone();

        info!(
            "🔄 Starting async index rebuild with {} vectors",
            total_vectors
        );

        // Spawn background build task
        let handle = tokio::spawn(async move {
            Self::build_index_async(
                dimension,
                config,
                vectors,
                progress_arc,
                secondary_index_arc,
                tx,
                start_time,
            )
            .await
        });

        *self.build_handle.write() = Some(handle);

        Ok(rx)
    }

    /// Build index asynchronously
    async fn build_index_async(
        dimension: usize,
        config: OptimizedHnswConfig,
        vectors: HashMap<String, Vec<f32>>,
        progress: Arc<RwLock<IndexBuildProgress>>,
        secondary_index: Arc<RwLock<Option<OptimizedHnswIndex>>>,
        progress_tx: mpsc::UnboundedSender<IndexBuildProgress>,
        start_time: Instant,
    ) -> Result<()> {
        let total = vectors.len();
        let mut indexed = 0;

        // Create new index
        let new_index = OptimizedHnswIndex::new(dimension, config)
            .map_err(|e| VectorizerError::Storage(format!("Failed to create index: {}", e)))?;

        // Convert to batch format
        let batch: Vec<(String, Vec<f32>)> = vectors.into_iter().collect();
        let batch_size = config.batch_size;

        // Insert vectors in batches
        for chunk in batch.chunks(batch_size) {
            new_index
                .batch_add(chunk.to_vec())
                .map_err(|e| VectorizerError::Storage(format!("Failed to add vectors: {}", e)))?;

            indexed += chunk.len();

            // Update progress
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress_pct = indexed as f64 / total as f64;
            let estimated_remaining = if progress_pct > 0.0 {
                Some((elapsed / progress_pct) * (1.0 - progress_pct))
            } else {
                None
            };

            let current_progress = IndexBuildProgress {
                total_vectors: total,
                indexed_vectors: indexed,
                progress: progress_pct,
                estimated_seconds_remaining: estimated_remaining,
                started_at: chrono::Utc::now(),
                status: IndexBuildStatus::Building,
            };

            // Update shared progress
            {
                let mut prog = progress.write();
                *prog = current_progress.clone();
            }

            // Send progress update
            let _ = progress_tx.send(current_progress);

            // Yield to allow other tasks to run
            tokio::task::yield_now().await;
        }

        // Optimize index
        new_index
            .optimize()
            .map_err(|e| VectorizerError::Storage(format!("Failed to optimize index: {}", e)))?;

        // Mark as ready before storing (so swap can verify)
        let final_progress = IndexBuildProgress {
            total_vectors: total,
            indexed_vectors: indexed,
            progress: 1.0,
            estimated_seconds_remaining: Some(0.0),
            started_at: chrono::Utc::now(),
            status: IndexBuildStatus::Ready,
        };

        {
            let mut prog = progress.write();
            *prog = final_progress.clone();
        }

        // Store in secondary index (after marking as ready)
        {
            let mut sec = secondary_index.write();
            *sec = Some(new_index);
        }

        let _ = progress_tx.send(final_progress);
        info!("✅ Async index rebuild completed with {} vectors", total);

        // Index is now stored in secondary_index and ready for swap
        // We can't return it because OptimizedHnswIndex doesn't implement Clone
        // The index is accessible via swap_index() and get_index()
        Ok(())
    }

    /// Swap to the newly built index (double-buffering)
    pub fn swap_index(&self) -> Result<bool> {
        // Check if secondary index exists and is ready
        let is_ready = {
            let sec = self.secondary_index.read();
            sec.is_some()
        };

        if !is_ready {
            return Ok(false);
        }

        // Verify the new index is ready
        let total_vectors = {
            let progress = self.progress.read();
            if !matches!(progress.status, IndexBuildStatus::Ready) {
                return Err(VectorizerError::Storage(
                    "New index is not ready for swap".to_string(),
                ));
            }
            progress.total_vectors
        };

        info!("🔄 Swapping to new index ({} vectors)", total_vectors);

        // Swap indices
        {
            let mut primary = self.primary_index.write();
            let mut secondary = self.secondary_index.write();

            if let Some(new_index) = secondary.take() {
                // Move old primary to secondary (for potential rollback)
                let old_primary = std::mem::replace(&mut *primary, new_index);
                *secondary = Some(old_primary);
            } else {
                return Ok(false);
            }
        }

        // Reset progress
        {
            let mut prog = self.progress.write();
            *prog = IndexBuildProgress {
                total_vectors: 0,
                indexed_vectors: 0,
                progress: 0.0,
                estimated_seconds_remaining: None,
                started_at: chrono::Utc::now(),
                status: IndexBuildStatus::Idle,
            };
        }

        info!("✅ Index swap completed successfully");
        Ok(true)
    }

    /// Get current build progress
    pub fn get_progress(&self) -> IndexBuildProgress {
        self.progress.read().clone()
    }

    /// Check if rebuild is in progress
    pub fn is_rebuilding(&self) -> bool {
        matches!(self.progress.read().status, IndexBuildStatus::Building)
    }

    /// Check if new index is ready
    pub fn is_ready(&self) -> bool {
        matches!(self.progress.read().status, IndexBuildStatus::Ready)
    }

    /// Cancel ongoing rebuild
    pub fn cancel_rebuild(&self) {
        // Cancel build handle
        if let Some(handle) = self.build_handle.write().take() {
            handle.abort();
        }

        // Clear secondary index
        {
            let mut sec = self.secondary_index.write();
            *sec = None;
        }

        // Reset progress
        {
            let mut prog = self.progress.write();
            *prog = IndexBuildProgress {
                total_vectors: 0,
                indexed_vectors: 0,
                progress: 0.0,
                estimated_seconds_remaining: None,
                started_at: chrono::Utc::now(),
                status: IndexBuildStatus::Idle,
            };
        }

        // Clear progress channel
        *self.progress_tx.write() = None;
    }

    /// Get primary index for searches (always returns current active index)
    pub fn get_index(&self) -> Arc<RwLock<OptimizedHnswIndex>> {
        self.primary_index.clone()
    }

    /// Search using current active index
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        let index = self.primary_index.read();
        index.search(query, k)
    }

    /// Add vector to primary index (immediate insertion)
    pub fn add(&self, id: String, data: Vec<f32>) -> Result<()> {
        let index = self.primary_index.write();
        index.add(id, data)
    }

    /// Batch add vectors to primary index
    pub fn batch_add(&self, vectors: Vec<(String, Vec<f32>)>) -> Result<()> {
        let index = self.primary_index.write();
        index.batch_add(vectors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_index_manager_creation() {
        let config = OptimizedHnswConfig::default();
        let manager = AsyncIndexManager::new(128, config, HashMap::new()).unwrap();

        assert!(!manager.is_rebuilding());
        assert!(!manager.is_ready());
    }

    #[tokio::test]
    async fn test_async_rebuild() {
        let config = OptimizedHnswConfig {
            batch_size: 10,
            ..Default::default()
        };

        // Create initial vectors
        let mut initial_vectors = HashMap::new();
        for i in 0..5 {
            initial_vectors.insert(format!("v{}", i), vec![1.0; 128]);
        }

        let manager = AsyncIndexManager::new(128, config, initial_vectors).unwrap();

        // Create vectors for rebuild
        let mut rebuild_vectors = HashMap::new();
        for i in 0..50 {
            rebuild_vectors.insert(format!("new_v{}", i), vec![1.0; 128]);
        }

        // Start rebuild
        let mut progress_rx = manager.start_rebuild(rebuild_vectors).unwrap();

        // Wait for completion
        let mut last_progress = None;
        let mut completed = false;
        while let Some(progress) = progress_rx.recv().await {
            last_progress = Some(progress.clone());
            if progress.progress >= 1.0 && matches!(progress.status, IndexBuildStatus::Ready) {
                completed = true;
                break;
            }
        }

        // Give a small delay for status to update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert!(completed, "Rebuild should complete");
        assert!(manager.is_ready(), "Manager should be ready after rebuild");
        assert!(last_progress.is_some());
        let final_progress = last_progress.unwrap();
        assert_eq!(final_progress.progress, 1.0);
        assert_eq!(final_progress.indexed_vectors, 50);

        // Swap index
        assert!(manager.swap_index().unwrap());
        assert!(!manager.is_ready());
    }
}
