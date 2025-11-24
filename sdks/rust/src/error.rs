//! Error types for the Vectorizer SDK

/// Result type alias for the Vectorizer SDK
pub type Result<T> = std::result::Result<T, VectorizerError>;

/// Main error type for the Vectorizer SDK
#[derive(Debug)]
pub enum VectorizerError {
    /// Authentication failed
    Authentication { message: String },

    /// Collection not found
    CollectionNotFound { collection: String },

    /// Vector not found
    VectorNotFound {
        collection: String,
        vector_id: String,
    },

    /// Validation error
    Validation { message: String },

    /// Network error
    Network { message: String },

    /// Server error
    Server { message: String },

    /// Timeout error
    Timeout { timeout_secs: u64 },

    /// Rate limit exceeded
    RateLimit { message: String },

    /// Configuration error
    Configuration { message: String },

    /// Embedding generation error
    Embedding { message: String },

    /// Search error
    Search { message: String },

    /// Storage error
    Storage { message: String },

    /// Batch operation error
    BatchOperation { message: String },

    /// MCP (Model Context Protocol) error
    Mcp { message: String },

    /// Serialization error
    Serialization(String),

    /// HTTP error
    Http(reqwest::Error),

    /// IO error
    Io(std::io::Error),
}

impl VectorizerError {
    /// Create a new authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create a new collection not found error
    pub fn collection_not_found(collection: impl Into<String>) -> Self {
        Self::CollectionNotFound {
            collection: collection.into(),
        }
    }

    /// Create a new vector not found error
    pub fn vector_not_found(collection: impl Into<String>, vector_id: impl Into<String>) -> Self {
        Self::VectorNotFound {
            collection: collection.into(),
            vector_id: vector_id.into(),
        }
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a new network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a new server error
    pub fn server(message: impl Into<String>) -> Self {
        Self::Server {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(timeout_secs: u64) -> Self {
        Self::Timeout { timeout_secs }
    }

    /// Create a new rate limit error
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::RateLimit {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a new embedding error
    pub fn embedding(message: impl Into<String>) -> Self {
        Self::Embedding {
            message: message.into(),
        }
    }

    /// Create a new search error
    pub fn search(message: impl Into<String>) -> Self {
        Self::Search {
            message: message.into(),
        }
    }

    /// Create a new storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }

    /// Create a new batch operation error
    pub fn batch_operation(message: impl Into<String>) -> Self {
        Self::BatchOperation {
            message: message.into(),
        }
    }

    /// Create a new MCP error
    pub fn mcp(message: impl Into<String>) -> Self {
        Self::Mcp {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for VectorizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorizerError::Authentication { message } => {
                write!(f, "Authentication failed: {}", message)
            }
            VectorizerError::CollectionNotFound { collection } => {
                write!(f, "Collection '{}' not found", collection)
            }
            VectorizerError::VectorNotFound {
                collection,
                vector_id,
            } => write!(
                f,
                "Vector '{}' not found in collection '{}'",
                vector_id, collection
            ),
            VectorizerError::Validation { message } => write!(f, "Validation error: {}", message),
            VectorizerError::Network { message } => write!(f, "Network error: {}", message),
            VectorizerError::Server { message } => write!(f, "Server error: {}", message),
            VectorizerError::Timeout { timeout_secs } => {
                write!(f, "Request timeout after {}s", timeout_secs)
            }
            VectorizerError::RateLimit { message } => write!(f, "Rate limit exceeded: {}", message),
            VectorizerError::Configuration { message } => {
                write!(f, "Configuration error: {}", message)
            }
            VectorizerError::Embedding { message } => {
                write!(f, "Embedding generation failed: {}", message)
            }
            VectorizerError::Search { message } => write!(f, "Search failed: {}", message),
            VectorizerError::Storage { message } => write!(f, "Storage error: {}", message),
            VectorizerError::BatchOperation { message } => {
                write!(f, "Batch operation failed: {}", message)
            }
            VectorizerError::Mcp { message } => write!(f, "MCP error: {}", message),
            VectorizerError::Serialization(message) => {
                write!(f, "Serialization error: {}", message)
            }
            VectorizerError::Http(err) => write!(f, "HTTP error: {}", err),
            VectorizerError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for VectorizerError {}

impl From<serde_json::Error> for VectorizerError {
    fn from(err: serde_json::Error) -> Self {
        VectorizerError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for VectorizerError {
    fn from(err: std::io::Error) -> Self {
        VectorizerError::Io(err)
    }
}

impl From<reqwest::Error> for VectorizerError {
    fn from(err: reqwest::Error) -> Self {
        VectorizerError::Http(err)
    }
}

/// Map HTTP status codes to appropriate VectorizerError variants
pub fn map_http_error(status: u16, message: Option<String>) -> VectorizerError {
    let default_message = format!("HTTP {}", status);
    let message = message.unwrap_or(default_message);

    match status {
        401 => VectorizerError::authentication(message),
        403 => VectorizerError::authentication("Access forbidden"),
        404 => VectorizerError::server("Resource not found"),
        429 => VectorizerError::rate_limit(message),
        500..=599 => VectorizerError::server(message),
        _ => VectorizerError::server(message),
    }
}
