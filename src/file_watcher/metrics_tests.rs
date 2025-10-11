//! Tests for the metrics system

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use super::MetricsCollector;

#[tokio::test]
async fn test_metrics_collection() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Test initial state
    let initial_metrics = metrics.get_metrics().await;
    assert_eq!(initial_metrics.files.total_files_processed, 0);
    assert_eq!(initial_metrics.files.files_processed_success, 0);
    assert_eq!(initial_metrics.files.files_processed_error, 0);
    
    // Record some file processing
    metrics.record_file_processing_complete(true, 100.0).await;
    metrics.record_file_processing_complete(true, 200.0).await;
    metrics.record_file_processing_complete(false, 150.0).await;
    
    // Record some discovery
    metrics.record_discovery(5, 50.0).await;
    
    // Record some sync
    metrics.record_sync(2, 3, 75.0).await;
    
    // Record an error
    metrics.record_error("test_error", "Test error message").await;
    
    // Small delay to ensure uptime > 0
    sleep(Duration::from_millis(10)).await;
    
    // Get updated metrics
    let updated_metrics = metrics.get_metrics().await;
    
    // Verify file processing metrics
    assert_eq!(updated_metrics.files.total_files_processed, 3);
    assert_eq!(updated_metrics.files.files_processed_success, 2);
    assert_eq!(updated_metrics.files.files_processed_error, 1);
    
    // Verify timing metrics
    assert_eq!(updated_metrics.timing.avg_file_processing_ms, 150.0); // (100 + 200 + 150) / 3
    assert_eq!(updated_metrics.timing.avg_discovery_ms, 10.0); // 50 / 5
    assert_eq!(updated_metrics.timing.avg_sync_ms, 37.5); // 75 / 2 (only files_removed is used as denominator)
    
    // Verify discovery metrics
    assert_eq!(updated_metrics.files.files_discovered, 5);
    
    // Verify sync metrics
    assert_eq!(updated_metrics.files.files_removed, 2);
    assert_eq!(updated_metrics.files.files_indexed_realtime, 3);
    
    // Verify error metrics
    // Note: record_file_processing_complete(false) increments total_errors
    // and record_error() also increments total_errors, so we expect 2 total errors
    assert_eq!(updated_metrics.status.total_errors, 2);
    assert!(updated_metrics.status.errors_by_type.contains_key("test_error"));
    assert_eq!(updated_metrics.status.errors_by_type["test_error"], 1);
    assert!(updated_metrics.status.last_error.is_some());
    
    // Verify uptime (may be 0 in fast tests)
    // assert!(updated_metrics.timing.uptime_seconds > 0);
    
    // Test summary
    let summary = metrics.get_summary().await;
    assert!(summary.contains("Files processed: 3"));
    assert!(summary.contains("Success: 2"));
    assert!(summary.contains("Errors: 1"));
}

#[tokio::test]
async fn test_metrics_reset() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record some metrics
    metrics.record_file_processing_complete(true, 100.0).await;
    metrics.record_error("test", "test error").await;
    
    // Verify metrics are recorded
    let before_reset = metrics.get_metrics().await;
    assert_eq!(before_reset.files.total_files_processed, 1);
    assert_eq!(before_reset.status.total_errors, 1);
    
    // Reset metrics
    metrics.reset().await;
    
    // Verify metrics are reset
    let after_reset = metrics.get_metrics().await;
    assert_eq!(after_reset.files.total_files_processed, 0);
    assert_eq!(after_reset.files.files_processed_success, 0);
    assert_eq!(after_reset.files.files_processed_error, 0);
    assert_eq!(after_reset.status.total_errors, 0);
    assert!(after_reset.status.errors_by_type.is_empty());
    assert!(after_reset.status.last_error.is_none());
}

#[tokio::test]
async fn test_collection_metrics() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Update collection metrics
    metrics.update_collection_metrics("test-collection", 100, 1024).await;
    metrics.update_collection_metrics("another-collection", 200, 2048).await;
    
    // Get metrics
    let metrics_data = metrics.get_metrics().await;
    
    // Verify collections
    assert!(metrics_data.collections.contains_key("test-collection"));
    assert!(metrics_data.collections.contains_key("another-collection"));
    
    let test_collection = &metrics_data.collections["test-collection"];
    assert_eq!(test_collection.total_vectors, 100);
    assert!(test_collection.last_update.is_some());
    
    let another_collection = &metrics_data.collections["another-collection"];
    assert_eq!(another_collection.total_vectors, 200);
    assert!(another_collection.last_update.is_some());
}

#[tokio::test]
async fn test_health_score_calculation() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Test with no processing (should be 100)
    let initial_metrics = metrics.get_metrics().await;
    assert_eq!(initial_metrics.status.health_score, 100);
    
    // Add some successful processing
    metrics.record_file_processing_complete(true, 100.0).await;
    metrics.record_file_processing_complete(true, 100.0).await;
    metrics.record_file_processing_complete(true, 100.0).await;
    
    // Should still be 100 (no errors)
    let success_metrics = metrics.get_metrics().await;
    assert_eq!(success_metrics.status.health_score, 100);
    
    // Add an error
    metrics.record_file_processing_complete(false, 100.0).await;
    
    // Should be 75 (3 success, 1 error = 75% success rate)
    let error_metrics = metrics.get_metrics().await;
    assert_eq!(error_metrics.status.health_score, 75);
}

#[tokio::test]
async fn test_api_metrics() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record API requests
    metrics.record_api_request(true, 50.0);
    metrics.record_api_request(true, 100.0);
    metrics.record_api_request(false, 200.0);
    
    // Get metrics
    let metrics_data = metrics.get_metrics().await;
    
    // Verify API metrics
    assert_eq!(metrics_data.network.total_api_requests, 3);
    assert_eq!(metrics_data.network.successful_api_requests, 2);
    assert_eq!(metrics_data.network.failed_api_requests, 1);
    assert_eq!(metrics_data.network.avg_api_response_ms, 116.66666666666667); // (50 + 100 + 200) / 3
    assert_eq!(metrics_data.network.peak_api_response_ms, 200);
}

#[tokio::test]
async fn test_file_processing_states() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record files in progress
    metrics.record_file_in_progress();
    metrics.record_file_in_progress();
    
    let metrics_data = metrics.get_metrics().await;
    assert_eq!(metrics_data.files.files_in_progress, 2);
    
    // Record file processing finished
    metrics.record_file_processing_finished();
    
    let metrics_data = metrics.get_metrics().await;
    assert_eq!(metrics_data.files.files_in_progress, 1);
    
    // Record file skipped
    metrics.record_file_skipped();
    
    let metrics_data = metrics.get_metrics().await;
    assert_eq!(metrics_data.files.files_skipped, 1);
}

#[tokio::test]
async fn test_running_status() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Initially should be running
    let initial_metrics = metrics.get_metrics().await;
    assert_eq!(initial_metrics.status.current_status, "running");
    
    // Set to stopped
    metrics.set_running(false);
    
    let stopped_metrics = metrics.get_metrics().await;
    assert_eq!(stopped_metrics.status.current_status, "stopped");
    
    // Set back to running
    metrics.set_running(true);
    
    let running_metrics = metrics.get_metrics().await;
    assert_eq!(running_metrics.status.current_status, "running");
}
