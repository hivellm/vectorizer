//! gRPC server implementation for Vectorizer
//!
//! This module provides gRPC API support using tonic, offering the same
//! functionality as the REST API but with better performance for high-throughput scenarios.

pub mod conversions;
pub mod server;

// Include generated protobuf code. The included files are auto-generated
// by `tonic-prost-build` on every `cargo build`; we cannot annotate
// individual items, so the wrapping module silences `missing_docs` for
// the whole tree.
#[allow(missing_docs)]
pub mod vectorizer {
    include!("vectorizer.rs");
}

#[allow(missing_docs)]
pub mod cluster {
    include!("vectorizer.cluster.rs");
}

#[allow(missing_docs)]
pub mod qdrant_proto {
    include!("qdrant/qdrant.rs");
}

// Qdrant gRPC service implementations
pub mod qdrant_grpc;

// Re-export service types
pub use qdrant_grpc::QdrantGrpcService;
pub use server::VectorizerGrpcService;
