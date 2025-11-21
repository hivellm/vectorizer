//! All Tests - Organized by Category
//!
//! This file imports all test modules organized by theme

mod core;
mod api;
mod integration;
mod replication;
mod performance;
mod gpu;
mod workflow;
mod infrastructure;

// Legacy tests that haven't been migrated yet
// These will be gradually moved to appropriate categories
#[path = "test_new_features.rs"]
mod test_new_features;

