//! Qdrant error models
//!
//! This module provides data structures for Qdrant error responses,
//! including error codes and messages.

use serde::{Deserialize, Serialize};

/// Qdrant error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantErrorResponse {
    /// Error status
    pub status: QdrantErrorStatus,
    /// Error time
    pub time: f64,
}

/// Error status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantErrorStatus {
    /// Error code
    pub error: String,
    /// Error description
    pub description: Option<String>,
}

/// Qdrant error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantErrorCode {
    /// Collection not found
    #[serde(rename = "CollectionNotFound")]
    CollectionNotFound,
    /// Point not found
    #[serde(rename = "PointNotFound")]
    PointNotFound,
    /// Invalid vector dimension
    #[serde(rename = "InvalidVectorDimension")]
    InvalidVectorDimension,
    /// Invalid payload
    #[serde(rename = "InvalidPayload")]
    InvalidPayload,
    /// Invalid filter
    #[serde(rename = "InvalidFilter")]
    InvalidFilter,
    /// Invalid search request
    #[serde(rename = "InvalidSearchRequest")]
    InvalidSearchRequest,
    /// Invalid batch request
    #[serde(rename = "InvalidBatchRequest")]
    InvalidBatchRequest,
    /// Internal error
    #[serde(rename = "InternalError")]
    InternalError,
    /// Bad request
    #[serde(rename = "BadRequest")]
    BadRequest,
    /// Unauthorized
    #[serde(rename = "Unauthorized")]
    Unauthorized,
    /// Forbidden
    #[serde(rename = "Forbidden")]
    Forbidden,
    /// Not found
    #[serde(rename = "NotFound")]
    NotFound,
    /// Method not allowed
    #[serde(rename = "MethodNotAllowed")]
    MethodNotAllowed,
    /// Conflict
    #[serde(rename = "Conflict")]
    Conflict,
    /// Unprocessable entity
    #[serde(rename = "UnprocessableEntity")]
    UnprocessableEntity,
    /// Too many requests
    #[serde(rename = "TooManyRequests")]
    TooManyRequests,
    /// Request entity too large
    #[serde(rename = "RequestEntityTooLarge")]
    RequestEntityTooLarge,
    /// Internal server error
    #[serde(rename = "InternalServerError")]
    InternalServerError,
    /// Bad gateway
    #[serde(rename = "BadGateway")]
    BadGateway,
    /// Gateway timeout
    #[serde(rename = "GatewayTimeout")]
    GatewayTimeout,
}

impl QdrantErrorCode {
    /// Get HTTP status code for error
    pub fn http_status_code(&self) -> u16 {
        match self {
            QdrantErrorCode::CollectionNotFound => 404,
            QdrantErrorCode::PointNotFound => 404,
            QdrantErrorCode::InvalidVectorDimension => 400,
            QdrantErrorCode::InvalidPayload => 400,
            QdrantErrorCode::InvalidFilter => 400,
            QdrantErrorCode::InvalidSearchRequest => 400,
            QdrantErrorCode::InvalidBatchRequest => 400,
            QdrantErrorCode::InternalError => 500,
            QdrantErrorCode::BadRequest => 400,
            QdrantErrorCode::Unauthorized => 401,
            QdrantErrorCode::Forbidden => 403,
            QdrantErrorCode::NotFound => 404,
            QdrantErrorCode::MethodNotAllowed => 405,
            QdrantErrorCode::Conflict => 409,
            QdrantErrorCode::UnprocessableEntity => 422,
            QdrantErrorCode::TooManyRequests => 429,
            QdrantErrorCode::RequestEntityTooLarge => 413,
            QdrantErrorCode::InternalServerError => 500,
            QdrantErrorCode::BadGateway => 502,
            QdrantErrorCode::GatewayTimeout => 504,
        }
    }

    /// Get error message for error code
    pub fn message(&self) -> &'static str {
        match self {
            QdrantErrorCode::CollectionNotFound => "Collection not found",
            QdrantErrorCode::PointNotFound => "Point not found",
            QdrantErrorCode::InvalidVectorDimension => "Invalid vector dimension",
            QdrantErrorCode::InvalidPayload => "Invalid payload",
            QdrantErrorCode::InvalidFilter => "Invalid filter",
            QdrantErrorCode::InvalidSearchRequest => "Invalid search request",
            QdrantErrorCode::InvalidBatchRequest => "Invalid batch request",
            QdrantErrorCode::InternalError => "Internal error",
            QdrantErrorCode::BadRequest => "Bad request",
            QdrantErrorCode::Unauthorized => "Unauthorized",
            QdrantErrorCode::Forbidden => "Forbidden",
            QdrantErrorCode::NotFound => "Not found",
            QdrantErrorCode::MethodNotAllowed => "Method not allowed",
            QdrantErrorCode::Conflict => "Conflict",
            QdrantErrorCode::UnprocessableEntity => "Unprocessable entity",
            QdrantErrorCode::TooManyRequests => "Too many requests",
            QdrantErrorCode::RequestEntityTooLarge => "Request entity too large",
            QdrantErrorCode::InternalServerError => "Internal server error",
            QdrantErrorCode::BadGateway => "Bad gateway",
            QdrantErrorCode::GatewayTimeout => "Gateway timeout",
        }
    }
}
