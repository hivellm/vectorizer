//! Replication Tests
//!
//! Tests for replication functionality:
//! - Basic replication
//! - Failover scenarios
//! - Replication handlers
//! - Qdrant compatibility

#![allow(clippy::unwrap_used, clippy::expect_used)]

// NOTE (phase39): `api.rs`, `comprehensive.rs`, `integration_basic.rs`
// are compiled through `integration.rs` via `#[path]` includes, and
// `qdrant_api.rs` / `qdrant_migration.rs` through `qdrant.rs` — they
// are NOT orphaned even though they don't appear as `pub mod` lines
// here. Declaring them here too is a clippy `duplicate_mod` error.
// (The 2026-07-11 analysis §3.6 flagged them as never-compiled; that
// finding was wrong — corrected in the analysis doc.)
pub mod failover;
pub mod handlers;
pub mod integration;
pub mod qdrant;
