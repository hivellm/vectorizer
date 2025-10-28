//! Batch Operations Definitions
//!
//! Defines the structures and traits for various batch operations (insert, update, delete, search),
//! and a manager to orchestrate them.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{BatchConfig, BatchError, BatchErrorType, BatchProcessor};
use crate::models::Vector;
use crate::search::SearchQuery;

/// Vector update operation
#[derive(Debug, Clone)]
pub struct VectorUpdate {
    pub id: String,
    pub vector: Option<Vector>,
    pub payload: Option<serde_json::Value>,
}

/// Batch operation status
#[derive(Debug, Clone)]
pub struct BatchStatus {
    pub operation_id: String,
    pub status: String,
    pub progress: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

use crate::db::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::error::Result;
use crate::models::SearchResult;

/// Type alias for batch operation results
pub type BatchResult<T> = std::result::Result<T, BatchError>;

/// Trait for defining common behavior of batch operations.
#[async_trait]
pub trait BatchOperationTrait: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;

    fn operation_id(&self) -> &str;
    fn collection_name(&self) -> &str;
    fn is_atomic(&self) -> bool;
    async fn execute<T, R>(
        &self,
        processor: &T,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output>
    where
        T: BatchProcessor<Self::Input, R> + ?Sized,
        R: Send + Sync;
}

/// Represents a batch insert operation.
pub struct BatchInsertOperation {
    pub id: String,
    pub collection: String,
    pub vectors: Vec<Vector>,
    pub atomic: bool,
    pub vector_dimension: usize,
}

#[async_trait]
impl BatchOperationTrait for BatchInsertOperation {
    type Input = Vec<Vector>;
    type Output = Vec<String>;

    fn operation_id(&self) -> &str {
        &self.id
    }
    fn collection_name(&self) -> &str {
        &self.collection
    }
    fn is_atomic(&self) -> bool {
        self.atomic
    }

    async fn execute<T, R>(
        &self,
        _processor: &T,
        vector_store: Arc<VectorStore>,
        _embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output>
    where
        T: BatchProcessor<Self::Input, R> + ?Sized,
        R: Send + Sync,
    {
        // Get the collection
        let collection = vector_store
            .get_collection(&self.collection)
            .map_err(|e| BatchError::new(BatchErrorType::CollectionNotFound, &e.to_string()))?;

        // Insert vectors in batch and collect IDs
        let inserted_ids: Vec<String> = self.vectors.iter().map(|v| v.id.clone()).collect();

        vector_store
            .insert(&self.collection, self.vectors.clone())
            .map_err(|e| BatchError::new(BatchErrorType::InsertionFailed, &e.to_string()))?;

        Ok(inserted_ids)
    }
}

/// Represents a batch update operation.
pub struct BatchUpdateOperation {
    pub id: String,
    pub collection: String,
    pub updates: Vec<VectorUpdate>,
    pub atomic: bool,
}

// Note: BatchUpdateOperation implementation removed temporarily
// Requires architectural change to support mutable collection access in batch context
// Alternative: Use individual update_vector calls or implement collection-level batch update API

#[async_trait]
impl BatchOperationTrait for BatchUpdateOperation {
    type Input = Vec<VectorUpdate>;
    type Output = Vec<String>;

    fn operation_id(&self) -> &str {
        &self.id
    }
    fn collection_name(&self) -> &str {
        &self.collection
    }
    fn is_atomic(&self) -> bool {
        self.atomic
    }

    async fn execute<T, R>(
        &self,
        _processor: &T,
        vector_store: Arc<VectorStore>,
        _embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output>
    where
        T: BatchProcessor<Self::Input, R> + ?Sized,
        R: Send + Sync,
    {
        // Batch update requires mutable collection access which is not currently supported
        // in the immutable Arc<CollectionType> architecture
        Err(BatchError::new(
            BatchErrorType::UpdateFailed,
            "Batch update not yet implemented - requires mutable collection access",
        ))
    }
}

/// Represents a batch delete operation.
pub struct BatchDeleteOperation {
    pub id: String,
    pub collection: String,
    pub vector_ids: Vec<String>,
    pub atomic: bool,
}

#[async_trait]
impl BatchOperationTrait for BatchDeleteOperation {
    type Input = Vec<String>;
    type Output = Vec<String>;

    fn operation_id(&self) -> &str {
        &self.id
    }
    fn collection_name(&self) -> &str {
        &self.collection
    }
    fn is_atomic(&self) -> bool {
        self.atomic
    }

    async fn execute<T, R>(
        &self,
        _processor: &T,
        vector_store: Arc<VectorStore>,
        _embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output>
    where
        T: BatchProcessor<Self::Input, R> + ?Sized,
        R: Send + Sync,
    {
        // Batch delete requires mutable collection access which is not currently supported
        // in the immutable Arc<CollectionType> architecture
        Err(BatchError::new(
            BatchErrorType::DeletionFailed,
            "Batch delete not yet implemented - requires mutable collection access",
        ))
    }
}

/// Represents a batch search operation.
pub struct BatchSearchOperation {
    pub id: String,
    pub collection: String,
    pub queries: Vec<SearchQuery>,
    pub atomic: bool,
}

#[async_trait]
impl BatchOperationTrait for BatchSearchOperation {
    type Input = Vec<SearchQuery>;
    type Output = Vec<Vec<SearchResult>>;

