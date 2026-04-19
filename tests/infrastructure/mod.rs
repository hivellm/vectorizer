//! Infrastructure Tests
//!
//! Tests for infrastructure concerns:
//! - Docker and virtual paths
//! - Logging configuration
//! - Tier-1 marker gate (AGENTS.md rule #1)
//! - JWT secret auto-generation boot path
//! - Handler robustness (no-panic-on-malformed-input)

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod docker;
pub mod handler_robustness;
pub mod jwt_secret_boot;
pub mod logging;
pub mod tier1_marker_gate;
