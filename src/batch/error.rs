//! Error handling for batch processing
//!
//! This module provides comprehensive error handling for batch operations,
//! including error classification, retry logic, and error reporting.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::batch::{BatchError, BatchErrorType};

/// Comprehensive error for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingError {
    /// Error type
    pub error_type: BatchErrorType,
    /// Error message
    pub message: String,
    /// Item index that caused the error
    pub item_index: Option<usize>,
    /// Batch index that caused the error
    pub batch_index: Option<usize>,
    /// Retry count
    pub retry_count: u32,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional context
    pub context: Option<ErrorContext>,
}

/// Additional context for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Memory usage when error occurred
    pub memory_usage_mb: Option<f64>,
    /// Processing time when error occurred
    pub processing_time_ms: Option<u64>,
    /// System load when error occurred
    pub system_load: Option<f64>,
    /// Custom context data
    pub custom_data: Option<serde_json::Value>,
}

impl fmt::Display for BatchProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BatchProcessingError({:?}): {}",
            self.error_type, self.message
        )?;

        if let Some(item_index) = self.item_index {
            write!(f, " [item: {}]", item_index)?;
        }

        if let Some(batch_index) = self.batch_index {
            write!(f, " [batch: {}]", batch_index)?;
        }

        if self.retry_count > 0 {
            write!(f, " [retries: {}]", self.retry_count)?;
        }

        Ok(())
    }
}

impl std::error::Error for BatchProcessingError {}

impl BatchProcessingError {
    /// Create a new batch processing error
    pub fn new(
        error_type: BatchErrorType,
        message: String,
        item_index: Option<usize>,
        batch_index: Option<usize>,
    ) -> Self {
        Self {
            error_type,
            message,
            item_index,
            batch_index,
            retry_count: 0,
            timestamp: chrono::Utc::now(),
            context: None,
        }
    }

    /// Create with context
    pub fn with_context(
        error_type: BatchErrorType,
        message: String,
        item_index: Option<usize>,
        batch_index: Option<usize>,
        context: ErrorContext,
    ) -> Self {
        Self {
            error_type,
            message,
            item_index,
            batch_index,
            retry_count: 0,
            timestamp: chrono::Utc::now(),
            context: Some(context),
        }
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self.error_type {
            BatchErrorType::Timeout => true,
            BatchErrorType::NetworkError => true,
            BatchErrorType::MemoryError => false,
            BatchErrorType::ValidationError => false,
            BatchErrorType::CollectionNotFound => false,
            BatchErrorType::InsertionFailed => true,
            BatchErrorType::UpdateFailed => true,
            BatchErrorType::DeletionFailed => true,
            BatchErrorType::SearchFailed => true,
            BatchErrorType::Unknown => true,
        }
    }

    /// Get retry delay based on error type and retry count
    pub fn get_retry_delay(&self) -> Duration {
        let base_delay = match self.error_type {
            BatchErrorType::Timeout => Duration::from_secs(1),
            BatchErrorType::NetworkError => Duration::from_secs(2),
            BatchErrorType::MemoryError => Duration::from_secs(30),
            BatchErrorType::ValidationError => Duration::from_secs(0),
            BatchErrorType::CollectionNotFound => Duration::from_secs(0),
            BatchErrorType::InsertionFailed => Duration::from_secs(2),
            BatchErrorType::UpdateFailed => Duration::from_secs(2),
            BatchErrorType::DeletionFailed => Duration::from_secs(2),
            BatchErrorType::SearchFailed => Duration::from_secs(1),
            BatchErrorType::Unknown => Duration::from_secs(5),
        };

        // Exponential backoff
        let multiplier = 2_u32.pow(self.retry_count.min(5));
        Duration::from_millis(base_delay.as_millis() as u64 * multiplier as u64)
    }
}

/// Error aggregator for batch processing
pub struct ErrorAggregator {
    errors: Vec<BatchProcessingError>,
    max_errors: usize,
    error_counts: std::collections::HashMap<BatchErrorType, usize>,
}

impl ErrorAggregator {
    /// Create a new error aggregator
    pub fn new(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
            error_counts: std::collections::HashMap::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: BatchProcessingError) {
        if self.errors.len() < self.max_errors {
            self.errors.push(error.clone());
        }

        *self.error_counts.entry(error.error_type).or_insert(0) += 1;
    }

    /// Get all errors
    pub fn get_errors(&self) -> &[BatchProcessingError] {
        &self.errors
    }

    /// Get error counts by type
    pub fn get_error_counts(&self) -> &std::collections::HashMap<BatchErrorType, usize> {
        &self.error_counts
    }

    /// Get total error count
    pub fn total_errors(&self) -> usize {
        self.error_counts.values().sum()
    }

    /// Get most common error type
    pub fn most_common_error_type(&self) -> Option<BatchErrorType> {
        self.error_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error_type, _)| error_type.clone())
    }

    /// Check if error limit exceeded
    pub fn is_error_limit_exceeded(&self) -> bool {
        self.errors.len() >= self.max_errors
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
        self.error_counts.clear();
    }
}

