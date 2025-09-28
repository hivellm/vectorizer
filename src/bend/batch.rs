//! Bend Batch Operations Integration
//! 
//! This module integrates Bend with the Vectorizer batch operations system,
//! enabling automatic parallelization of batch similarity searches.

use std::sync::Arc;
use crate::batch::{BatchProcessor, BatchConfig, BatchError, BatchResult, SearchQuery};
use crate::bend::{BendVectorOperations, BendConfig};
use crate::models::SearchResult;
use crate::error::VectorizerError;

/// Bend-enhanced batch processor
pub struct BendBatchProcessor {
    /// Original batch processor
    base_processor: Arc<BatchProcessor>,
    /// Bend vector operations
    bend_operations: BendVectorOperations,
    /// Bend configuration
    bend_config: BendConfig,
}

impl BendBatchProcessor {
    /// Create a new Bend-enhanced batch processor
    pub fn new(
        base_processor: Arc<BatchProcessor>,
        bend_config: BendConfig,
    ) -> Self {
        let bend_operations = BendVectorOperations::new(bend_config.clone());
        
        Self {
            base_processor,
            bend_operations,
            bend_config,
        }
    }

    /// Perform batch search with Bend acceleration
    pub async fn batch_search_with_bend(
        &self,
        collection: String,
        queries: Vec<SearchQuery>,
        atomic: Option<bool>,
    ) -> Result<Vec<Vec<SearchResult>>, BatchError> {
        // For now, use the base processor directly
        // In a full implementation, we would integrate Bend here
        self.base_processor.batch_search(collection, queries, atomic).await
    }

    /// Perform single search with Bend acceleration
    pub async fn search_with_bend(
        &self,
        collection: String,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>, BatchError> {
        // For now, use the base processor directly
        // In a full implementation, we would integrate Bend here
        self.base_processor.batch_search(collection, vec![query], Some(true)).await
            .map(|results| results.into_iter().next().unwrap_or_default())
    }

    /// Get Bend configuration
    pub fn bend_config(&self) -> &BendConfig {
        &self.bend_config
    }

    /// Update Bend configuration
    pub fn update_bend_config(&mut self, config: BendConfig) {
        self.bend_config = config.clone();
        self.bend_operations = BendVectorOperations::new(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bend_batch_processor_creation() {
        let base_processor = Arc::new(BatchProcessor::new(BatchConfig::default()));
        let bend_config = BendConfig::default();
        
        let processor = BendBatchProcessor::new(base_processor, bend_config);
        assert!(processor.bend_config().enabled);
    }
}