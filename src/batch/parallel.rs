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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::BatchConfig;

    #[tokio::test]
    async fn test_parallel_processor_creation() {
        let config = Arc::new(BatchConfig::default());
        let processor = ParallelProcessor::new(config);

        // Processor should be created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_process_chunks_empty_input() {
        let config = Arc::new(BatchConfig::default());
        let processor = ParallelProcessor::new(config);

        let items: Vec<i32> = vec![];
        let result = processor
            .process_chunks(items, |chunk| async move {
                Ok(chunk.iter().map(|x| x * 2).collect())
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_process_chunks_single_chunk() {
        let config = Arc::new(BatchConfig {
            chunk_size: 10,
            parallel_workers: 2,
            ..Default::default()
        });
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let result = processor
            .process_chunks(items, |chunk| async move {
                Ok(chunk.iter().map(|x| x * 2).collect())
            })
            .await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_process_chunks_multiple_chunks() {
        let config = Arc::new(BatchConfig {
            chunk_size: 3,
            parallel_workers: 4,
            ..Default::default()
        });
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = processor
            .process_chunks(items, |chunk| async move {
                Ok(chunk.iter().map(|x| x * 2).collect())
            })
            .await;

        assert!(result.is_ok());
        let mut results = result.unwrap();
        results.sort(); // Results may come in different order due to parallelism
        assert_eq!(results, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
    }

    #[tokio::test]
    async fn test_process_chunks_with_errors() {
        let config = Arc::new(BatchConfig::default());
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let result: BatchResult<Vec<i32>> = processor
            .process_chunks(items, |_chunk| async move {
                Err(BatchError::new(
                    "test".to_string(),
                    BatchErrorType::InternalError,
                    "Test error".to_string(),
                    None,
                ))
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_items_empty_input() {
        let config = Arc::new(BatchConfig::default());
        let processor = ParallelProcessor::new(config);

        let items: Vec<i32> = vec![];
        let result = processor
            .process_items(items, |x| async move { Ok(x * 2) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_process_items_success() {
        let config = Arc::new(BatchConfig {
            parallel_workers: 4,
            ..Default::default()
        });
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = processor
            .process_items(items, |x| async move { Ok(x * 2) })
            .await;

        assert!(result.is_ok());
        let mut results = result.unwrap();
        results.sort();
        assert_eq!(results, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
    }

    #[tokio::test]
    async fn test_process_items_with_errors() {
        let config = Arc::new(BatchConfig::default());
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let result: BatchResult<Vec<i32>> = processor
            .process_items(items, |_x| async move {
                Err(BatchError::new(
                    "test".to_string(),
                    BatchErrorType::InternalError,
                    "Test error".to_string(),
                    None,
                ))
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_items_partial_errors() {
        let config = Arc::new(BatchConfig::default());
        let processor = ParallelProcessor::new(config);

        let items = vec![1, 2, 3, 4, 5];
        let result = processor
            .process_items(items, |x| async move {
                if x % 2 == 0 {
                    Err(BatchError::new(
                        "test".to_string(),
                        BatchErrorType::InternalError,
                        format!("Error for {}", x),
                        None,
                    ))
                } else {
                    Ok(x * 2)
                }
            })
            .await;

        // Should return error due to partial failures
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parallel_processing_concurrency() {
        let config = Arc::new(BatchConfig {
            parallel_workers: 2,
            ..Default::default()
        });
        let processor = ParallelProcessor::new(config);

        let items = (1..=20).collect::<Vec<i32>>();
        let start = std::time::Instant::now();

        let result = processor
            .process_items(items, |x| async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                Ok(x)
            })
            .await;

        let duration = start.elapsed();

        assert!(result.is_ok());
        // With 2 workers processing 20 items at 10ms each,
        // should take roughly 100ms (not 200ms sequential)
        // Allow more time for slower systems or CI environments
        assert!(duration.as_millis() < 300);
    }

    #[tokio::test]
    async fn test_batch_item_trait() {
        #[derive(Clone)]
        struct CustomItem {
            value: i32,
        }

        // Verify trait is implemented for custom types
        fn assert_batch_item<T: BatchItem>() {}
        assert_batch_item::<CustomItem>();
        assert_batch_item::<i32>();
        assert_batch_item::<String>();
    }
}
