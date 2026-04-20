//! `ErrorKind` — the classification taxonomy for [`VectorizerError`].
//!
//! Each kind maps to exactly one HTTP status, one gRPC `Code`, and one
//! MCP error code, and the mapping lives in [`super::mapping`]. Adding
//! a new variant to [`VectorizerError`] should pin it to an existing
//! `ErrorKind` first — introduce a new `ErrorKind` only when the three
//! wire protocols require a distinct code.

use super::VectorizerError;

/// The category of an error, independent of its message.
///
/// Used by the three wire-protocol mappers (HTTP / gRPC / MCP) to
/// pick the right status / code. If a new wire protocol is added,
/// its mapper only needs to cover every variant of this enum — no
/// need to touch every `VectorizerError` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The requested resource does not exist. HTTP 404 / gRPC NOT_FOUND.
    NotFound,
    /// The caller is not authenticated. HTTP 401 / gRPC UNAUTHENTICATED.
    Unauthorized,
    /// The caller is authenticated but lacks permission. HTTP 403 /
    /// gRPC PERMISSION_DENIED.
    Forbidden,
    /// Request payload or parameters failed validation. HTTP 400 /
    /// gRPC INVALID_ARGUMENT.
    BadRequest,
    /// The resource already exists / a conflicting state prevents the
    /// operation. HTTP 409 / gRPC ALREADY_EXISTS.
    Conflict,
    /// Rate limit exceeded. HTTP 429 / gRPC RESOURCE_EXHAUSTED.
    TooManyRequests,
    /// A subsystem is temporarily unavailable (watcher down, cluster
    /// degraded). HTTP 503 / gRPC UNAVAILABLE.
    Unavailable,
    /// Any other unexpected condition. HTTP 500 / gRPC INTERNAL.
    Internal,
}

impl VectorizerError {
    /// Classify this error into an [`ErrorKind`].
    ///
    /// The classification is lossless from the caller's point of view:
    /// every variant of [`VectorizerError`] maps to exactly one kind,
    /// and the HTTP / gRPC / MCP mappers derive their status codes from
    /// this single source of truth.
    pub fn kind(&self) -> ErrorKind {
        match self {
            // Not found
            VectorizerError::CollectionNotFound(_)
            | VectorizerError::VectorNotFound(_)
            | VectorizerError::NotFound(_) => ErrorKind::NotFound,

            // Auth
            VectorizerError::AuthenticationError(_) => ErrorKind::Unauthorized,
            VectorizerError::AuthorizationError(_) => ErrorKind::Forbidden,

            // Conflict
            VectorizerError::CollectionAlreadyExists(_) => ErrorKind::Conflict,

            // Rate limit
            VectorizerError::RateLimitExceeded { .. } => ErrorKind::TooManyRequests,

            // Bad request — invalid input, dimension, config, encryption, or encoding failures
            // that originate from the caller's payload.
            VectorizerError::InvalidDimension { .. }
            | VectorizerError::DimensionMismatch { .. }
            | VectorizerError::InvalidConfiguration { .. }
            | VectorizerError::ConfigurationError(_)
            | VectorizerError::Configuration(_)
            | VectorizerError::EncryptionRequired(_)
            | VectorizerError::EncryptionError(_)
            | VectorizerError::SerializationError(_)
            | VectorizerError::Serialization(_)
            | VectorizerError::Deserialization(_)
            | VectorizerError::JsonError(_)
            | VectorizerError::YamlError(_)
            | VectorizerError::TransmutationError(_)
            | VectorizerError::UmicpError(_) => ErrorKind::BadRequest,

            // Internal — everything else (I/O, persistence, indexing, ML, catch-all).
            VectorizerError::IoError(_)
            | VectorizerError::Io(_)
            | VectorizerError::PersistenceError(_)
            | VectorizerError::IndexError(_)
            | VectorizerError::Storage(_)
            | VectorizerError::InternalError(_)
            | VectorizerError::Unimplemented(_)
            | VectorizerError::Other(_) => ErrorKind::Internal,

            #[cfg(feature = "candle-models")]
            VectorizerError::CandleError(_) => ErrorKind::Internal,
        }
    }

    /// A short, stable machine-readable identifier for this error —
    /// used as the `error_type` field in REST and MCP responses.
    ///
    /// Unlike the human-readable `Display` output (which embeds
    /// user-supplied strings), this identifier is fixed per variant
    /// and safe to match on in client code.
    pub fn code(&self) -> &'static str {
        match self {
            VectorizerError::CollectionNotFound(_) => "collection_not_found",
            VectorizerError::CollectionAlreadyExists(_) => "collection_already_exists",
            VectorizerError::VectorNotFound(_) => "vector_not_found",
            VectorizerError::InvalidDimension { .. } => "invalid_dimension",
            VectorizerError::DimensionMismatch { .. } => "dimension_mismatch",
            VectorizerError::PersistenceError(_) => "persistence_error",
            VectorizerError::IndexError(_) => "index_error",
            VectorizerError::ConfigurationError(_) | VectorizerError::Configuration(_) => {
                "configuration_error"
            }
            VectorizerError::SerializationError(_) | VectorizerError::Serialization(_) => {
                "serialization_error"
            }
            VectorizerError::Deserialization(_) => "deserialization_error",
            VectorizerError::IoError(_) | VectorizerError::Io(_) => "io_error",
            VectorizerError::JsonError(_) => "json_error",
            VectorizerError::YamlError(_) => "yaml_error",
            VectorizerError::AuthenticationError(_) => "authentication_error",
            VectorizerError::AuthorizationError(_) => "authorization_error",
            VectorizerError::EncryptionRequired(_) => "encryption_required",
            VectorizerError::EncryptionError(_) => "encryption_error",
            VectorizerError::RateLimitExceeded { .. } => "rate_limit_exceeded",
            VectorizerError::InvalidConfiguration { .. } => "invalid_configuration",
            VectorizerError::InternalError(_) => "internal_error",
            VectorizerError::NotFound(_) => "not_found",
            VectorizerError::Other(_) => "other_error",
            VectorizerError::UmicpError(_) => "umicp_error",
            VectorizerError::TransmutationError(_) => "transmutation_error",
            VectorizerError::Storage(_) => "storage_error",
            VectorizerError::Unimplemented(_) => "unimplemented",
            #[cfg(feature = "candle-models")]
            VectorizerError::CandleError(_) => "candle_error",
        }
    }
}
