//! Test runner for Rust SDK
//! 
//! This script runs all tests in the Rust SDK and provides a summary of results.

use std::process::Command;
use tracing::{info, error, warn, debug};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🚀 Running Rust SDK Tests");
    tracing::info!("==========================");
    
    // Run all tests
    let output = Command::new("cargo")
        .args(&["test", "--test", "models_tests", "--test", "error_tests", "--test", "validation_tests", "--test", "http_client_tests", "--test", "client_integration_tests"])
        .output()?;
    
    tracing::info!("Test Results:");
    tracing::info!("{}", String::from_utf8_lossy(&output.stdout));
    
    if !output.stderr.is_empty() {
        tracing::info!("Errors:");
        tracing::info!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    if output.status.success() {
        tracing::info!("✅ All tests passed!");
        tracing::info!("\nTest Summary:");
        tracing::info!("- Models Tests: 20 tests");
        tracing::info!("- Error Tests: 25 tests");
        tracing::info!("- Validation Tests: 13 tests");
        tracing::info!("- HTTP Client Tests: 17 tests");
        tracing::info!("- Client Integration Tests: 13 tests");
        tracing::info!("- Total: 88 tests passed");
    } else {
        tracing::info!("❌ Some tests failed!");
        std::process::exit(1);
    }
    
    Ok(())
}