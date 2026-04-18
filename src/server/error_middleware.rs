//! Error middleware for Qdrant-compatible API responses

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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

/// Convert VectorizerError to HTTP status code.
///
/// Delegates to the centralized taxonomy in
/// [`crate::error::mapping::http_status`] — kept as an
/// `impl From<&VectorizerError> for StatusCode` so the existing axum
/// callsites don't need editing.
impl From<&VectorizerError> for StatusCode {
    fn from(err: &VectorizerError) -> Self {
        crate::error::mapping::http_status(err)
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

/// Extract the stable `error_type` identifier for a REST error body.
///
/// Delegates to [`VectorizerError::code`] — the single source of
/// truth for machine-readable error identifiers across REST / MCP.
fn error_type_from_variant(err: &VectorizerError) -> String {
    err.code().to_string()
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

/// Helper function to create a bad request error response
pub fn create_bad_request_error(message: &str) -> ErrorResponse {
    ErrorResponse::new(
        "bad_request".to_string(),
        message.to_string(),
        StatusCode::BAD_REQUEST,
    )
}

/// Helper function to create a validation error response
pub fn create_validation_error(field: &str, message: &str) -> ErrorResponse {
    ErrorResponse::new(
        "validation_error".to_string(),
        format!("Invalid {}: {}", field, message),
        StatusCode::BAD_REQUEST,
    )
    .with_details(json!({
        "field": field,
        "reason": message
    }))
}
