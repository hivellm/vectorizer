//! gRPC API Test Suite
//!
//! Tests organized by theme:
//! - helpers: Shared test utilities
//! - collections: Collection management operations
//! - vectors: Vector CRUD operations
//! - search: Search operations (basic, batch, hybrid)
//! - configurations: Different configs (metrics, storage, quantization, HNSW)
//! - edge_cases: Boundary conditions and error handling
//! - performance: Stress tests and concurrent operations
//! - s2s: Server-to-server tests (requires s2s-tests feature)

pub mod helpers;

pub mod collections;
pub mod vectors;
// TODO: Create remaining modules
// pub mod search;
// pub mod configurations;
// pub mod edge_cases;
// pub mod performance;

#[cfg(feature = "s2s-tests")]
pub mod s2s;

