//! Core Functionality Tests
//!
//! Tests for core Vectorizer functionality:
//! - SIMD operations
//! - Quantization
//! - Storage backends
//! - Write-Ahead Log
//! - Collection cleanup
//! - Collection persistence

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod backpressure;
pub mod backpressure_indexer;
pub mod backpressure_metrics;
pub mod bm25_warn_rate_limit;
pub mod collection_cleanup;
pub mod persistence;
pub mod quantization;
pub mod simd;
pub mod storage;
pub mod storage_integration;
pub mod upsert_queue;
pub mod wal;
