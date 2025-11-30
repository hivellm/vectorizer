//! Write-Ahead Log (WAL) Tests
//!
//! Tests for WAL functionality:
//! - WAL integration with VectorStore
//! - Comprehensive WAL operations
//! - Crash recovery scenarios

// Each file is a separate module to avoid import conflicts
#[path = "wal_vector_store.rs"]
mod wal_vector_store;

#[path = "wal_comprehensive.rs"]
mod wal_comprehensive;

#[path = "wal_crash_recovery.rs"]
mod wal_crash_recovery;
