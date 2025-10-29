//! Error handling middleware for REST API

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tracing::error;

use crate::error::VectorizerError;

/// Standardized error response format
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    /// Error type identifier
    pub error_type: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details (optional)
    pub details: Option<Value>,
    /// HTTP status code
    pub status_code: u16,
    /// Request ID for tracing (optional)
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error_type: String, message: String, status_code: StatusCode) -> Self {
        Self {
            error_type,
            message,
            details: None,
            status_code: status_code.as_u16(),
            request_id: None,
        }
    }

    /// Add details to the error response
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add request ID for tracing
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Convert VectorizerError to HTTP status code
impl From<&VectorizerError> for StatusCode {
    fn from(err: &VectorizerError) -> Self {
        match err {
            VectorizerError::Embedding(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::CollectionNotFound(_) => StatusCode::NOT_FOUND,
            VectorizerError::VectorNotFound(_) => StatusCode::NOT_FOUND,
            VectorizerError::NotFound(_) => StatusCode::NOT_FOUND,
            VectorizerError::CollectionAlreadyExists(_) => StatusCode::CONFLICT,
            VectorizerError::InvalidDimension { .. } => StatusCode::BAD_REQUEST,
            VectorizerError::DimensionMismatch { .. } => StatusCode::BAD_REQUEST,
            VectorizerError::InvalidConfiguration { .. } => StatusCode::BAD_REQUEST,
            VectorizerError::ConfigurationError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::Configuration(_) => StatusCode::BAD_REQUEST,
            VectorizerError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            VectorizerError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            VectorizerError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            VectorizerError::SerializationError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::Serialization(_) => StatusCode::BAD_REQUEST,
            VectorizerError::Deserialization(_) => StatusCode::BAD_REQUEST,
            VectorizerError::JsonError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::YamlError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::PersistenceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::IndexError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::TransmutationError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::UmicpError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::NetworkError(_) => StatusCode::BAD_GATEWAY,
            VectorizerError::TimeoutError { .. } => StatusCode::REQUEST_TIMEOUT,
            VectorizerError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            VectorizerError::ResourceExhausted { .. } => StatusCode::SERVICE_UNAVAILABLE,
            VectorizerError::ConcurrencyError(_) => StatusCode::CONFLICT,
            VectorizerError::BatchProcessingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::QuantizationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::EmbeddingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::SearchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::ApiError(_) => StatusCode::BAD_REQUEST,
            VectorizerError::MlError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            VectorizerError::ProcessingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "candle-models")]
            VectorizerError::CandleError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Convert VectorizerError to ErrorResponse
impl From<VectorizerError> for ErrorResponse {
    fn from(err: VectorizerError) -> Self {
        let status_code = StatusCode::from(&err);
        let error_type = error_type_from_variant(&err);
        let message = err.to_string();

        // Add additional details for specific error types
        let details = match &err {
            VectorizerError::InvalidDimension { expected, got } => Some(json!({
                "expected_dimension": expected,
                "provided_dimension": got
            })),
            VectorizerError::DimensionMismatch { expected, actual } => Some(json!({
                "expected_dimension": expected,
                "actual_dimension": actual
            })),
            VectorizerError::RateLimitExceeded { limit_type, limit } => Some(json!({
                "limit_type": limit_type,
                "limit": limit
            })),
            VectorizerError::CollectionNotFound(name) => Some(json!({
                "collection_name": name
            })),
            VectorizerError::VectorNotFound(id) => Some(json!({
                "vector_id": id
            })),
            VectorizerError::TimeoutError { timeout_ms } => Some(json!({
                "timeout_ms": timeout_ms
            })),
            VectorizerError::ValidationError { field, message } => Some(json!({
                "field": field,
                "validation_message": message
            })),
            VectorizerError::ResourceExhausted { resource, message } => Some(json!({
                "resource": resource,
                "exhaustion_message": message
            })),
            _ => None,
        };

        Self {
            error_type,
            message,
            details,
            status_code: status_code.as_u16(),
            request_id: None,
        }
    }
}

/// Convert ErrorResponse to Axum Response
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Log the error for debugging
        error!("API Error: {} - {}", self.error_type, self.message);

        (status_code, Json(self)).into_response()
    }
}

