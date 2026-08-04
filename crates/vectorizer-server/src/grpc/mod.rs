//! gRPC server implementation for Vectorizer
//!
//! This module provides gRPC API support using tonic, offering the same
//! functionality as the REST API but with better performance for high-throughput scenarios.

// `conversions` lives in the umbrella `vectorizer` crate
// (`vectorizer::grpc_conversions`) because its
// `impl From<proto::*> for models::*` blocks need both types local
// for the orphan rule.
pub mod server;

// Generated proto modules live in the `vectorizer-grpc` crate. Re-exported
// here so existing consumers (`use vectorizer::grpc::vectorizer::*` etc.)
// keep working without code changes.
pub use vectorizer_grpc::grpc_gen::{cluster, qdrant_proto, vectorizer};

// Qdrant gRPC service implementations (server-side, depend on the
// storage engine — stay in this crate, not the wire-protocol crate).
pub mod qdrant_grpc;

// Re-export service types
pub use qdrant_grpc::QdrantGrpcService;
pub use server::VectorizerGrpcService;
