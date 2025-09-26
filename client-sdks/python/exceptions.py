"""
Exception classes for the Hive Vectorizer SDK.

This module contains all custom exceptions used throughout the SDK
for proper error handling and debugging.
"""


class VectorizerError(Exception):
    """
    Base exception class for all Vectorizer-related errors.
    
    This is the parent class for all custom exceptions in the SDK.
    """
    
    def __init__(self, message: str, error_code: str = None, details: dict = None):
        """
        Initialize the exception.
        
        Args:
            message: Error message
            error_code: Optional error code
            details: Optional additional error details
        """
        super().__init__(message)
        self.message = message
        self.error_code = error_code
        self.details = details or {}
        
    def __str__(self):
        """String representation of the exception."""
        if self.error_code:
            return f"[{self.error_code}] {self.message}"
        return self.message


class AuthenticationError(VectorizerError):
    """
    Raised when authentication fails or API key is invalid.
    
    This exception is raised when:
    - API key is missing or invalid
    - Token has expired
    - Authentication credentials are incorrect
    """
    
    def __init__(self, message: str = "Authentication failed", **kwargs):
        super().__init__(message, error_code="AUTH_ERROR", **kwargs)


class CollectionNotFoundError(VectorizerError):
    """
    Raised when a requested collection does not exist.
    
    This exception is raised when:
    - Collection name is not found
    - Collection has been deleted
    - Collection name is misspelled
    """
    
    def __init__(self, message: str = "Collection not found", **kwargs):
        super().__init__(message, error_code="COLLECTION_NOT_FOUND", **kwargs)


class ValidationError(VectorizerError):
    """
    Raised when input validation fails.
    
    This exception is raised when:
    - Required parameters are missing
    - Parameter values are invalid
    - Data format is incorrect
    """
    
    def __init__(self, message: str = "Validation failed", **kwargs):
        super().__init__(message, error_code="VALIDATION_ERROR", **kwargs)


class NetworkError(VectorizerError):
    """
    Raised when network-related errors occur.
    
    This exception is raised when:
    - Connection timeout
    - Network unreachable
    - DNS resolution fails
    - SSL/TLS errors
    """
    
    def __init__(self, message: str = "Network error occurred", **kwargs):
        super().__init__(message, error_code="NETWORK_ERROR", **kwargs)


class ServerError(VectorizerError):
    """
    Raised when server returns an error response.
    
    This exception is raised when:
    - HTTP 5xx status codes
    - Server is overloaded
    - Internal server errors
    """
    
    def __init__(self, message: str = "Server error occurred", **kwargs):
        super().__init__(message, error_code="SERVER_ERROR", **kwargs)


class RateLimitError(VectorizerError):
    """
    Raised when rate limit is exceeded.
    
    This exception is raised when:
    - Too many requests per minute/hour
    - API quota exceeded
    - Rate limit headers indicate throttling
    """
    
    def __init__(self, message: str = "Rate limit exceeded", **kwargs):
        super().__init__(message, error_code="RATE_LIMIT_ERROR", **kwargs)


class TimeoutError(VectorizerError):
    """
    Raised when operations timeout.
    
    This exception is raised when:
    - Request timeout
    - Operation takes too long
    - Connection timeout
    """
    
    def __init__(self, message: str = "Operation timed out", **kwargs):
        super().__init__(message, error_code="TIMEOUT_ERROR", **kwargs)


class VectorNotFoundError(VectorizerError):
    """
    Raised when a requested vector does not exist.
    
    This exception is raised when:
    - Vector ID is not found
    - Vector has been deleted
    - Vector ID is invalid
    """
    
    def __init__(self, message: str = "Vector not found", **kwargs):
        super().__init__(message, error_code="VECTOR_NOT_FOUND", **kwargs)


class EmbeddingError(VectorizerError):
    """
    Raised when embedding generation fails.
    
    This exception is raised when:
    - Text is too long for embedding
    - Embedding model fails
    - Invalid text format
    """
    
    def __init__(self, message: str = "Embedding generation failed", **kwargs):
        super().__init__(message, error_code="EMBEDDING_ERROR", **kwargs)


class IndexingError(VectorizerError):
    """
    Raised when indexing operations fail.
    
    This exception is raised when:
    - Index creation fails
    - Index corruption
    - Indexing process errors
    """
    
    def __init__(self, message: str = "Indexing operation failed", **kwargs):
        super().__init__(message, error_code="INDEXING_ERROR", **kwargs)


class ConfigurationError(VectorizerError):
    """
    Raised when configuration is invalid.
    
    This exception is raised when:
    - Invalid configuration parameters
    - Missing required configuration
    - Configuration file errors
    """
    
    def __init__(self, message: str = "Configuration error", **kwargs):
        super().__init__(message, error_code="CONFIGURATION_ERROR", **kwargs)


class BatchOperationError(VectorizerError):
    """
    Raised when batch operations fail.
    
    This exception is raised when:
    - Batch size exceeds limits
    - Partial batch failures
    - Batch validation errors
    """
    
    def __init__(self, message: str = "Batch operation failed", **kwargs):
        super().__init__(message, error_code="BATCH_OPERATION_ERROR", **kwargs)


# Error mapping for HTTP status codes
HTTP_ERROR_MAPPING = {
    400: ValidationError,
    401: AuthenticationError,
    403: AuthenticationError,
    404: CollectionNotFoundError,
    408: TimeoutError,
    429: RateLimitError,
    500: ServerError,
    502: ServerError,
    503: ServerError,
    504: ServerError,
}


def map_http_error(status_code: int, message: str = None) -> VectorizerError:
    """
    Map HTTP status code to appropriate exception.
    
    Args:
        status_code: HTTP status code
        message: Optional error message
        
    Returns:
        Appropriate exception instance
    """
    error_class = HTTP_ERROR_MAPPING.get(status_code, ServerError)
    default_message = f"HTTP {status_code} error"
    return error_class(message or default_message)