/// Error recovery strategies
pub enum ErrorRecoveryStrategy {
    /// Retry with exponential backoff
    ExponentialBackoff {
        max_retries: u32,
        base_delay: Duration,
    },
    /// Retry with fixed delay
    FixedDelay { max_retries: u32, delay: Duration },
    /// Skip failed items
    SkipFailed,
    /// Stop processing on first error
    StopOnError,
    /// Custom recovery strategy
    Custom(Box<dyn Fn(&BatchProcessingError) -> RecoveryAction + Send + Sync>),
}

/// Recovery action to take
pub enum RecoveryAction {
    /// Retry the operation
    Retry,
    /// Skip the item
    Skip,
    /// Stop processing
    Stop,
    /// Wait before retrying
    Wait(Duration),
}

impl Default for ErrorRecoveryStrategy {
    fn default() -> Self {
        Self::ExponentialBackoff {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
        }
    }
}

/// Error recovery manager
pub struct ErrorRecoveryManager {
    strategy: ErrorRecoveryStrategy,
    error_aggregator: ErrorAggregator,
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new(strategy: ErrorRecoveryStrategy, max_errors: usize) -> Self {
        Self {
            strategy,
            error_aggregator: ErrorAggregator::new(max_errors),
        }
    }

    /// Handle an error and determine recovery action
    pub fn handle_error(&mut self, error: BatchProcessingError) -> RecoveryAction {
        self.error_aggregator.add_error(error.clone());

        match &self.strategy {
            ErrorRecoveryStrategy::ExponentialBackoff {
                max_retries,
                base_delay,
            } => {
                if error.retry_count >= *max_retries {
                    RecoveryAction::Skip
                } else if error.is_retryable() {
                    let delay = Duration::from_millis(
                        base_delay.as_millis() as u64 * 2_u64.pow(error.retry_count.min(5)),
                    );
                    RecoveryAction::Wait(delay)
                } else {
                    RecoveryAction::Skip
                }
            }
            ErrorRecoveryStrategy::FixedDelay { max_retries, delay } => {
                if error.retry_count >= *max_retries {
                    RecoveryAction::Skip
                } else if error.is_retryable() {
                    RecoveryAction::Wait(*delay)
                } else {
                    RecoveryAction::Skip
                }
            }
            ErrorRecoveryStrategy::SkipFailed => RecoveryAction::Skip,
            ErrorRecoveryStrategy::StopOnError => RecoveryAction::Stop,
            ErrorRecoveryStrategy::Custom(func) => func(&error),
        }
    }

    /// Get error statistics
    pub fn get_error_stats(&self) -> ErrorStats {
        ErrorStats {
            total_errors: self.error_aggregator.total_errors(),
            error_counts: self.error_aggregator.get_error_counts().clone(),
            most_common_error: self.error_aggregator.most_common_error_type(),
            is_limit_exceeded: self.error_aggregator.is_error_limit_exceeded(),
        }
    }

    /// Clear error history
    pub fn clear_errors(&mut self) {
        self.error_aggregator.clear();
    }
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Total number of errors
    pub total_errors: usize,
    /// Error counts by type
    pub error_counts: std::collections::HashMap<BatchErrorType, usize>,
    /// Most common error type
    pub most_common_error: Option<BatchErrorType>,
    /// Whether error limit was exceeded
    pub is_limit_exceeded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processing_error() {
        let error = BatchProcessingError::new(
            BatchErrorType::Timeout,
            "Operation timed out".to_string(),
            Some(42),
            Some(1),
        );

        assert_eq!(error.error_type, BatchErrorType::Timeout);
        assert_eq!(error.item_index, Some(42));
        assert_eq!(error.batch_index, Some(1));
        assert!(error.is_retryable());
        assert_eq!(error.retry_count, 0);
    }

    #[test]
    fn test_error_aggregator() {
        let mut aggregator = ErrorAggregator::new(10);

        let error1 = BatchProcessingError::new(
            BatchErrorType::Timeout,
            "Timeout 1".to_string(),
            Some(1),
            Some(0),
        );

        let error2 = BatchProcessingError::new(
            BatchErrorType::Timeout,
            "Timeout 2".to_string(),
            Some(2),
            Some(0),
        );

        aggregator.add_error(error1);
        aggregator.add_error(error2);

        assert_eq!(aggregator.total_errors(), 2);
        assert_eq!(
            aggregator.get_error_counts().get(&BatchErrorType::Timeout),
            Some(&2)
        );
        assert_eq!(
            aggregator.most_common_error_type(),
            Some(BatchErrorType::Timeout)
        );
    }

    #[test]
    fn test_error_recovery_manager() {
        let strategy = ErrorRecoveryStrategy::ExponentialBackoff {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
        };

        let mut manager = ErrorRecoveryManager::new(strategy, 100);

        let error = BatchProcessingError::new(
            BatchErrorType::Timeout,
            "Timeout".to_string(),
            Some(1),
            Some(0),
        );

        let action = manager.handle_error(error);
        match action {
            RecoveryAction::Wait(delay) => {
                assert!(delay >= Duration::from_secs(1));
            }
            _ => panic!("Expected Wait action"),
        }
    }

    #[test]
    fn test_retry_delay_calculation() {
        let mut error = BatchProcessingError::new(
            BatchErrorType::Timeout,
            "Timeout".to_string(),
            Some(1),
            Some(0),
        );

        let delay1 = error.get_retry_delay();
        error.increment_retry();
        let delay2 = error.get_retry_delay();

        assert!(delay2 > delay1);
    }
}
