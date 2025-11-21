//! Write-Ahead Log (WAL) Tests
//!
//! Tests for WAL functionality:
//! - WAL integration with VectorStore
//! - Comprehensive WAL operations
//! - Crash recovery scenarios

// Include tests from test_vector_store_wal.rs
include!("../test_vector_store_wal.rs");

// Include tests from test_wal_comprehensive.rs
include!("../test_wal_comprehensive.rs");

// Include tests from wal_crash_recovery.rs
include!("../wal_crash_recovery.rs");

