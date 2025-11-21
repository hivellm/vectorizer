//! Tests for logging level functionality

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn test_logging_function_exists() {
    // Test that the logging function exists and can be called
    use vectorizer::logging;

    // Function should exist and compile
    let _ = logging::init_logging_with_level("test_service", "warn");
    let _ = logging::init_logging_with_level("test_service", "info");
    let _ = logging::init_logging_with_level("test_service", "debug");
    let _ = logging::init_logging_with_level("test_service", "error");
}

#[test]
fn test_logging_default_function() {
    // Test that the default logging function still works
    use vectorizer::logging;

    // Should compile and work (may fail if already initialized, which is OK)
    let _ = logging::init_logging("test_service");
}

#[test]
fn test_logging_helpers() {
    use vectorizer::logging;

    // Test helper functions exist
    let logs_dir = logging::get_logs_dir();
    assert!(logs_dir.to_string_lossy().contains(".logs"));

    let log_path = logging::get_log_file_path("test_service", None);
    assert!(log_path.to_string_lossy().contains("test_service"));
    assert!(log_path.to_string_lossy().contains(".log"));
}
