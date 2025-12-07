//! Integration Tests
//!
//! Tests for integrated features:
//! - Hybrid search
//! - Sparse vectors
//! - Payload indexing
//! - Query caching
//! - Binary quantization
//! - New implementations (batch insert, hybrid search, rate limiting, etc.)

pub mod binary_quantization;
pub mod cluster;
pub mod cluster_e2e;
pub mod cluster_failures;
pub mod cluster_fault_tolerance;
pub mod cluster_integration;
pub mod cluster_multitenant;
pub mod cluster_performance;
pub mod cluster_scale;
pub mod distributed_search;
pub mod distributed_sharding;
pub mod graph;
pub mod hybrid_search;
pub mod multi_tenancy;
pub mod multi_tenancy_comprehensive;
pub mod new_implementations;
pub mod payload_index;
pub mod qdrant_api;
pub mod query_cache;
pub mod raft;
pub mod raft_comprehensive;
pub mod sharding;
pub mod sharding_comprehensive;
pub mod sharding_validation;
pub mod sparse_vector;