    fn operation_id(&self) -> &str {
        &self.id
    }
    fn collection_name(&self) -> &str {
        &self.collection
    }
    fn is_atomic(&self) -> bool {
        self.atomic
    }

    async fn execute<T, R>(
        &self,
        _processor: &T,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output>
    where
        T: BatchProcessor<Self::Input, R> + ?Sized,
        R: Send + Sync,
    {
        // Get collection
        let collection = vector_store
            .get_collection(&self.collection)
            .map_err(|e| {
                BatchError::new(
                    BatchErrorType::SearchFailed,
                    &format!("Failed to get collection '{}': {}", self.collection, e),
                )
            })?;

        let mut all_results = Vec::new();

        // Get embedding manager
        let emb_mgr = embedding_manager.read().await;

        // Process each search query
        for (idx, query) in self.queries.iter().enumerate() {
            // Convert query text to embedding
            let query_result = match emb_mgr.embed(&query.query).await {
                Ok(emb) => emb,
                Err(e) => {
                    if self.atomic {
                        return Err(BatchError::new(
                            BatchErrorType::SearchFailed,
                            &format!("Failed to embed query {}: {}", idx, e),
                        ));
                    }
                    tracing::warn!("Failed to embed query {}: {}", idx, e);
                    all_results.push(Vec::new());
                    continue;
                }
            };

            let query_embedding = query_result.embedding;

            // Perform search
            match collection.search(&query_embedding, query.max_results) {
                Ok(results) => all_results.push(results),
                Err(e) => {
                    if self.atomic {
                        // In atomic mode, fail entire operation on first error
                        return Err(BatchError::new(
                            BatchErrorType::SearchFailed,
                            &format!("Atomic batch search failed at query {}: {}", idx, e),
                        ));
                    }
                    // In non-atomic mode, log error and return empty results
                    tracing::warn!("Search failed for query {}: {}", idx, e);
                    all_results.push(Vec::new());
                }
            }
        }

        Ok(all_results)
    }
}

/// Manages and dispatches various batch operations.
pub struct BatchOperationManager {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
}

impl BatchOperationManager {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
        }
    }

    /// Executes any batch operation.
    pub async fn execute_operation<
        T: BatchOperationTrait + ?Sized,
        P: BatchProcessor<T::Input, T::Output> + ?Sized,
    >(
        &self,
        operation: &T,
        processor: &P,
    ) -> BatchResult<T::Output> {
        operation
            .execute(
                processor,
                self.vector_store.clone(),
                self.embedding_manager.clone(),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Payload;

    #[test]
    fn test_batch_insert_operation_creation() {
        let op = BatchInsertOperation {
            id: "test_insert".to_string(),
            collection: "test_coll".to_string(),
            vectors: vec![],
            atomic: true,
            vector_dimension: 128,
        };

        assert_eq!(op.operation_id(), "test_insert");
        assert_eq!(op.collection_name(), "test_coll");
        assert!(op.is_atomic());
    }

    #[test]
    fn test_batch_update_operation_creation() {
        let op = BatchUpdateOperation {
            id: "test_update".to_string(),
            collection: "test_coll".to_string(),
            updates: vec![],
            atomic: false,
        };

        assert_eq!(op.operation_id(), "test_update");
        assert_eq!(op.collection_name(), "test_coll");
        assert!(!op.is_atomic());
    }

    #[test]
    fn test_batch_delete_operation_creation() {
        let op = BatchDeleteOperation {
            id: "test_delete".to_string(),
            collection: "test_coll".to_string(),
            vector_ids: vec!["id1".to_string(), "id2".to_string()],
            atomic: true,
        };

        assert_eq!(op.operation_id(), "test_delete");
        assert_eq!(op.collection_name(), "test_coll");
        assert!(op.is_atomic());
        assert_eq!(op.vector_ids.len(), 2);
    }

    #[test]
    fn test_batch_search_operation_creation() {
        let query = SearchQuery {
            query: "test query".to_string(),
            mode: crate::search::advanced_search::SearchMode::Hybrid,
            collections: vec!["test_coll".to_string()],
            max_results: 10,
            offset: 0,
            sort: None,
            filters: HashMap::new(),
            facets: vec![],
            highlight: None,
        };

        let op = BatchSearchOperation {
            id: "test_search".to_string(),
            collection: "test_coll".to_string(),
            queries: vec![query],
            atomic: false,
        };

        assert_eq!(op.operation_id(), "test_search");
        assert_eq!(op.collection_name(), "test_coll");
        assert!(!op.is_atomic());
        assert_eq!(op.queries.len(), 1);
    }
}
