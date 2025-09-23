//! Error types for Vectorizer

use thiserror::Error;

/// Main error type for Vectorizer operations
#[derive(Error, Debug)]
pub enum VectorizerError {
    /// Invalid vector dimension
    #[error("Invalid dimension: expected {expected}, got {got}")]
    InvalidDimension { expected: usize, got: usize },

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

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Result type alias for Vectorizer operations
pub type Result<T> = std::result::Result<T, VectorizerError>;
