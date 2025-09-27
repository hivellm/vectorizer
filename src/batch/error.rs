//! Batch Operations Error Handling
//! 
//! Comprehensive error handling for batch operations including error types,
//! error codes, and result structures.

use std::fmt;
use serde::{Deserialize, Serialize};

/// Batch operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchStatus {
    Success,
    Partial,
    Failed,
}

impl fmt::Display for BatchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchStatus::Success => write!(f, "success"),
            BatchStatus::Partial => write!(f, "partial"),
            BatchStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Batch operation error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchErrorType {
    // Validation errors
    InvalidBatchSize,
    InvalidVectorData,
    InvalidVectorId,
    InvalidCollection,
    InvalidQuery,
    
    // Resource errors
    MemoryLimitExceeded,
    TimeoutExceeded,
    ResourceUnavailable,
    ConcurrentBatchLimitExceeded,
    
    // Processing errors
    EmbeddingGenerationFailed,
    VectorStoreError,
    IndexingError,
    SerializationError,
    
    // System errors
    InternalError,
    DatabaseError,
    NetworkError,
    AuthenticationError,
    
    // Custom error
    Custom(String),
}

impl fmt::Display for BatchErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchErrorType::InvalidBatchSize => write!(f, "invalid_batch_size"),
            BatchErrorType::InvalidVectorData => write!(f, "invalid_vector_data"),
            BatchErrorType::InvalidVectorId => write!(f, "invalid_vector_id"),
            BatchErrorType::InvalidCollection => write!(f, "invalid_collection"),
            BatchErrorType::InvalidQuery => write!(f, "invalid_query"),
            BatchErrorType::MemoryLimitExceeded => write!(f, "memory_limit_exceeded"),
            BatchErrorType::TimeoutExceeded => write!(f, "timeout_exceeded"),
            BatchErrorType::ResourceUnavailable => write!(f, "resource_unavailable"),
            BatchErrorType::ConcurrentBatchLimitExceeded => write!(f, "concurrent_batch_limit_exceeded"),
            BatchErrorType::EmbeddingGenerationFailed => write!(f, "embedding_generation_failed"),
            BatchErrorType::VectorStoreError => write!(f, "vector_store_error"),
            BatchErrorType::IndexingError => write!(f, "indexing_error"),
            BatchErrorType::SerializationError => write!(f, "serialization_error"),
            BatchErrorType::InternalError => write!(f, "internal_error"),
            BatchErrorType::DatabaseError => write!(f, "database_error"),
            BatchErrorType::NetworkError => write!(f, "network_error"),
            BatchErrorType::AuthenticationError => write!(f, "authentication_error"),
            BatchErrorType::Custom(msg) => write!(f, "custom:{}", msg),
        }
    }
}

/// Batch operation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub operation_id: String,
    pub error_type: BatchErrorType,
    pub error_message: String,
    pub vector_id: Option<String>,
    pub retry_count: u32,
    pub timestamp: u64,
}

impl BatchError {
    pub fn new(
        operation_id: String,
        error_type: BatchErrorType,
        error_message: String,
        vector_id: Option<String>,
    ) -> Self {
        Self {
            operation_id,
            error_type,
            error_message,
            vector_id,
            retry_count: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn is_retryable(&self) -> bool {
        match self.error_type {
            BatchErrorType::TimeoutExceeded
            | BatchErrorType::ResourceUnavailable
            | BatchErrorType::NetworkError
            | BatchErrorType::VectorStoreError => true,
            _ => false,
        }
    }

    pub fn should_retry(&self, max_retries: u32) -> bool {
        self.is_retryable() && self.retry_count < max_retries
    }

    pub fn error_code(&self) -> String {
        format!("BATCH_{}", self.error_type)
    }
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BatchError {{ operation_id: {}, error_type: {}, message: {}, vector_id: {:?}, retry_count: {} }}",
            self.operation_id,
            self.error_type,
            self.error_message,
            self.vector_id,
            self.retry_count
        )
    }
}

impl std::error::Error for BatchError {}

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult<T> {
    pub successful_operations: Vec<T>,
    pub failed_operations: Vec<BatchError>,
    pub processing_time_ms: f64,
    pub status: BatchStatus,
    pub total_operations: usize,
    pub successful_count: usize,
    pub failed_count: usize,
}