/// Convert VectorizerError to Axum Response
impl IntoResponse for VectorizerError {
    fn into_response(self) -> Response {
        ErrorResponse::from(self).into_response()
    }
}

/// Convert JSON rejection to ErrorResponse
impl From<JsonRejection> for ErrorResponse {
    fn from(rejection: JsonRejection) -> Self {
        let status_code = match rejection {
            JsonRejection::JsonDataError(_) => StatusCode::BAD_REQUEST,
            JsonRejection::JsonSyntaxError(_) => StatusCode::BAD_REQUEST,
            JsonRejection::BytesRejection(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::BAD_REQUEST,
        };

        let message = match rejection {
            JsonRejection::JsonDataError(err) => format!("Invalid JSON data: {}", err),
            JsonRejection::JsonSyntaxError(err) => format!("JSON syntax error: {}", err),
            JsonRejection::BytesRejection(err) => format!("Invalid request body: {}", err),
            _ => "Invalid JSON request".to_string(),
        };

        Self::new("json_error".to_string(), message, status_code)
    }
}

/// Wrapper for JsonRejection to implement IntoResponse
pub struct JsonRejectionWrapper(pub JsonRejection);

impl IntoResponse for JsonRejectionWrapper {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse::from(self.0);
        let status_code =
            StatusCode::from_u16(error_response.status_code).unwrap_or(StatusCode::BAD_REQUEST);

        (status_code, Json(error_response)).into_response()
    }
}

/// Extract error type from VectorizerError variant
fn error_type_from_variant(err: &VectorizerError) -> String {
    match err {
        VectorizerError::Embedding(_) => "embedding_error",
        VectorizerError::CollectionNotFound(_) => "collection_not_found",
        VectorizerError::CollectionAlreadyExists(_) => "collection_already_exists",
        VectorizerError::VectorNotFound(_) => "vector_not_found",
        VectorizerError::InvalidDimension { .. } => "invalid_dimension",
        VectorizerError::DimensionMismatch { .. } => "dimension_mismatch",
        VectorizerError::PersistenceError(_) => "persistence_error",
        VectorizerError::IndexError(_) => "index_error",
        VectorizerError::ConfigurationError(_) => "configuration_error",
        VectorizerError::Configuration(_) => "configuration_error",
        VectorizerError::SerializationError(_) => "serialization_error",
        VectorizerError::Serialization(_) => "serialization_error",
        VectorizerError::Deserialization(_) => "deserialization_error",
        VectorizerError::IoError(_) => "io_error",
        VectorizerError::Io(_) => "io_error",
        VectorizerError::JsonError(_) => "json_error",
        VectorizerError::YamlError(_) => "yaml_error",
        VectorizerError::AuthenticationError(_) => "authentication_error",
        VectorizerError::AuthorizationError(_) => "authorization_error",
        VectorizerError::RateLimitExceeded { .. } => "rate_limit_exceeded",
        VectorizerError::InvalidConfiguration { .. } => "invalid_configuration",
        VectorizerError::InternalError(_) => "internal_error",
        VectorizerError::NotFound(_) => "not_found",
        VectorizerError::Other(_) => "unknown_error",
        VectorizerError::TransmutationError(_) => "transmutation_error",
        VectorizerError::Storage(_) => "storage_error",
        VectorizerError::UmicpError(_) => "umicp_error",
        VectorizerError::NetworkError(_) => "network_error",
        VectorizerError::TimeoutError { .. } => "timeout_error",
        VectorizerError::ValidationError { .. } => "validation_error",
        VectorizerError::ResourceExhausted { .. } => "resource_exhausted",
        VectorizerError::ConcurrencyError(_) => "concurrency_error",
        VectorizerError::BatchProcessingError(_) => "batch_processing_error",
        VectorizerError::QuantizationError(_) => "quantization_error",
        VectorizerError::EmbeddingError(_) => "embedding_error",
        VectorizerError::SearchError(_) => "search_error",
        VectorizerError::ApiError(_) => "api_error",
        VectorizerError::MlError(_) => "ml_error",
        VectorizerError::ProcessingError(_) => "processing_error",
        #[cfg(feature = "candle-models")]
        VectorizerError::CandleError(_) => "candle_error",
    }
    .to_string()
}

