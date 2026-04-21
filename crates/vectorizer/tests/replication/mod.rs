//! Replication Tests
//!
//! Tests for replication functionality:
//! - Basic replication
//! - Failover scenarios
//! - Replication handlers
//! - Qdrant compatibility

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod failover;
pub mod handlers;
pub mod integration;
pub mod qdrant;