impl<T> BatchResult<T> {
    pub fn new() -> Self {
        Self {
            successful_operations: Vec::new(),
            failed_operations: Vec::new(),
            processing_time_ms: 0.0,
            status: BatchStatus::Success,
            total_operations: 0,
            successful_count: 0,
            failed_count: 0,
        }
    }

    pub fn with_processing_time(mut self, time_ms: f64) -> Self {
        self.processing_time_ms = time_ms;
        self
    }

    pub fn add_success(&mut self, result: T) {
        self.successful_operations.push(result);
        self.successful_count += 1;
        self.total_operations += 1;
        self.update_status();
    }

    pub fn add_error(&mut self, error: BatchError) {
        self.failed_operations.push(error);
        self.failed_count += 1;
        self.total_operations += 1;
        self.update_status();
    }

    fn update_status(&mut self) {
        self.status = if self.failed_count == 0 {
            BatchStatus::Success
        } else if self.successful_count > 0 {
            BatchStatus::Partial
        } else {
            BatchStatus::Failed
        };
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.successful_count as f64 / self.total_operations as f64) * 100.0
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.failed_operations.is_empty()
    }

    pub fn is_complete_success(&self) -> bool {
        self.status == BatchStatus::Success
    }

    pub fn is_complete_failure(&self) -> bool {
        self.status == BatchStatus::Failed
    }

    pub fn is_partial_success(&self) -> bool {
        self.status == BatchStatus::Partial
    }

    pub fn get_errors_by_type(&self, error_type: &BatchErrorType) -> Vec<&BatchError> {
        self.failed_operations
            .iter()
            .filter(|error| std::mem::discriminant(&error.error_type) == std::mem::discriminant(error_type))
            .collect()
    }

    pub fn get_retryable_errors(&self) -> Vec<&BatchError> {
        self.failed_operations
            .iter()
            .filter(|error| error.is_retryable())
            .collect()
    }
}

impl<T> Default for BatchResult<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub retryable_errors: usize,
    pub processing_time_ms: f64,
    pub throughput_ops_per_second: f64,
    pub average_latency_ms: f64,
    pub error_distribution: std::collections::HashMap<String, usize>,
}

impl BatchStatistics {
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            retryable_errors: 0,
            processing_time_ms: 0.0,
            throughput_ops_per_second: 0.0,
            average_latency_ms: 0.0,
            error_distribution: std::collections::HashMap::new(),
        }
    }

    pub fn from_result<T>(result: &BatchResult<T>) -> Self {
        let mut stats = Self::new();
        
        stats.total_operations = result.total_operations;
        stats.successful_operations = result.successful_count;
        stats.failed_operations = result.failed_count;
        stats.processing_time_ms = result.processing_time_ms;
        
        // Calculate throughput
        if result.processing_time_ms > 0.0 {
            stats.throughput_ops_per_second = (result.total_operations as f64 * 1000.0) / result.processing_time_ms;
            stats.average_latency_ms = result.processing_time_ms / result.total_operations as f64;
        }
        
        // Count retryable errors
        stats.retryable_errors = result.get_retryable_errors().len();
        
        // Build error distribution
        for error in &result.failed_operations {
            let error_code = error.error_code();
            *stats.error_distribution.entry(error_code).or_insert(0) += 1;
        }
        
        stats
    }
}

impl Default for BatchStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common batch errors
pub fn invalid_batch_size_error(operation_id: String, actual_size: usize, max_size: usize) -> BatchError {
    BatchError::new(
        operation_id,
        BatchErrorType::InvalidBatchSize,
        format!("Batch size {} exceeds maximum allowed size {}", actual_size, max_size),
        None,
    )
}

pub fn memory_limit_exceeded_error(operation_id: String, estimated_mb: usize, limit_mb: usize) -> BatchError {
    BatchError::new(
        operation_id,
        BatchErrorType::MemoryLimitExceeded,
        format!("Estimated memory usage {}MB exceeds limit {}MB", estimated_mb, limit_mb),
        None,
    )
}

pub fn vector_store_error(operation_id: String, vector_id: String, message: String) -> BatchError {
    BatchError::new(
        operation_id,
        BatchErrorType::VectorStoreError,
        message,
        Some(vector_id),
    )
}

