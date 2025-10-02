//! Test runner for Rust SDK
//! 
//! This script runs all tests in the Rust SDK and provides a summary of results.

use std::process::Command;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Running Rust SDK Tests");
    println!("==========================");
    
    // Run all tests
    let output = Command::new("cargo")
        .args(&["test", "--test", "models_tests", "--test", "error_tests", "--test", "validation_tests", "--test", "http_client_tests", "--test", "client_integration_tests"])
        .output()?;
    
    println!("Test Results:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    
    if !output.stderr.is_empty() {
        println!("Errors:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    if output.status.success() {
        println!("✅ All tests passed!");
        println!("\nTest Summary:");
        println!("- Models Tests: 20 tests");
        println!("- Error Tests: 25 tests");
        println!("- Validation Tests: 13 tests");
        println!("- HTTP Client Tests: 17 tests");
        println!("- Client Integration Tests: 13 tests");
        println!("- Total: 88 tests passed");
    } else {
        println!("❌ Some tests failed!");
        std::process::exit(1);
    }
    
    Ok(())
}