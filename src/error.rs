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
}

/// Result type alias for Vectorizer operations
pub type Result<T> = std::result::Result<T, VectorizerError>;
