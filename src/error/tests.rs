//! Classification + mapping round-trip tests.
//!
//! These tests lock in the taxonomy so that a future edit to
//! [`VectorizerError`] can't silently downgrade a `NotFound` into a
//! `500` (the exact failure mode described in the
//! phase3_unify-error-enums proposal).

use super::{ErrorKind, VectorizerError};

/// Every variant that carries user-supplied context is a 404.
#[test]
fn not_found_family_classifies_as_not_found() {
    assert_eq!(
        VectorizerError::CollectionNotFound("x".into()).kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        VectorizerError::VectorNotFound("v1".into()).kind(),
        ErrorKind::NotFound
    );
    assert_eq!(
        VectorizerError::NotFound("anything".into()).kind(),
        ErrorKind::NotFound
    );
}

#[test]
fn auth_variants_split_between_401_and_403() {
    assert_eq!(
        VectorizerError::AuthenticationError("no token".into()).kind(),
        ErrorKind::Unauthorized
    );
    assert_eq!(
        VectorizerError::AuthorizationError("needs admin".into()).kind(),
        ErrorKind::Forbidden
    );
}

#[test]
fn collection_already_exists_is_conflict_not_bad_request() {
    assert_eq!(
        VectorizerError::CollectionAlreadyExists("dup".into()).kind(),
        ErrorKind::Conflict
    );
}

#[test]
fn dimension_and_config_errors_are_bad_request() {
    assert_eq!(
        VectorizerError::InvalidDimension {
            expected: 384,
            got: 512,
        }
        .kind(),
        ErrorKind::BadRequest,
    );
    assert_eq!(
        VectorizerError::DimensionMismatch {
            expected: 384,
            actual: 512,
        }
        .kind(),
        ErrorKind::BadRequest,
    );
    assert_eq!(
        VectorizerError::ConfigurationError("missing field".into()).kind(),
        ErrorKind::BadRequest
    );
}

#[test]
fn rate_limit_is_429() {
    assert_eq!(
        VectorizerError::RateLimitExceeded {
            limit_type: "global".into(),
            limit: 100,
        }
        .kind(),
        ErrorKind::TooManyRequests,
    );
}

#[test]
fn storage_and_internal_are_500() {
    assert_eq!(
        VectorizerError::PersistenceError("disk full".into()).kind(),
        ErrorKind::Internal
    );
    assert_eq!(
        VectorizerError::IndexError("hnsw corrupt".into()).kind(),
        ErrorKind::Internal
    );
    assert_eq!(
        VectorizerError::Storage(".vecdb write failed".into()).kind(),
        ErrorKind::Internal
    );
    assert_eq!(
        VectorizerError::InternalError("oops".into()).kind(),
        ErrorKind::Internal
    );
    assert_eq!(
        VectorizerError::Other("catch-all".into()).kind(),
        ErrorKind::Internal
    );
}

/// HTTP mapping covers all eight kinds.
#[test]
fn http_mapping_is_exhaustive() {
    assert_eq!(
        ErrorKind::NotFound.http_status(),
        axum::http::StatusCode::NOT_FOUND
    );
    assert_eq!(
        ErrorKind::Unauthorized.http_status(),
        axum::http::StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        ErrorKind::Forbidden.http_status(),
        axum::http::StatusCode::FORBIDDEN
    );
    assert_eq!(
        ErrorKind::BadRequest.http_status(),
        axum::http::StatusCode::BAD_REQUEST
    );
    assert_eq!(
        ErrorKind::Conflict.http_status(),
        axum::http::StatusCode::CONFLICT
    );
    assert_eq!(
        ErrorKind::TooManyRequests.http_status(),
        axum::http::StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        ErrorKind::Unavailable.http_status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        ErrorKind::Internal.http_status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn grpc_mapping_preserves_classification() {
    assert_eq!(ErrorKind::NotFound.grpc_code(), tonic::Code::NotFound);
    assert_eq!(
        ErrorKind::Unauthorized.grpc_code(),
        tonic::Code::Unauthenticated
    );
    assert_eq!(
        ErrorKind::Forbidden.grpc_code(),
        tonic::Code::PermissionDenied
    );
    assert_eq!(
        ErrorKind::BadRequest.grpc_code(),
        tonic::Code::InvalidArgument
    );
    assert_eq!(ErrorKind::Conflict.grpc_code(), tonic::Code::AlreadyExists);
    assert_eq!(
        ErrorKind::TooManyRequests.grpc_code(),
        tonic::Code::ResourceExhausted
    );
    assert_eq!(ErrorKind::Unavailable.grpc_code(), tonic::Code::Unavailable);
    assert_eq!(ErrorKind::Internal.grpc_code(), tonic::Code::Internal);
}

#[test]
fn mcp_codes_are_stable() {
    assert_eq!(ErrorKind::BadRequest.mcp_code(), -32600);
    assert_eq!(ErrorKind::NotFound.mcp_code(), -32601);
    assert_eq!(ErrorKind::Internal.mcp_code(), -32603);
    assert_eq!(ErrorKind::Unavailable.mcp_code(), -32603);
    assert_eq!(ErrorKind::Unauthorized.mcp_code(), -32001);
    assert_eq!(ErrorKind::Forbidden.mcp_code(), -32002);
    assert_eq!(ErrorKind::Conflict.mcp_code(), -32003);
    assert_eq!(ErrorKind::TooManyRequests.mcp_code(), -32004);
}

/// A `VectorizerError` converts directly into a `tonic::Status` with
/// the right code — this replaces ~dozens of `tonic::Status::unknown`
/// callsites across `src/grpc/`.
#[test]
fn tonic_conversion_picks_correct_code_not_unknown() {
    let err = VectorizerError::CollectionNotFound("test".into());
    let status: tonic::Status = err.into();
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert!(status.message().contains("Collection not found"));

    let err = VectorizerError::InvalidDimension {
        expected: 384,
        got: 512,
    };
    let status: tonic::Status = err.into();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);

    let err = VectorizerError::RateLimitExceeded {
        limit_type: "global".into(),
        limit: 100,
    };
    let status: tonic::Status = err.into();
    assert_eq!(status.code(), tonic::Code::ResourceExhausted);

    // The old behavior was `tonic::Status::unknown(...)` for every error,
    // which is specifically what this conversion is meant to fix.
    let err = VectorizerError::Storage(".vecdb missing".into());
    let status: tonic::Status = err.into();
    assert_ne!(status.code(), tonic::Code::Unknown);
    assert_eq!(status.code(), tonic::Code::Internal);
}

/// The stable `code()` identifier must stay consistent across versions —
/// client code may match on these strings.
#[test]
fn error_code_strings_are_stable() {
    assert_eq!(
        VectorizerError::CollectionNotFound("x".into()).code(),
        "collection_not_found"
    );
    assert_eq!(
        VectorizerError::VectorNotFound("y".into()).code(),
        "vector_not_found"
    );
    assert_eq!(
        VectorizerError::AuthenticationError("no token".into()).code(),
        "authentication_error"
    );
    assert_eq!(
        VectorizerError::RateLimitExceeded {
            limit_type: "global".into(),
            limit: 100,
        }
        .code(),
        "rate_limit_exceeded"
    );
    assert_eq!(
        VectorizerError::Storage("disk".into()).code(),
        "storage_error"
    );
}
