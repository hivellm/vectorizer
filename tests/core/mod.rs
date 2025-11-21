//! Core Functionality Tests
//!
//! Tests for core Vectorizer functionality:
//! - SIMD operations
//! - Quantization
//! - Storage backends
//! - Write-Ahead Log

pub mod simd;
pub mod quantization;
pub mod storage;
pub mod storage_integration;
pub mod wal;

