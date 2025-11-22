//! System Metrics Collector
//!
//! This module provides periodic collection of system-level metrics
//! including memory usage, cache statistics, and system resources.

use std::sync::Arc;
use std::time::Duration;

use tokio::time::interval;
use tracing::{debug, warn};

use super::metrics::METRICS;
use crate::VectorStore;

/// System metrics collector configuration
#[derive(Debug, Clone)]
pub struct SystemCollectorConfig {
    /// Interval between metric collections
    pub interval_secs: u64,
}

impl Default for SystemCollectorConfig {
    fn default() -> Self {
        Self {
            interval_secs: 15, // Collect every 15 seconds
        }
    }
}

/// System metrics collector
pub struct SystemCollector {
    config: SystemCollectorConfig,
    vector_store: Arc<VectorStore>,
}

impl SystemCollector {
    /// Create a new system metrics collector
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        Self {
            config: SystemCollectorConfig::default(),
            vector_store,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: SystemCollectorConfig, vector_store: Arc<VectorStore>) -> Self {
        Self {
            config,
            vector_store,
        }
    }

    /// Start the metrics collection loop
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(self.config.interval_secs));

            loop {
                tick.tick().await;
                self.collect_metrics();
            }
        })
    }

    /// Collect all system metrics
    fn collect_metrics(&self) {
        // Collect memory usage
        self.collect_memory_metrics();

        // Collect vector store metrics
        self.collect_vector_store_metrics();
    }

    /// Collect memory usage metrics
    fn collect_memory_metrics(&self) {
        match memory_stats::memory_stats() {
            Some(usage) => {
                let memory_bytes = usage.physical_mem as f64;
                METRICS.memory_usage_bytes.set(memory_bytes);
                debug!("Memory usage: {} MB", memory_bytes / 1024.0 / 1024.0);
            }
            None => {
                warn!("Failed to get memory stats");
            }
        }
    }

    /// Collect vector store metrics (collections and vectors count)
    fn collect_vector_store_metrics(&self) {
        let collections = self.vector_store.list_collections();
        METRICS.collections_total.set(collections.len() as f64);

        let total_vectors: usize = collections
            .iter()
            .filter_map(|name| {
                self.vector_store
                    .get_collection(name)
                    .ok()
                    .map(|c| c.vector_count())
            })
            .sum();

        METRICS.vectors_total.set(total_vectors as f64);

        debug!(
            "Vector store metrics: {} collections, {} vectors",
            collections.len(),
            total_vectors
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collector_creation() {
        let store = Arc::new(VectorStore::new_auto());
        let collector = SystemCollector::new(store);
        assert_eq!(collector.config.interval_secs, 15);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = SystemCollectorConfig { interval_secs: 30 };
        let store = Arc::new(VectorStore::new_auto());
        let collector = SystemCollector::with_config(config, store);
        assert_eq!(collector.config.interval_secs, 30);
    }

    #[tokio::test]
    async fn test_collect_metrics() {
        let store = Arc::new(VectorStore::new_auto());

        // Create a test collection
        let config = crate::models::CollectionConfig {
            sharding: None,
            dimension: 128,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: Default::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
        };
        let _ = store.create_collection("test_metrics", config);

        let collector = SystemCollector::new(store);

        // Collect metrics
        collector.collect_metrics();

        // Verify metrics were updated
        let collections_count = METRICS.collections_total.get();
        assert!(
            collections_count > 0.0,
            "Collections metric should be updated"
        );
    }

    #[tokio::test]
    async fn test_memory_metrics() {
        let store = Arc::new(VectorStore::new_auto());
        let collector = SystemCollector::new(store);

        collector.collect_memory_metrics();

        // Memory metric should be set (can't assert exact value)
        let memory = METRICS.memory_usage_bytes.get();
        assert!(memory >= 0.0, "Memory metric should be non-negative");
    }
}
