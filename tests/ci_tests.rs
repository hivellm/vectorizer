//! CI/CD Integration Tests
//! 
//! These tests verify that the CI/CD pipeline components work correctly
//! and that the build process produces valid artifacts.

use std::process::Command;
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn test_cargo_build_release() {
    // Test that release build works
    let output = Command::new("cargo")
        .args(&["build", "--release", "--features", "full"])
        .output()
        .expect("Failed to execute cargo build");
    
    assert!(output.status.success(), "Release build failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_cargo_build_debug() {
    // Test that debug build works
    let output = Command::new("cargo")
        .args(&["build", "--features", "full"])
        .output()
        .expect("Failed to execute cargo build");
    
    assert!(output.status.success(), "Debug build failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_cargo_test_all_features() {
    // Test that all feature combinations work
    let features = ["default", "real-models", "onnx-models", "candle-models", "full"];
    
    for feature in &features {
        let output = Command::new("cargo")
            .args(&["test", "--features", feature])
            .output()
            .expect("Failed to execute cargo test");
        
        assert!(output.status.success(), 
            "Tests failed for feature '{}': {}", 
            feature, String::from_utf8_lossy(&output.stderr));
    }
}

#[tokio::test]
async fn test_cargo_clippy() {
    // Test that clippy passes
    let output = Command::new("cargo")
        .args(&["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
        .output()
        .expect("Failed to execute cargo clippy");
    
    assert!(output.status.success(), "Clippy failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_cargo_fmt() {
    // Test that code is properly formatted
    let output = Command::new("cargo")
        .args(&["fmt", "--all", "--", "--check"])
        .output()
        .expect("Failed to execute cargo fmt");
    
    assert!(output.status.success(), "Code formatting check failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_binary_executables_exist() {
    // Test that required binaries are built
    let binaries = ["vectorizer-cli", "vectorizer-server"];
    
    for binary in &binaries {
        let output = Command::new("cargo")
            .args(&["build", "--release", "--bin", binary])
            .output()
            .expect("Failed to execute cargo build");
        
        assert!(output.status.success(), 
            "Failed to build binary '{}': {}", 
            binary, String::from_utf8_lossy(&output.stderr));
        
        // Check that binary exists
        let binary_path = format!("target/release/{}", binary);
        assert!(Path::new(&binary_path).exists(), 
            "Binary '{}' was not created", binary);
    }
}

#[tokio::test]
async fn test_config_validation() {
    // Test that config.example.yml is valid
    let output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "vectorizer-cli", 
                "config", "validate", "--file", "config.example.yml"])
        .output()
        .expect("Failed to execute config validation");
    
    assert!(output.status.success(), 
        "Config validation failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_documentation_builds() {
    // Test that documentation builds without errors
    let output = Command::new("cargo")
        .args(&["doc", "--all-features", "--no-deps", "--document-private-items"])
        .output()
        .expect("Failed to execute cargo doc");
    
    assert!(output.status.success(), "Documentation build failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_benchmarks_compile() {
    // Test that benchmarks compile (dry run)
    let output = Command::new("cargo")
        .args(&["bench", "--no-run"])
        .output()
        .expect("Failed to execute cargo bench");
    
    assert!(output.status.success(), "Benchmarks compilation failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_docker_build() {
    // Test that Docker image builds successfully
    let output = Command::new("docker")
        .args(&["build", "-t", "vectorizer-test", "."])
        .output()
        .expect("Failed to execute docker build");
    
    assert!(output.status.success(), "Docker build failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_docker_compose_config() {
    // Test that docker-compose.yml is valid
    let output = Command::new("docker-compose")
        .args(&["config"])
        .output()
        .expect("Failed to execute docker-compose config");
    
    assert!(output.status.success(), "Docker Compose config invalid: {}", 
        String::from_utf8_lossy(&output.stderr));
}
