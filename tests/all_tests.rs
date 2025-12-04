//! All Tests - Organized by Category
//!
//! This file imports all test modules organized by theme
//!
//! Note: Each test module imports its own helpers to avoid duplicate mod errors

mod api;
mod core;
mod gpu;
mod hub;
mod infrastructure;
mod integration;
mod performance;
mod replication;
mod workflow;

// Legacy tests that haven't been migrated yet
// These will be gradually moved to appropriate categories
#[path = "test_new_features.rs"]
mod test_new_features;
