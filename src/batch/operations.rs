//! Batch Operations Definitions
//!
//! Defines the structures and traits for various batch operations (insert, update, delete, search),
//! and a manager to orchestrate them.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{
    BatchConfig, BatchError, BatchErrorType, BatchProcessor, BatchStatus, SearchQuery, VectorUpdate,
};
use crate::db::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::error::Result;
use crate::models::{SearchResult, Vector};

/// Type alias for batch operation results
pub type BatchResult<T> = std::result::Result<T, BatchError>;

/// Trait for defining common behavior of batch operations.
#[async_trait]
pub trait BatchOperationTrait {
    type Input;
    type Output;

    fn operation_id(&self) -> &str;
    fn collection_name(&self) -> &str;
    fn is_atomic(&self) -> bool;
    async fn execute(
        &self,
        processor: &BatchProcessor,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output>;
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

    async fn execute(
        &self,
        processor: &BatchProcessor,
        _vector_store: Arc<VectorStore>, // Not directly used here, passed to processor
        _embedding_manager: Arc<RwLock<EmbeddingManager>>, // Not directly used here
    ) -> BatchResult<Self::Output> {
        processor
            .batch_insert(
                self.collection.clone(),
                self.vectors.clone(),
                Some(self.atomic),
                self.vector_dimension,
            )
            .await
    }
}

/// Represents a batch update operation.
pub struct BatchUpdateOperation {
    pub id: String,
    pub collection: String,
    pub updates: Vec<VectorUpdate>,
    pub atomic: bool,
}

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

    async fn execute(
        &self,
        processor: &BatchProcessor,
        _vector_store: Arc<VectorStore>,
        _embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output> {
        processor
            .batch_update(
                self.collection.clone(),
                self.updates.clone(),
                Some(self.atomic),
            )
            .await
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

    async fn execute(
        &self,
        processor: &BatchProcessor,
        _vector_store: Arc<VectorStore>,
        _embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output> {
        processor
            .batch_delete(
                self.collection.clone(),
                self.vector_ids.clone(),
                Some(self.atomic),
            )
            .await
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

    async fn execute(
        &self,
        processor: &BatchProcessor,
        _vector_store: Arc<VectorStore>,
        _embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> BatchResult<Self::Output> {
        processor
            .batch_search(
                self.collection.clone(),
                self.queries.clone(),
                Some(self.atomic),
            )
            .await
    }
}

/// Manages and dispatches various batch operations.
pub struct BatchOperationManager {
    processor: Arc<BatchProcessor>,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
}

impl BatchOperationManager {
    pub fn new(
        processor: Arc<BatchProcessor>,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> Self {
        Self {
            processor,
            vector_store,
            embedding_manager,
        }
    }

    /// Executes any batch operation.
    pub async fn execute_operation<T: BatchOperationTrait + ?Sized>(
        &self,
        operation: &T,
    ) -> BatchResult<T::Output> {
        operation
            .execute(
                &self.processor,
                self.vector_store.clone(),
                self.embedding_manager.clone(),
            )
            .await
    }

    /// Get active operations
    pub async fn get_active_operations(&self) -> HashMap<String, BatchStatus> {
        // TODO: Implement this method in BatchProcessor
        HashMap::new()
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
            query_vector: Some(vec![0.1; 128]),
            query_text: None,
            limit: 10,
            threshold: Some(0.7),
            filters: HashMap::new(),
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
