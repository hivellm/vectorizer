//! All Tests - Organized by Category
//!
//! This file imports all test modules organized by theme
//!
//! Note: Each test module imports its own helpers to avoid duplicate mod errors

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod api;
mod cluster;
mod config;
mod core;
mod discovery;
mod gpu;
mod hub;
mod infrastructure;
mod integration;
mod performance;
mod protocol;
mod replication;
mod workflow;

// Legacy tests that haven't been migrated yet
// These will be gradually moved to appropriate categories
#[path = "test_new_features.rs"]
mod test_new_features;
