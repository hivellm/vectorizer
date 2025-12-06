//! Core Functionality Tests
//!
//! Tests for core Vectorizer functionality:
//! - SIMD operations
//! - Quantization
//! - Storage backends
//! - Write-Ahead Log
//! - Collection cleanup

pub mod collection_cleanup;
pub mod quantization;
pub mod simd;
pub mod storage;
pub mod storage_integration;
pub mod wal;
