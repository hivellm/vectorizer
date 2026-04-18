//! Centralized conversions from [`VectorizerError`] to wire-protocol
//! error shapes.
//!
//! The three shapes are:
//! - axum [`axum::http::StatusCode`] — for the REST API (via
//!   [`crate::server::error_middleware::ErrorResponse`])
//! - `tonic::Status` — for the gRPC surface
//! - MCP `ErrorData` code — for the Model Context Protocol entry point
//!
//! Every mapping here is a pure function of [`ErrorKind`]. Adding a
//! new [`VectorizerError`] variant should pin it to an existing
//! `ErrorKind` in [`kind`](super::kind) first; only when a new wire
//! protocol needs a distinct code should a new `ErrorKind` appear.

use super::{ErrorKind, VectorizerError};

/// Convert a [`VectorizerError`] reference to an HTTP status code.
///
/// Equivalent to `error.kind().http_status()` — exposed as a
/// standalone conversion so the existing
/// `impl From<&VectorizerError> for StatusCode` in
/// `server::error_middleware` can delegate here without pulling in
/// the axum types from this crate module.
pub fn http_status(err: &VectorizerError) -> axum::http::StatusCode {
    err.kind().http_status()
}

/// Convert a [`VectorizerError`] reference to a gRPC [`tonic::Code`].
pub fn grpc_code(err: &VectorizerError) -> tonic::Code {
    err.kind().grpc_code()
}

/// Convert a [`VectorizerError`] reference to an MCP error code.
///
/// Codes follow the JSON-RPC 2.0 error range convention used by the
/// MCP spec:
/// - `-32600` — invalid request (BadRequest kind)
/// - `-32601` — method not found (NotFound kind)
/// - `-32603` — internal error (Internal / Unavailable)
/// - `-32001..=-32099` — server-defined range for auth / conflict /
///   rate-limit
pub fn mcp_code(err: &VectorizerError) -> i32 {
    err.kind().mcp_code()
}

impl ErrorKind {
    /// Map to an HTTP status code.
    pub fn http_status(self) -> axum::http::StatusCode {
        match self {
            ErrorKind::NotFound => axum::http::StatusCode::NOT_FOUND,
            ErrorKind::Unauthorized => axum::http::StatusCode::UNAUTHORIZED,
            ErrorKind::Forbidden => axum::http::StatusCode::FORBIDDEN,
            ErrorKind::BadRequest => axum::http::StatusCode::BAD_REQUEST,
            ErrorKind::Conflict => axum::http::StatusCode::CONFLICT,
            ErrorKind::TooManyRequests => axum::http::StatusCode::TOO_MANY_REQUESTS,
            ErrorKind::Unavailable => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            ErrorKind::Internal => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Map to a gRPC [`tonic::Code`].
    pub fn grpc_code(self) -> tonic::Code {
        match self {
            ErrorKind::NotFound => tonic::Code::NotFound,
            ErrorKind::Unauthorized => tonic::Code::Unauthenticated,
            ErrorKind::Forbidden => tonic::Code::PermissionDenied,
            ErrorKind::BadRequest => tonic::Code::InvalidArgument,
            ErrorKind::Conflict => tonic::Code::AlreadyExists,
            ErrorKind::TooManyRequests => tonic::Code::ResourceExhausted,
            ErrorKind::Unavailable => tonic::Code::Unavailable,
            ErrorKind::Internal => tonic::Code::Internal,
        }
    }

    /// Map to a JSON-RPC / MCP error code.
    pub fn mcp_code(self) -> i32 {
        match self {
            ErrorKind::BadRequest => -32600, // Invalid Request
            ErrorKind::NotFound => -32601,   // Method not found
            ErrorKind::Internal | ErrorKind::Unavailable => -32603, // Internal error
            ErrorKind::Unauthorized => -32001, // Server-defined: unauthorized
            ErrorKind::Forbidden => -32002,  // Server-defined: forbidden
            ErrorKind::Conflict => -32003,   // Server-defined: conflict
            ErrorKind::TooManyRequests => -32004, // Server-defined: rate limit
        }
    }
}

impl From<VectorizerError> for tonic::Status {
    /// Convert into a `tonic::Status` with the right code + the
    /// error's `Display` message. Replaces the ad-hoc
    /// `tonic::Status::unknown(format!("{}", e))` pattern found across
    /// the gRPC layer.
    fn from(err: VectorizerError) -> Self {
        let code = grpc_code(&err);
        tonic::Status::new(code, err.to_string())
    }
}

impl From<&VectorizerError> for tonic::Status {
    fn from(err: &VectorizerError) -> Self {
        let code = grpc_code(err);
        tonic::Status::new(code, err.to_string())
    }
}