pub fn embedding_generation_error(operation_id: String, vector_id: Option<String>, message: String) -> BatchError {
    BatchError::new(
        operation_id,
        BatchErrorType::EmbeddingGenerationFailed,
        message,
        vector_id,
    )
}

pub fn timeout_error(operation_id: String, timeout_seconds: u64) -> BatchError {
    BatchError::new(
        operation_id,
        BatchErrorType::TimeoutExceeded,
        format!("Operation timed out after {} seconds", timeout_seconds),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_status_display() {
        assert_eq!(format!("{}", BatchStatus::Success), "success");
        assert_eq!(format!("{}", BatchStatus::Partial), "partial");
        assert_eq!(format!("{}", BatchStatus::Failed), "failed");
    }

    #[test]
    fn test_batch_error_type_display() {
        assert_eq!(format!("{}", BatchErrorType::InvalidBatchSize), "invalid_batch_size");
        assert_eq!(format!("{}", BatchErrorType::MemoryLimitExceeded), "memory_limit_exceeded");
        assert_eq!(format!("{}", BatchErrorType::Custom("test".to_string())), "custom:test");
    }

    #[test]
    fn test_batch_error_creation() {
        let error = BatchError::new(
            "op1".to_string(),
            BatchErrorType::InvalidVectorData,
            "Invalid vector".to_string(),
            Some("vec1".to_string()),
        );
        
        assert_eq!(error.operation_id, "op1");
        assert_eq!(error.vector_id, Some("vec1".to_string()));
        assert_eq!(error.retry_count, 0);
    }

    #[test]
    fn test_batch_error_retryable() {
        let timeout_error = BatchError::new(
            "op1".to_string(),
            BatchErrorType::TimeoutExceeded,
            "Timeout".to_string(),
            None,
        );
        assert!(timeout_error.is_retryable());

        let invalid_error = BatchError::new(
            "op2".to_string(),
            BatchErrorType::InvalidVectorData,
            "Invalid".to_string(),
            None,
        );
        assert!(!invalid_error.is_retryable());
    }

    #[test]
    fn test_batch_result_operations() {
        let mut result: BatchResult<String> = BatchResult::new();
        
        result.add_success("success1".to_string());
        result.add_success("success2".to_string());
        
        let error = BatchError::new(
            "op1".to_string(),
            BatchErrorType::InvalidVectorData,
            "Invalid".to_string(),
            None,
        );
        result.add_error(error);
        
        assert_eq!(result.successful_count, 2);
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.total_operations, 3);
        assert_eq!(result.status, BatchStatus::Partial);
        assert!((result.success_rate() - 66.66666666666667).abs() < 1e-10);
    }

    #[test]
    fn test_batch_result_status() {
        let mut result: BatchResult<String> = BatchResult::new();
        
        // Empty result should be success
        assert_eq!(result.status, BatchStatus::Success);
        
        // Add success - should remain success
        result.add_success("test".to_string());
        assert_eq!(result.status, BatchStatus::Success);
        
        // Add error - should become partial
        let error = BatchError::new(
            "op1".to_string(),
            BatchErrorType::InvalidVectorData,
            "Invalid".to_string(),
            None,
        );
        result.add_error(error);
        assert_eq!(result.status, BatchStatus::Partial);
        assert!(result.is_partial_success());
    }

    #[test]
    fn test_batch_statistics() {
        let mut result: BatchResult<String> = BatchResult::new();
        result.add_success("success".to_string());
        result.add_success("success2".to_string());
        
        let error = BatchError::new(
            "op1".to_string(),
            BatchErrorType::TimeoutExceeded,
            "Timeout".to_string(),
            None,
        );
        result.add_error(error);
        
        let stats = BatchStatistics::from_result(&result);
        assert_eq!(stats.total_operations, 3);
        assert_eq!(stats.successful_operations, 2);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.retryable_errors, 1);
    }

    #[test]
    fn test_helper_functions() {
        let error = invalid_batch_size_error("op1".to_string(), 1500, 1000);
        assert_eq!(error.error_type, BatchErrorType::InvalidBatchSize);
        assert!(error.error_message.contains("1500"));
        assert!(error.error_message.contains("1000"));
        
        let error = memory_limit_exceeded_error("op1".to_string(), 1024, 512);
        assert_eq!(error.error_type, BatchErrorType::MemoryLimitExceeded);
        assert!(error.error_message.contains("1024"));
        assert!(error.error_message.contains("512"));
    }
}
