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

// Re-export service types
pub use server::VectorizerGrpcService;
