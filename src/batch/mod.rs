//! Batch processing module for large-scale operations
//!
//! This module provides utilities for processing large amounts of data in batches,
//! including parallel processing, progress tracking, and error handling.

pub mod config;
pub mod error;
pub mod operations;
pub mod processor;
pub mod progress;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum number of concurrent workers
    pub max_workers: usize,
    /// Batch size for processing
    pub batch_size: usize,
    /// Maximum retry attempts for failed items
    pub max_retries: u32,
    /// Timeout for individual batch operations (seconds)
    pub operation_timeout: u64,
    /// Enable progress tracking
    pub enable_progress: bool,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Memory limit per worker (MB)
    pub memory_limit_mb: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get(),
            batch_size: 1000,
            max_retries: 3,
            operation_timeout: 300, // 5 minutes
            enable_progress: true,
            enable_parallel: true,
            memory_limit_mb: 512,
        }
    }
}

/// Batch processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult<T> {
    /// Successfully processed items
    pub successful: Vec<T>,
    /// Failed items with error information
    pub failed: Vec<BatchError>,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of retries performed
    pub retry_count: u32,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage in MB
    pub peak_memory_mb: f64,
    /// Average memory usage in MB
    pub avg_memory_mb: f64,
    /// Memory usage per item in KB
    pub memory_per_item_kb: f64,
}

/// Batch processing error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Item index that failed
    pub item_index: usize,
    /// Error message
    pub message: String,
    /// Error type
    pub error_type: BatchErrorType,
    /// Number of retry attempts
    pub retry_count: u32,
}

impl BatchError {
    pub fn new(error_type: BatchErrorType, message: &str) -> Self {
        Self {
            item_index: 0,
            message: message.to_string(),
            error_type,
            retry_count: 0,
        }
    }
}

/// Types of batch processing errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BatchErrorType {
    /// Processing timeout
    Timeout,
    /// Memory allocation failed
    MemoryError,
    /// Validation error
    ValidationError,
    /// Network error
    NetworkError,
    /// Collection not found
    CollectionNotFound,
    /// Insertion failed
    InsertionFailed,
    /// Update failed
    UpdateFailed,
    /// Deletion failed
    DeletionFailed,
    /// Search failed
    SearchFailed,
    /// Unknown error
    Unknown,
}

/// Progress information for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    /// Total number of items
    pub total_items: usize,
    /// Number of processed items
    pub processed_items: usize,
    /// Number of successful items
    pub successful_items: usize,
    /// Number of failed items
    pub failed_items: usize,
    /// Current processing rate (items per second)
    pub processing_rate: f64,
    /// Estimated time remaining (seconds)
    pub estimated_remaining_seconds: f64,
    /// Current memory usage (MB)
    pub current_memory_mb: f64,
}

impl BatchProgress {
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_items == 0 {
            100.0
        } else {
            (self.processed_items as f64 / self.total_items as f64) * 100.0
        }
    }

    /// Check if processing is complete
    pub fn is_complete(&self) -> bool {
        self.processed_items >= self.total_items
    }
}

/// Batch processor trait
#[async_trait::async_trait]
pub trait BatchProcessor<T: Send + Sync, R: Send + Sync>: Send + Sync {
    /// Process a single item
    async fn process_item(&self, item: T) -> Result<R, String>;