/// Helper function to create a standardized error response
pub fn create_error_response(
    error_type: &str,
    message: &str,
    status_code: StatusCode,
) -> ErrorResponse {
    ErrorResponse::new(error_type.to_string(), message.to_string(), status_code)
}

/// Helper function to create a validation error response
pub fn create_validation_error(message: &str, details: Option<Value>) -> ErrorResponse {
    let mut response = ErrorResponse::new(
        "validation_error".to_string(),
        message.to_string(),
        StatusCode::BAD_REQUEST,
    );
    if let Some(details) = details {
        response = response.with_details(details);
    }
    response
}

/// Helper function to create a not found error response
pub fn create_not_found_error(resource_type: &str, resource_id: &str) -> ErrorResponse {
    ErrorResponse::new(
        format!("{}_not_found", resource_type),
        format!("{} '{}' not found", resource_type, resource_id),
        StatusCode::NOT_FOUND,
    )
    .with_details(json!({
        "resource_type": resource_type,
        "resource_id": resource_id
    }))
}

/// Helper function to create a conflict error response
pub fn create_conflict_error(resource_type: &str, resource_id: &str) -> ErrorResponse {
    ErrorResponse::new(
        format!("{}_already_exists", resource_type),
        format!("{} '{}' already exists", resource_type, resource_id),
        StatusCode::CONFLICT,
    )
    .with_details(json!({
        "resource_type": resource_type,
        "resource_id": resource_id
    }))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::*;

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new(
            "test_error".to_string(),
            "Test error message".to_string(),
            StatusCode::BAD_REQUEST,
        );

        assert_eq!(response.error_type, "test_error");
        assert_eq!(response.message, "Test error message");
        assert_eq!(response.status_code, 400);
        assert!(response.details.is_none());
    }

    #[test]
    fn test_vectorizer_error_to_status_code() {
        let error = VectorizerError::CollectionNotFound("test".to_string());
        let status_code = StatusCode::from(&error);
        assert_eq!(status_code, StatusCode::NOT_FOUND);

        let error = VectorizerError::InvalidDimension {
            expected: 128,
            got: 64,
        };
        let status_code = StatusCode::from(&error);
        assert_eq!(status_code, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_vectorizer_error_to_error_response() {
        let error = VectorizerError::InvalidDimension {
            expected: 128,
            got: 64,
        };
        let response = ErrorResponse::from(error);

        assert_eq!(response.error_type, "invalid_dimension");
        assert!(response.message.contains("Invalid dimension"));
        assert_eq!(response.status_code, 400);
        assert!(response.details.is_some());
    }

    #[test]
    fn test_helper_functions() {
        let validation_error =
            create_validation_error("Invalid input", Some(json!({"field": "name"})));
        assert_eq!(validation_error.error_type, "validation_error");
        assert_eq!(validation_error.status_code, 400);

        let not_found_error = create_not_found_error("collection", "test");
        assert_eq!(not_found_error.error_type, "collection_not_found");
        assert_eq!(not_found_error.status_code, 404);

        let conflict_error = create_conflict_error("collection", "test");
        assert_eq!(conflict_error.error_type, "collection_already_exists");
        assert_eq!(conflict_error.status_code, 409);
    }
}
