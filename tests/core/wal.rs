//! Write-Ahead Log (WAL) Tests
//!
//! Tests for WAL functionality:
//! - WAL integration with VectorStore
//! - Comprehensive WAL operations
//! - Crash recovery scenarios

// Include tests from moved files
include!("wal_vector_store.rs");
include!("wal_comprehensive.rs");
include!("wal_crash_recovery.rs");
