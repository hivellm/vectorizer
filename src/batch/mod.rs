//! Batch Operations Module - Phase 6
//! 
//! This module implements high-performance batch operations for vector management,
//! enabling AI models to perform multiple operations in a single API call.
//! 
//! Key features:
//! - Batch insert, update, delete, and search operations
//! - Atomic transactions (all succeed or all fail)
//! - Parallel processing with configurable workers
//! - Memory-efficient streaming for large batches
//! - Comprehensive error handling and reporting
//! 
//! Performance targets:
//! - 10,000 vectors/second batch insert
//! - 10x throughput improvement over individual operations
//! - <100ms latency for batches up to 1,000 operations

pub mod processor;
pub mod operations;
pub mod config;
pub mod error;
pub mod parallel;
pub mod validation;

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::db::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::models::Vector;

pub use processor::BatchProcessor;
pub use config::BatchConfig;
pub use error::{BatchError, BatchResult, BatchStatus, BatchErrorType};
pub use operations::{
    BatchInsertOperation,
    BatchUpdateOperation, 
    BatchDeleteOperation,
    BatchSearchOperation,
    BatchOperationManager,
};
pub use parallel::ParallelProcessor;
pub use validation::BatchValidator;

/// Batch operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperation {
    Insert {
        vectors: Vec<Vector>,
        atomic: bool,
    },
    Update {
        updates: Vec<VectorUpdate>,
        atomic: bool,
    },
    Delete {
        vector_ids: Vec<String>,
        atomic: bool,
    },
    Search {
        queries: Vec<SearchQuery>,
        atomic: bool,
    },
}

/// Vector update structure for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorUpdate {
    pub id: String,
    pub data: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Search query for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query_vector: Option<Vec<f32>>,
    pub query_text: Option<String>,
    pub limit: i32,
    pub threshold: Option<f32>,
    pub filters: HashMap<String, String>,
}

/// Batch processor factory
pub struct BatchProcessorBuilder {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    config: BatchConfig,
}

impl BatchProcessorBuilder {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
            config: BatchConfig::default(),
        }
    }

    pub fn with_config(mut self, config: BatchConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> BatchProcessor {
        BatchProcessor::new(
            Arc::new(self.config),
            self.vector_store,
            self.embedding_manager,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 1000);
        assert_eq!(config.max_memory_usage_mb, 512);
        assert_eq!(config.parallel_workers, 4);
        assert_eq!(config.chunk_size, 100);
        assert!(config.atomic_by_default);
        assert!(config.progress_reporting);
    }

    #[test]
    fn test_batch_status_equality() {
        assert_eq!(BatchStatus::Success, BatchStatus::Success);
        assert_ne!(BatchStatus::Success, BatchStatus::Failed);
    }

    #[test]
    fn test_vector_update_creation() {
        let update = VectorUpdate {
            id: "test_id".to_string(),
            data: Some(vec![0.1, 0.2, 0.3]),
            metadata: Some(HashMap::new()),
        };
        
        assert_eq!(update.id, "test_id");
        assert_eq!(update.data, Some(vec![0.1, 0.2, 0.3]));
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery {
            query_vector: Some(vec![0.1, 0.2, 0.3]),
            query_text: Some("test query".to_string()),
            limit: 10,
            threshold: Some(0.8),
            filters: HashMap::new(),
        };
        
        assert_eq!(query.limit, 10);
        assert_eq!(query.threshold, Some(0.8));
    }
}