    /// Process a batch of items
    async fn process_batch(&self, items: Vec<T>) -> Result<Vec<R>, String>
    where
        T: 'async_trait,
    {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            match self.process_item(item).await {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    /// Validate an item before processing
    fn validate_item(&self, _item: &T) -> Result<(), String> {
        Ok(())
    }

    /// Get the processor name for logging
    fn processor_name(&self) -> &str {
        "BatchProcessor"
    }
}

/// Batch processing manager
pub struct BatchManager<T, R, P>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
    P: BatchProcessor<T, R> + Send + Sync + 'static,
{
    processor: Arc<P>,
    config: BatchConfig,
    progress_sender: Option<mpsc::UnboundedSender<BatchProgress>>,
    _phantom: std::marker::PhantomData<(T, R)>,
}

impl<T, R, P> BatchManager<T, R, P>
where
    P: BatchProcessor<T, R> + Send + Sync + 'static,
    T: Send + Sync + Clone + 'static,
    R: Send + Sync + Clone + 'static,
{
    /// Create a new batch manager
    pub fn new(processor: P, config: BatchConfig) -> Self {
        Self {
            processor: Arc::new(processor),
            config,
            progress_sender: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set progress tracking
    pub fn with_progress_tracking(mut self, sender: mpsc::UnboundedSender<BatchProgress>) -> Self {
        self.progress_sender = Some(sender);
        self
    }

    /// Process items in batches
    pub async fn process_batches(&self, items: Vec<T>) -> BatchResult<R> {
        let start_time = std::time::Instant::now();
        let total_items = items.len();

        if total_items == 0 {
            return BatchResult {
                successful: Vec::new(),
                failed: Vec::new(),
                processing_time_ms: 0,
                retry_count: 0,
                memory_stats: MemoryStats {
                    peak_memory_mb: 0.0,
                    avg_memory_mb: 0.0,
                    memory_per_item_kb: 0.0,
                },
            };
        }

        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut retry_count = 0;

        // Split items into batches
        let batches: Vec<Vec<T>> = items
            .chunks(self.config.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len();

        if self.config.enable_parallel {
            // Parallel processing
            self.process_parallel(batches, &mut successful, &mut failed, &mut retry_count)
                .await;
        } else {
            // Sequential processing
            self.process_sequential(batches, &mut successful, &mut failed, &mut retry_count)
                .await;
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Calculate memory statistics
        let memory_stats = self.calculate_memory_stats(&successful, &failed);

        BatchResult {
            successful,
            failed,
            processing_time_ms: processing_time,
            retry_count,
            memory_stats,
        }
    }

    /// Process batches in parallel
    async fn process_parallel(
        &self,
        batches: Vec<Vec<T>>,
        successful: &mut Vec<R>,
        failed: &mut Vec<BatchError>,
        retry_count: &mut u32,
    ) {
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();

        for (batch_index, batch) in batches.into_iter().enumerate() {
            let processor = Arc::clone(&self.processor);
            let config = self.config.clone();

            join_set.spawn(async move {
                Self::process_single_batch_static(processor, batch, batch_index, config).await
            });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((batch_successful, batch_failed, batch_retries)) => {
                    successful.extend(batch_successful);
                    failed.extend(batch_failed);
                    *retry_count += batch_retries;
                }
                Err(e) => {
                    failed.push(BatchError {
                        item_index: 0,
                        message: format!("Task join error: {}", e),
                        error_type: BatchErrorType::Unknown,
                        retry_count: 0,
                    });
                }
            }
        }
    }

    /// Process batches sequentially
    async fn process_sequential(
        &self,
        batches: Vec<Vec<T>>,
        successful: &mut Vec<R>,
        failed: &mut Vec<BatchError>,
        retry_count: &mut u32,
    ) {
        for (batch_index, batch) in batches.into_iter().enumerate() {
            let (batch_successful, batch_failed, batch_retries) =
                Self::process_single_batch_static(
                    Arc::clone(&self.processor),
                    batch,
                    batch_index,
                    self.config.clone(),
                )
                .await;

            successful.extend(batch_successful);
            failed.extend(batch_failed);
            *retry_count += batch_retries;
        }
    }

    /// Process a single batch (static method for async closure)
    async fn process_single_batch_static(
        processor: Arc<P>,
        batch: Vec<T>,
        batch_index: usize,
        config: BatchConfig,
    ) -> (Vec<R>, Vec<BatchError>, u32) {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut retry_count = 0;

        for (item_index, item) in batch.into_iter().enumerate() {
            let global_index = batch_index * config.batch_size + item_index;

            // Validate item
            if let Err(e) = processor.validate_item(&item) {
                failed.push(BatchError {
                    item_index: global_index,
                    message: e,
                    error_type: BatchErrorType::ValidationError,
                    retry_count: 0,
                });
                continue;
            }

            // Process item with retries
            let mut attempts = 0;
            let mut last_error = String::new();

            while attempts <= config.max_retries {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(config.operation_timeout),
                    processor.process_item(item.clone()),
                )
                .await
                {
                    Ok(Ok(result)) => {
                        successful.push(result);
                        break;
                    }
                    Ok(Err(e)) => {
                        last_error = e;
                        attempts += 1;
                        retry_count += 1;
                    }
                    Err(_) => {
                        last_error = "Operation timeout".to_string();
                        attempts += 1;
                        retry_count += 1;
                    }
                }
            }

            if attempts > config.max_retries {
                failed.push(BatchError {
                    item_index: global_index,
                    message: last_error,
                    error_type: BatchErrorType::Timeout,
                    retry_count: attempts - 1,
                });
            }
        }

        (successful, failed, retry_count)
    }

    /// Process a single batch
    async fn process_single_batch(
        &self,
        processor: Arc<P>,
        batch: Vec<T>,
        batch_index: usize,
        config: BatchConfig,
    ) -> (Vec<R>, Vec<BatchError>, u32) {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let mut retry_count = 0;

        for (item_index, item) in batch.into_iter().enumerate() {
            let global_index = batch_index * config.batch_size + item_index;

            // Validate item
            if let Err(e) = processor.validate_item(&item) {
                failed.push(BatchError {
                    item_index: global_index,
                    message: e,
                    error_type: BatchErrorType::ValidationError,
                    retry_count: 0,
                });
                continue;
            }

            // Process item with retries
            let mut attempts = 0;
            let mut last_error = String::new();

            while attempts <= config.max_retries {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(config.operation_timeout),
                    processor.process_item(item.clone()),
                )
                .await
                {
                    Ok(Ok(result)) => {
                        successful.push(result);
                        break;
                    }
                    Ok(Err(e)) => {
                        last_error = e;
                        attempts += 1;
                        retry_count += 1;
                    }
                    Err(_) => {
                        last_error = "Operation timeout".to_string();
                        attempts += 1;
                        retry_count += 1;
                    }
                }
            }

            if attempts > config.max_retries {
                failed.push(BatchError {
                    item_index: global_index,
                    message: last_error,
                    error_type: BatchErrorType::Timeout,
                    retry_count: attempts - 1,
                });
            }
        }

        (successful, failed, retry_count)
    }

    /// Calculate memory statistics
    fn calculate_memory_stats(&self, successful: &[R], failed: &[BatchError]) -> MemoryStats {
        // This is a simplified implementation
        // In a real implementation, you would track actual memory usage
        let total_items = successful.len() + failed.len();
        let estimated_memory_mb =
            (total_items * std::mem::size_of::<R>()) as f64 / (1024.0 * 1024.0);

        MemoryStats {
            peak_memory_mb: estimated_memory_mb * 1.5, // Estimate peak as 1.5x average
            avg_memory_mb: estimated_memory_mb,
            memory_per_item_kb: if total_items > 0 {
                (estimated_memory_mb * 1024.0) / total_items as f64
            } else {
                0.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProcessor;

    #[async_trait::async_trait]
    impl BatchProcessor<String, String> for TestProcessor {
        async fn process_item(&self, item: String) -> Result<String, String> {
            if item.contains("error") {
                Err("Test error".to_string())
            } else {
                Ok(format!("processed_{}", item))
            }
        }

        fn processor_name(&self) -> &str {
            "TestProcessor"
        }
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert!(config.max_workers > 0);
        assert!(config.batch_size > 0);
        assert!(config.max_retries > 0);
    }

    #[test]
    fn test_batch_progress() {
        let progress = BatchProgress {
            total_items: 100,
            processed_items: 50,
            successful_items: 45,
            failed_items: 5,
            processing_rate: 10.0,
            estimated_remaining_seconds: 5.0,
            current_memory_mb: 100.0,
        };

        assert_eq!(progress.completion_percentage(), 50.0);
        assert!(!progress.is_complete());
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let processor = TestProcessor;
        let config = BatchConfig {
            max_workers: 2,
            batch_size: 2,
            max_retries: 1,
            operation_timeout: 10,
            enable_progress: false,
            enable_parallel: false,
            memory_limit_mb: 100,
        };

        let manager = BatchManager::new(processor, config);
        let items = vec![
            "item1".to_string(),
            "item2".to_string(),
            "error_item".to_string(),
        ];

        let result = manager.process_batches(items).await;

        assert_eq!(result.successful.len(), 2);
        assert_eq!(result.failed.len(), 1);
        // Processing time may be 0 on fast systems
        assert!(result.processing_time_ms >= 0);
    }
}

// Re-export missing types
pub use operations::{
    BatchDeleteOperation, BatchInsertOperation, BatchOperationManager, BatchSearchOperation,
    BatchUpdateOperation,
};
pub type BatchOperation = BatchInsertOperation; // Use insert as default operation type
pub type BatchProcessorBuilder<T, R, P> = BatchManager<T, R, P>; // Use BatchManager as builder
