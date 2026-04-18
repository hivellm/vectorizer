//! Infrastructure Tests
//!
//! Tests for infrastructure concerns:
//! - Docker and virtual paths
//! - Logging configuration
//! - Tier-1 marker gate (AGENTS.md rule #1)

pub mod docker;
pub mod logging;
pub mod tier1_marker_gate;
