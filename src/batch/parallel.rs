//! Parallel Processing for Batch Operations
//!
//! Provides utilities for processing items in parallel, chunking large inputs
//! and managing concurrent execution for batch operations.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use crate::batch::error::{BatchError, BatchErrorType};

/// Type alias for batch operation results
pub type BatchResult<T> = std::result::Result<T, BatchError>;

use crate::batch::config::BatchConfig;

/// A trait for items that can be processed in a batch.
pub trait BatchItem: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> BatchItem for T {}

/// Manages parallel processing of items in chunks.
pub struct ParallelProcessor {
    config: Arc<BatchConfig>,
    semaphore: Arc<Semaphore>,
}

impl ParallelProcessor {
    /// Creates a new `ParallelProcessor`.
    pub fn new(config: Arc<BatchConfig>) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.parallel_workers));
        Self { config, semaphore }
    }

    /// Processes a list of items in parallel, chunking them as configured.
    pub async fn process_chunks<T, R, F, Fut>(
        &self,
        items: Vec<T>,
        processor: F,
    ) -> BatchResult<Vec<R>>
    where
        T: BatchItem + Clone,
        R: BatchItem,
        F: Fn(Vec<T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = BatchResult<Vec<R>>> + Send + 'static,
    {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let chunk_size = self.config.chunk_size;
        let mut handles = Vec::new();

        for chunk_start in (0..items.len()).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(items.len());
            let chunk = items[chunk_start..chunk_end].to_vec();

            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let processor_clone = processor.clone();
            let handle = tokio::spawn(async move {
                let result = processor_clone(chunk).await;
                drop(permit);
                result
            });
            handles.push(handle);
        }

        let mut successful_results = Vec::new();
        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(results)) => successful_results.extend(results),
                Ok(Err(e)) => errors.push(e),
                Err(e) => errors.push(BatchError::new(
                    "unknown".to_string(),
                    BatchErrorType::InternalError,
                    format!("Task join error: {}", e),
                    None,
                )),
            }
        }

        if errors.is_empty() {
            Ok(successful_results)
        } else if successful_results.is_empty() {
            Err(BatchError::new(
                "unknown".to_string(),
                BatchErrorType::InternalError,
                format!("All chunks failed: {:?}", errors),
                None,
            ))
        } else {
            Err(BatchError::new(
                "unknown".to_string(),
                BatchErrorType::InternalError,
                format!("Partial success with errors: {:?}", errors),
                None,
            ))
        }
    }

    /// Processes a list of individual items in parallel.
    pub async fn process_items<T, R, F, Fut>(
        &self,
        items: Vec<T>,
        processor: F,
    ) -> BatchResult<Vec<R>>
    where
        T: BatchItem + Clone,
        R: BatchItem,
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = BatchResult<R>> + Send + 'static,
    {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut handles = Vec::new();

        for item in items {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let processor_clone = processor.clone();
            let handle = tokio::spawn(async move {
                let result = processor_clone(item).await;
                drop(permit);
                result
            });
            handles.push(handle);
        }

        let mut successful_results = Vec::new();
        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => successful_results.push(result),
                Ok(Err(e)) => errors.push(e),
                Err(e) => errors.push(BatchError::new(
                    "unknown".to_string(),
                    BatchErrorType::InternalError,
                    format!("Task join error: {}", e),
                    None,
                )),
            }
        }

        if errors.is_empty() {
            Ok(successful_results)
        } else if successful_results.is_empty() {
            Err(BatchError::new(
                "unknown".to_string(),
                BatchErrorType::InternalError,
                format!("All items failed: {:?}", errors),
                None,
            ))
        } else {
            Err(BatchError::new(
                "unknown".to_string(),
                BatchErrorType::InternalError,
                format!("Partial success with errors: {:?}", errors),
                None,
            ))
        }
    }
}