//! gRPC server implementation for Vectorizer
//!
//! This module provides gRPC API support using tonic, offering the same
//! functionality as the REST API but with better performance for high-throughput scenarios.

pub mod conversions;
pub mod server;

// Include generated protobuf code
pub mod vectorizer {
    include!("vectorizer.rs");
}

// Include generated cluster proto code
pub mod cluster {
    include!("vectorizer.cluster.rs");
}

// Include generated Qdrant-compatible protobuf code
pub mod qdrant_proto {
    include!("qdrant/qdrant.rs");
}

// Qdrant gRPC service implementations
pub mod qdrant_grpc;

// Re-export service types
pub use qdrant_grpc::QdrantGrpcService;
pub use server::VectorizerGrpcService;
