//! Benchmark Helper Utilities
//!
//! This module provides common utilities and data structures for benchmarking
//! the Vectorizer vector database. It includes test data generation, performance
//! measurement, report generation, and configuration management utilities.
//!
//! # Usage
//!
//! ```rust,ignore
//! use vectorizer::benchmark::{
//!     BenchmarkConfig, BenchmarkRunner, TestDataGenerator,
//!     PerformanceMetrics, ReportGenerator
//! };
//!
//! // Create benchmark configuration
//! let config = BenchmarkConfig::default();
//!
//! // Generate test data
//! let test_data = TestDataGenerator::new(config.clone())
//!     .generate_vectors(10000, 256);
//!
//! // See benchmark module documentation for usage details
//! ```

pub mod config;
pub mod data_generator;
pub mod metrics;
pub mod reporter;
pub mod runner;
pub mod system_monitor;
pub mod utils;

// Re-export commonly used types
pub use config::BenchmarkConfig;
pub use data_generator::TestDataGenerator;
pub use metrics::{BenchmarkResult, OperationMetrics, PerformanceMetrics};
pub use reporter::ReportGenerator;
pub use runner::BenchmarkRunner;
pub use system_monitor::SystemMonitor;
pub use utils::*;
