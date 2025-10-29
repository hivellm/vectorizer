//! Error types for Vectorizer

use thiserror::Error;

/// Main error type for Vectorizer operations
#[derive(Error, Debug)]
pub enum VectorizerError {
    /// Invalid vector dimension
    #[error("Invalid dimension: expected {expected}, got {got}")]
    InvalidDimension { expected: usize, got: usize },

    /// Dimension mismatch
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    /// Collection not found
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    /// Collection already exists
    #[error("Collection already exists: {0}")]
    CollectionAlreadyExists(String),

    /// Vector not found
    #[error("Vector not found: {0}")]
    VectorNotFound(String),

    /// Persistence error
    #[error("Persistence error: {0}")]
    PersistenceError(String),

    /// Index error
    #[error("Index error: {0}")]
    IndexError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[cfg(feature = "candle-models")]
    /// Candle ML framework error
    #[error("Candle error: {0}")]
    CandleError(#[from] candle_core::Error),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Authorization error (insufficient permissions)
    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit_type} limit of {limit}")]
    RateLimitExceeded { limit_type: String, limit: u32 },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    /// Embedding error
    #[error("Embedding error: {0}")]
    Embedding(#[from] crate::embedding::EmbeddingError),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Other errors
    #[error("{0}")]
    Other(String),

    /// UMICP protocol error
    #[error("UMICP error: {0}")]
    UmicpError(#[from] umicp_core::error::UmicpError),

    /// Transmutation document conversion error
    #[error("Transmutation error: {0}")]
    TransmutationError(String),

    /// Storage error (for .vecdb operations)
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error (alias)
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Serialization error (string-based)
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error (string-based)
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// IO error (alias for convenience)
    #[error("IO error: {0}")]
    Io(std::io::Error),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout error
    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Resource exhausted error
    #[error("Resource exhausted: {resource} - {message}")]
    ResourceExhausted { resource: String, message: String },

    /// Concurrency error
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    /// Batch processing error
    #[error("Batch processing error: {0}")]
    BatchProcessingError(String),

    /// Quantization error
    #[error("Quantization error: {0}")]
    QuantizationError(String),

    /// Embedding error
    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    /// Search error
    #[error("Search error: {0}")]
    SearchError(String),

    /// API error
    #[error("API error: {0}")]
    ApiError(String),

    /// ML error
    #[error("ML error: {0}")]
    MlError(String),

    /// Processing error
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

/// Result type alias for Vectorizer operations
pub type Result<T> = std::result::Result<T, VectorizerError>;

/// Error context helpers for better error reporting
impl VectorizerError {
    /// Add context to an error
    pub fn with_context(self, context: &str) -> Self {
        match self {
            VectorizerError::Other(msg) => VectorizerError::Other(format!("{}: {}", context, msg)),
            VectorizerError::InternalError(msg) => {
                VectorizerError::InternalError(format!("{}: {}", context, msg))
            }
            VectorizerError::PersistenceError(msg) => {
                VectorizerError::PersistenceError(format!("{}: {}", context, msg))
            }
            VectorizerError::IndexError(msg) => {
                VectorizerError::IndexError(format!("{}: {}", context, msg))
            }
            VectorizerError::ConfigurationError(msg) => {
                VectorizerError::ConfigurationError(format!("{}: {}", context, msg))
            }
            VectorizerError::AuthenticationError(msg) => {
                VectorizerError::AuthenticationError(format!("{}: {}", context, msg))
            }
            VectorizerError::AuthorizationError(msg) => {
                VectorizerError::AuthorizationError(format!("{}: {}", context, msg))
            }
            VectorizerError::NetworkError(msg) => {
                VectorizerError::NetworkError(format!("{}: {}", context, msg))
            }
            VectorizerError::ConcurrencyError(msg) => {
                VectorizerError::ConcurrencyError(format!("{}: {}", context, msg))
            }
            VectorizerError::BatchProcessingError(msg) => {
                VectorizerError::BatchProcessingError(format!("{}: {}", context, msg))
            }
            VectorizerError::QuantizationError(msg) => {
                VectorizerError::QuantizationError(format!("{}: {}", context, msg))
            }
            VectorizerError::EmbeddingError(msg) => {
                VectorizerError::EmbeddingError(format!("{}: {}", context, msg))
            }
            VectorizerError::SearchError(msg) => {
                VectorizerError::SearchError(format!("{}: {}", context, msg))
            }
            other => VectorizerError::Other(format!("{}: {}", context, other)),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            VectorizerError::NetworkError(_)
                | VectorizerError::TimeoutError { .. }
                | VectorizerError::ConcurrencyError(_)
                | VectorizerError::IoError(_)
                | VectorizerError::Io(_)
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            VectorizerError::InvalidDimension { .. }
            | VectorizerError::DimensionMismatch { .. }
            | VectorizerError::ValidationError { .. } => ErrorSeverity::Warning,

            VectorizerError::CollectionNotFound(_)
            | VectorizerError::VectorNotFound(_)
            | VectorizerError::NotFound(_) => ErrorSeverity::Info,

            VectorizerError::AuthenticationError(_)
            | VectorizerError::AuthorizationError(_)
            | VectorizerError::RateLimitExceeded { .. } => ErrorSeverity::Warning,

            VectorizerError::InternalError(_)
            | VectorizerError::ResourceExhausted { .. }
            | VectorizerError::ConcurrencyError(_) => ErrorSeverity::Error,

            _ => ErrorSeverity::Info,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_with_context() {
        let error = VectorizerError::Other("test error".to_string());
        let contextual_error = error.with_context("operation failed");

        match contextual_error {
            VectorizerError::Other(msg) => {
                assert!(msg.contains("operation failed"));
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected Other error variant"),
        }
    }

    #[test]
    fn test_error_retryable() {
        let retryable_errors = vec![
            VectorizerError::NetworkError("connection failed".to_string()),
            VectorizerError::TimeoutError { timeout_ms: 5000 },
            VectorizerError::ConcurrencyError("lock contention".to_string()),
            VectorizerError::IoError(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
        ];

        for error in retryable_errors {
            assert!(
                error.is_retryable(),
                "Error should be retryable: {:?}",
                error
            );
        }

        let non_retryable_errors = vec![
            VectorizerError::InvalidDimension {
                expected: 512,
                got: 256,
            },
            VectorizerError::CollectionNotFound("test".to_string()),
            VectorizerError::AuthenticationError("invalid token".to_string()),
        ];

        for error in non_retryable_errors {
            assert!(
                !error.is_retryable(),
                "Error should not be retryable: {:?}",
                error
            );
        }
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(
            VectorizerError::InvalidDimension {
                expected: 512,
                got: 256
            }
            .severity(),
            ErrorSeverity::Warning
        );

        assert_eq!(
            VectorizerError::CollectionNotFound("test".to_string()).severity(),
            ErrorSeverity::Info
        );

        assert_eq!(
            VectorizerError::InternalError("system failure".to_string()).severity(),
            ErrorSeverity::Error
        );

        assert_eq!(
            VectorizerError::ResourceExhausted {
                resource: "memory".to_string(),
                message: "out of memory".to_string()
            }
            .severity(),
            ErrorSeverity::Error
        );
    }

    #[test]
    fn test_error_severity_display() {
        assert_eq!(format!("{}", ErrorSeverity::Info), "INFO");
        assert_eq!(format!("{}", ErrorSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", ErrorSeverity::Error), "ERROR");
        assert_eq!(format!("{}", ErrorSeverity::Critical), "CRITICAL");
    }

    #[test]
    fn test_new_error_types() {
        // Test validation error
        let validation_error = VectorizerError::ValidationError {
            field: "dimension".to_string(),
            message: "must be positive".to_string(),
        };
        assert!(validation_error.to_string().contains("dimension"));
        assert!(validation_error.to_string().contains("must be positive"));

        // Test timeout error
        let timeout_error = VectorizerError::TimeoutError { timeout_ms: 5000 };
        assert!(timeout_error.to_string().contains("5000"));

        // Test resource exhausted error
        let resource_error = VectorizerError::ResourceExhausted {
            resource: "memory".to_string(),
            message: "insufficient memory".to_string(),
        };
        assert!(resource_error.to_string().contains("memory"));
        assert!(resource_error.to_string().contains("insufficient memory"));
    }
}
