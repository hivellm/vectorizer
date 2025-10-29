//! Error middleware for Qdrant-compatible API responses

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::error;

use crate::error::VectorizerError;

/// Standard error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error_type: String, message: String, status_code: StatusCode) -> Self {
        Self {
            error_type,
            message,
            details: None,
            status_code: status_code.as_u16(),
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Convert VectorizerError to HTTP status code
impl From<&VectorizerError> for StatusCode {
    fn from(err: &VectorizerError) -> Self {
        match err {
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

        let details = match &err {
            VectorizerError::InvalidDimension { expected, got } => Some(json!({
                "expected_dimension": expected,
                "actual_dimension": got
            })),
            VectorizerError::DimensionMismatch { expected, actual } => Some(json!({
                "expected": expected,
                "actual": actual
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

        error!("API Error: {} - {}", self.error_type, self.message);

        (status_code, Json(self)).into_response()
    }
}

/// Extract error type from VectorizerError variant
fn error_type_from_variant(err: &VectorizerError) -> String {
    match err {
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

/// Helper function to create a not found error response
pub fn create_not_found_error(resource_type: &str, resource_id: &str) -> ErrorResponse {
    ErrorResponse::new(
        format!("{}_not_found", resource_type),
        format!("{} '{}' not found", resource_type, resource_id),
        StatusCode::NOT_FOUND,
    )
}

/// Helper function to create a conflict error response
pub fn create_conflict_error(resource_type: &str, resource_id: &str) -> ErrorResponse {
    ErrorResponse::new(
        format!("{}_already_exists", resource_type),
        format!("{} '{}' already exists", resource_type, resource_id),
        StatusCode::CONFLICT,
    )
}
