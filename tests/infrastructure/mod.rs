//! Infrastructure Tests
//!
//! Tests for infrastructure concerns:
//! - Docker and virtual paths
//! - Logging configuration
//! - Tier-1 marker gate (AGENTS.md rule #1)
//! - JWT secret auto-generation boot path

pub mod docker;
pub mod jwt_secret_boot;
pub mod logging;
pub mod tier1_marker_gate;
