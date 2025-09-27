//! Batch Processor
//!
//! Orchestrates batch operations, handling parallel processing, error management,
//! and atomic transactions.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::models::{Vector, SearchResult, Payload};
use crate::error::{Result, VectorizerError};

use super::{
    BatchConfig, BatchError, BatchStatus, BatchErrorType,
    VectorUpdate, SearchQuery,
};

/// Type alias for batch operation results
pub type BatchResult<T> = std::result::Result<T, BatchError>;

/// Manages and executes batch operations on the vector store.
pub struct BatchProcessor {
    config: Arc<BatchConfig>,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<std::sync::Mutex<EmbeddingManager>>,
    // In-progress operations tracking (for progress reporting, cancellation, etc.)
    in_progress_operations: RwLock<HashMap<String, BatchStatus>>,
}

impl BatchProcessor {
    /// Creates a new `BatchProcessor`.
    pub fn new(
        config: Arc<BatchConfig>,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<std::sync::Mutex<EmbeddingManager>>,
    ) -> Self {
        Self {
            config,
            vector_store,
            embedding_manager,
            in_progress_operations: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new batch operation.
    async fn register_operation(&self, operation_id: &str) {
        let mut ops = self.in_progress_operations.write().await;
        ops.insert(operation_id.to_string(), BatchStatus::Partial);
    }

    /// Unregisters a batch operation.
    async fn unregister_operation(&self, operation_id: &str) {
        let mut ops = self.in_progress_operations.write().await;
        ops.remove(operation_id);
    }

    /// Inserts multiple vectors into a collection.
    pub async fn batch_insert(
        &self,
        collection: String,
        vectors: Vec<Vector>,
        atomic: Option<bool>,
        vector_dimension: usize,
    ) -> BatchResult<Vec<String>> {
        let operation_id = format!("batch_insert_{}", Uuid::new_v4());
        self.register_operation(&operation_id).await;

        // Validate batch size
        if !self.config.is_batch_size_valid(vectors.len()) {
            let error = BatchError::new(
                operation_id.clone(),
                BatchErrorType::InvalidBatchSize,
                format!(
                    "Batch size {} exceeds maximum allowed {}",
                    vectors.len(),
                    self.config.max_batch_size
                ),
                None,
            );
            self.unregister_operation(&operation_id).await;
            return Err(error);
        }

        // Validate memory usage
        if self.config.would_exceed_memory_limit(vectors.len(), vector_dimension) {
            let error = BatchError::new(
                operation_id.clone(),
                BatchErrorType::MemoryLimitExceeded,
                format!(
                    "Estimated memory usage for batch ({}MB) exceeds limit ({}MB)",
                    vectors.len() * vector_dimension * 4 / (1024 * 1024),
                    self.config.max_memory_usage_mb
                ),
                None,
            );
            self.unregister_operation(&operation_id).await;
            return Err(error);
        }

        let result = if atomic.unwrap_or(self.config.atomic_by_default) {
            self.batch_insert_atomic(collection, vectors, operation_id.clone()).await
        } else {
            self.batch_insert_non_atomic(collection, vectors, operation_id.clone()).await
        };

        self.unregister_operation(&operation_id).await;
        result
    }

    /// Updates multiple vectors in a collection.
    pub async fn batch_update(
        &self,
        collection: String,
        updates: Vec<VectorUpdate>,
        atomic: Option<bool>,
    ) -> BatchResult<Vec<String>> {
        let operation_id = format!("batch_update_{}", Uuid::new_v4());
        self.register_operation(&operation_id).await;

        // Validate batch size
        if !self.config.is_batch_size_valid(updates.len()) {
            let error = BatchError::new(
                operation_id.clone(),
                BatchErrorType::InvalidBatchSize,
                format!(
                    "Batch size {} exceeds maximum allowed {}",
                    updates.len(),
                    self.config.max_batch_size
                ),
                None,
            );
            self.unregister_operation(&operation_id).await;
            return Err(error);
        }

        let result = if atomic.unwrap_or(self.config.atomic_by_default) {
            self.batch_update_atomic(collection, updates, operation_id.clone()).await
        } else {
            self.batch_update_non_atomic(collection, updates, operation_id.clone()).await
        };

        self.unregister_operation(&operation_id).await;
        result
    }

    /// Deletes multiple vectors from a collection.
    pub async fn batch_delete(
        &self,
        collection: String,
        vector_ids: Vec<String>,
        atomic: Option<bool>,
    ) -> BatchResult<Vec<String>> {
        let operation_id = format!("batch_delete_{}", Uuid::new_v4());
        self.register_operation(&operation_id).await;

        // Validate batch size
        if !self.config.is_batch_size_valid(vector_ids.len()) {
            let error = BatchError::new(
                operation_id.clone(),
                BatchErrorType::InvalidBatchSize,
                format!(
                    "Batch size {} exceeds maximum allowed {}",
                    vector_ids.len(),
                    self.config.max_batch_size
                ),
                None,
            );
            self.unregister_operation(&operation_id).await;
            return Err(error);
        }

        let result = if atomic.unwrap_or(self.config.atomic_by_default) {
            self.batch_delete_atomic(collection, vector_ids, operation_id.clone()).await
        } else {
            self.batch_delete_non_atomic(collection, vector_ids, operation_id.clone()).await
        };

        self.unregister_operation(&operation_id).await;
        result
    }

    /// Searches for multiple queries in a collection.
    pub async fn batch_search(
        &self,
        collection: String,
        queries: Vec<SearchQuery>,
        atomic: Option<bool>,
    ) -> BatchResult<Vec<Vec<SearchResult>>> {
        let operation_id = format!("batch_search_{}", Uuid::new_v4());
        self.register_operation(&operation_id).await;

        // Validate batch size
        if !self.config.is_batch_size_valid(queries.len()) {
            let error = BatchError::new(
                operation_id.clone(),
                BatchErrorType::InvalidBatchSize,
                format!(
                    "Batch size {} exceeds maximum allowed {}",
                    queries.len(),
                    self.config.max_batch_size
                ),
                None,
            );
            self.unregister_operation(&operation_id).await;
            return Err(error);
        }

        let result = if atomic.unwrap_or(self.config.atomic_by_default) {
            self.batch_search_atomic(collection, queries, operation_id.clone()).await
        } else {
            self.batch_search_non_atomic(collection, queries, operation_id.clone()).await
        };

        self.unregister_operation(&operation_id).await;
        result
    }

    // --- Atomic Operations ---

    async fn batch_insert_atomic(
        &self,
        collection: String,
        vectors: Vec<Vector>,
        operation_id: String,
    ) -> BatchResult<Vec<String>> {
        let vector_store = self.vector_store.clone();
        
        // For atomic operations, we try to insert all vectors at once
        match vector_store.insert(&collection, vectors.clone()) {
            Ok(_) => Ok(vectors.into_iter().map(|v| v.id).collect()),
            Err(e) => Err(BatchError::new(
                operation_id,
                BatchErrorType::VectorStoreError,
                e.to_string(),
                None,
            )),
        }
    }

    async fn batch_update_atomic(
        &self,
        collection: String,
        updates: Vec<VectorUpdate>,
        operation_id: String,
    ) -> BatchResult<Vec<String>> {
        let vector_store = self.vector_store.clone();
        let mut successful_ids = Vec::new();

        for update in updates {
            match Self::update_single_vector_static(&vector_store, &collection, &update) {
                Ok(_) => successful_ids.push(update.id),
                Err(e) => {
                    return Err(BatchError::new(
                        operation_id,
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        Some(update.id),
                    ));
                }
            }
        }

        Ok(successful_ids)
    }

    async fn batch_delete_atomic(
        &self,
        collection: String,
        vector_ids: Vec<String>,
        operation_id: String,
    ) -> BatchResult<Vec<String>> {
        let vector_store = self.vector_store.clone();
        let mut successful_ids = Vec::new();

        for vector_id in vector_ids {
            match vector_store.delete(&collection, &vector_id) {
                Ok(_) => successful_ids.push(vector_id),
                Err(e) => {
                    return Err(BatchError::new(
                        operation_id,
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        Some(vector_id),
                    ));
                }
            }
        }

        Ok(successful_ids)
    }

    async fn batch_search_atomic(
        &self,
        collection: String,
        queries: Vec<SearchQuery>,
        operation_id: String,
    ) -> BatchResult<Vec<Vec<SearchResult>>> {
        let vector_store = self.vector_store.clone();
        let embedding_manager = self.embedding_manager.clone();
        let mut results = Vec::new();

        for query in queries {
            match Self::execute_single_search_static(
                &vector_store,
                &embedding_manager,
                &collection,
                query,
            ).await {
                Ok(query_results) => results.push(query_results),
                Err(e) => {
                    return Err(BatchError::new(
                        operation_id,
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        None,
                    ));
                }
            }
        }

        Ok(results)
    }

    // --- Non-Atomic Operations ---

    async fn batch_insert_non_atomic(
        &self,
        collection: String,
        vectors: Vec<Vector>,
        operation_id: String,
    ) -> BatchResult<Vec<String>> {
        let vector_store = self.vector_store.clone();
        let mut successful_ids = Vec::new();
        let mut errors = Vec::new();

        for vector in vectors {
            match vector_store.insert(&collection, vec![vector.clone()]) {
                Ok(_) => successful_ids.push(vector.id),
                Err(e) => {
                    errors.push(BatchError::new(
                        operation_id.clone(),
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        Some(vector.id),
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(successful_ids)
        } else if successful_ids.is_empty() {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                "All insertions failed".to_string(),
                None,
            ))
        } else {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                format!("Partial success with {} errors", errors.len()),
                None,
            ))
        }
    }

    async fn batch_update_non_atomic(
        &self,
        collection: String,
        updates: Vec<VectorUpdate>,
        operation_id: String,
    ) -> BatchResult<Vec<String>> {
        let vector_store = self.vector_store.clone();
        let mut successful_ids = Vec::new();
        let mut errors = Vec::new();

        for update in updates {
            match Self::update_single_vector_static(&vector_store, &collection, &update) {
                Ok(_) => successful_ids.push(update.id),
                Err(e) => {
                    errors.push(BatchError::new(
                        operation_id.clone(),
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        Some(update.id),
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(successful_ids)
        } else if successful_ids.is_empty() {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                "All updates failed".to_string(),
                None,
            ))
        } else {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                format!("Partial success with {} errors", errors.len()),
                None,
            ))
        }
    }

    async fn batch_delete_non_atomic(
        &self,
        collection: String,
        vector_ids: Vec<String>,
        operation_id: String,
    ) -> BatchResult<Vec<String>> {
        let vector_store = self.vector_store.clone();
        let mut successful_ids = Vec::new();
        let mut errors = Vec::new();

        for vector_id in vector_ids {
            match vector_store.delete(&collection, &vector_id) {
                Ok(_) => successful_ids.push(vector_id),
                Err(e) => {
                    errors.push(BatchError::new(
                        operation_id.clone(),
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        Some(vector_id),
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(successful_ids)
        } else if successful_ids.is_empty() {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                "All deletions failed".to_string(),
                None,
            ))
        } else {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                format!("Partial success with {} errors", errors.len()),
                None,
            ))
        }
    }

    async fn batch_search_non_atomic(
        &self,
        collection: String,
        queries: Vec<SearchQuery>,
        operation_id: String,
    ) -> BatchResult<Vec<Vec<SearchResult>>> {
        let vector_store = self.vector_store.clone();
        let embedding_manager = self.embedding_manager.clone();
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for query in queries {
            match Self::execute_single_search_static(
                &vector_store,
                &embedding_manager,
                &collection,
                query,
            ).await {
                Ok(query_results) => results.push(query_results),
                Err(e) => {
                    errors.push(BatchError::new(
                        operation_id.clone(),
                        BatchErrorType::VectorStoreError,
                        e.to_string(),
                        None,
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(results)
        } else if results.is_empty() {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                "All searches failed".to_string(),
                None,
            ))
        } else {
            Err(BatchError::new(
                operation_id,
                BatchErrorType::InternalError,
                format!("Partial success with {} errors", errors.len()),
                None,
            ))
        }
    }

    // Helper for single vector update
    fn update_single_vector_static(
        vector_store: &VectorStore,
        collection: &str,
        update: &VectorUpdate,
    ) -> Result<()> {
        let existing_vector = vector_store.get_vector(collection, &update.id)?;
        
        let updated_vector = Vector {
            id: update.id.clone(),
            data: update.data.clone().unwrap_or(existing_vector.data),
            payload: update.metadata.clone().map(|m| Payload {
                data: serde_json::to_value(m).unwrap_or_default()
            }).or(existing_vector.payload),
        };

        vector_store.update(collection, updated_vector)
    }

    // Helper for single search execution
    async fn execute_single_search_static(
        vector_store: &VectorStore,
        embedding_manager: &Arc<std::sync::Mutex<EmbeddingManager>>,
        collection: &str,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        let query_vector = if let Some(vec) = query.query_vector {
            vec
        } else if let Some(text) = query.query_text {
            let manager = embedding_manager.lock().unwrap();
            manager.embed(&text)?
        } else {
            return Err(VectorizerError::Other("No query vector or text provided".to_string()));
        };

        vector_store.search(
            collection,
            &query_vector,
            query.limit as usize,
        )
    }

    /// Execute a batch operation with unified interface
    pub async fn execute_operation(
        &self,
        collection: String,
        operation: super::BatchOperation,
    ) -> BatchResult<super::BatchResult<String>> {
        use super::BatchOperation;

        let start_time = std::time::Instant::now();

        match operation {
            BatchOperation::Insert { vectors, atomic } => {
                // Get vector dimension from first vector or assume default
                let dimension = vectors.first().map(|v| v.data.len()).unwrap_or(384);
                match self.batch_insert(collection, vectors, Some(atomic), dimension).await {
                    Ok(ids) => {
                        let mut result = super::BatchResult::new();
                        for id in ids {
                            result.add_success(id);
                        }
                        result.processing_time_ms = start_time.elapsed().as_millis() as f64;
                        Ok(result)
                    }
                    Err(e) => Err(e),
                }
            }
            BatchOperation::Update { updates, atomic } => {
                match self.batch_update(collection, updates, Some(atomic)).await {
                    Ok(ids) => {
                        let mut result = super::BatchResult::new();
                        for id in ids {
                            result.add_success(id);
                        }
                        result.processing_time_ms = start_time.elapsed().as_millis() as f64;
                        Ok(result)
                    }
                    Err(e) => Err(e),
                }
            }
            BatchOperation::Delete { vector_ids, atomic } => {
                match self.batch_delete(collection, vector_ids, Some(atomic)).await {
                    Ok(ids) => {
                        let mut result = super::BatchResult::new();
                        for id in ids {
                            result.add_success(id);
                        }
                        result.processing_time_ms = start_time.elapsed().as_millis() as f64;
                        Ok(result)
                    }
                    Err(e) => Err(e),
                }
            }
            BatchOperation::Search { queries, atomic } => {
                match self.batch_search(collection, queries, Some(atomic)).await {
                    Ok(results) => {
                        let mut result = super::BatchResult::new();
                        // For search operations, we return a summary
                        result.add_success("search_completed".to_string());
                        result.processing_time_ms = start_time.elapsed().as_millis() as f64;
                        Ok(result)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}