//! Tests for system metrics functionality

use std::sync::Arc;
use super::MetricsCollector;

#[tokio::test]
async fn test_system_metrics_functionality() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Test initial system metrics
    let initial_metrics = metrics.get_metrics().await;
    
    // Memory usage should be > 0 on Linux systems
    println!("Memory usage: {} bytes", initial_metrics.system.memory_usage_bytes);
    println!("CPU usage: {}%", initial_metrics.system.cpu_usage_percent);
    println!("Thread count: {}", initial_metrics.system.thread_count);
    println!("Active file handles: {}", initial_metrics.system.active_file_handles);
    
    // On Linux systems, these should be > 0
    if cfg!(target_os = "linux") {
        assert!(initial_metrics.system.memory_usage_bytes > 0, "Memory usage should be > 0 on Linux");
        assert!(initial_metrics.system.thread_count > 0, "Thread count should be > 0");
        assert!(initial_metrics.system.active_file_handles > 0, "Active file handles should be > 0");
    }
    
    // Test connection tracking
    metrics.record_connection_opened();
    metrics.record_connection_opened();
    
    let metrics_with_connections = metrics.get_metrics().await;
    assert_eq!(metrics_with_connections.network.active_connections, 2);
    
    metrics.record_connection_closed();
    let metrics_after_close = metrics.get_metrics().await;
    assert_eq!(metrics_after_close.network.active_connections, 1);
    
    // Test I/O tracking
    metrics.record_disk_io(1024);
    metrics.record_disk_io(2048);
    metrics.record_network_io(512);
    
    let metrics_with_io = metrics.get_metrics().await;
    assert_eq!(metrics_with_io.system.disk_io_ops_per_sec, 2);
    assert_eq!(metrics_with_io.system.network_io_bytes_per_sec, 512);
    
    println!("✅ All system metrics are working correctly!");
}

#[tokio::test]
async fn test_connection_tracking() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Test multiple connections
    for i in 0..10 {
        metrics.record_connection_opened();
        let current_metrics = metrics.get_metrics().await;
        assert_eq!(current_metrics.network.active_connections, (i + 1) as u32);
    }
    
    // Close some connections
    for i in 0..5 {
        metrics.record_connection_closed();
        let current_metrics = metrics.get_metrics().await;
        assert_eq!(current_metrics.network.active_connections, (10 - i - 1) as u32);
    }
    
    let final_metrics = metrics.get_metrics().await;
    assert_eq!(final_metrics.network.active_connections, 5);
    
    println!("✅ Connection tracking is working correctly!");
}

#[tokio::test]
async fn test_io_tracking() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Test disk I/O
    let mut total_disk_ops = 0;
    for i in 1..=5 {
        metrics.record_disk_io(i * 1024);
        total_disk_ops += 1;
        
        let current_metrics = metrics.get_metrics().await;
        assert_eq!(current_metrics.system.disk_io_ops_per_sec, total_disk_ops);
    }
    
    // Test network I/O
    let mut total_network_bytes = 0;
    for i in 1..=3 {
        let bytes = i * 512;
        metrics.record_network_io(bytes);
        total_network_bytes += bytes;
        
        let current_metrics = metrics.get_metrics().await;
        assert_eq!(current_metrics.system.network_io_bytes_per_sec, total_network_bytes);
    }
    
    let final_metrics = metrics.get_metrics().await;
    assert_eq!(final_metrics.system.disk_io_ops_per_sec, 5);
    assert_eq!(final_metrics.system.network_io_bytes_per_sec, 3072); // 512 + 1024 + 1536
    
    println!("✅ I/O tracking is working correctly!");
}

#[tokio::test]
async fn test_comprehensive_metrics() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Simulate a comprehensive workload
    metrics.record_connection_opened();
    metrics.record_connection_opened();
    
    metrics.record_file_processing_complete(true, 100.0).await;
    metrics.record_file_processing_complete(true, 200.0).await;
    metrics.record_file_processing_complete(false, 150.0).await;
    
    metrics.record_api_request(true, 50.0);
    metrics.record_api_request(true, 100.0);
    metrics.record_api_request(false, 200.0);
    
    metrics.record_disk_io(1024);
    metrics.record_network_io(512);
    
    metrics.record_discovery(5, 50.0).await;
    metrics.record_sync(2, 3, 75.0).await;
    
    let final_metrics = metrics.get_metrics().await;
    
    // Verify all metrics are working
    assert_eq!(final_metrics.files.total_files_processed, 3);
    assert_eq!(final_metrics.files.files_processed_success, 2);
    assert_eq!(final_metrics.files.files_processed_error, 1);
    
    assert_eq!(final_metrics.network.total_api_requests, 3);
    assert_eq!(final_metrics.network.successful_api_requests, 2);
    assert_eq!(final_metrics.network.failed_api_requests, 1);
    assert_eq!(final_metrics.network.active_connections, 2);
    
    assert_eq!(final_metrics.system.disk_io_ops_per_sec, 1);
    assert_eq!(final_metrics.system.network_io_bytes_per_sec, 512);
    
    assert_eq!(final_metrics.files.files_discovered, 5);
    assert_eq!(final_metrics.files.files_removed, 2);
    assert_eq!(final_metrics.files.files_indexed_realtime, 3);
    
    // Verify timing calculations
    assert_eq!(final_metrics.timing.avg_file_processing_ms, 150.0); // (100 + 200 + 150) / 3
    assert_eq!(final_metrics.network.avg_api_response_ms, 116.66666666666667); // (50 + 100 + 200) / 3
    
    // Verify health score
    // Note: record_file_processing_complete(false) and record_api_request(false) both increment total_errors
    // So we have 2 total errors from 3 file processing + 3 API requests = 6 total operations
    // 4 success, 2 errors = 66% success rate
    assert_eq!(final_metrics.status.health_score, 66); // 4 success, 2 errors = 66%
    
    println!("✅ Comprehensive metrics test passed!");
    println!("   - Files processed: {}", final_metrics.files.total_files_processed);
    println!("   - API requests: {}", final_metrics.network.total_api_requests);
    println!("   - Active connections: {}", final_metrics.network.active_connections);
    println!("   - Health score: {}%", final_metrics.status.health_score);
    println!("   - Memory usage: {} bytes", final_metrics.system.memory_usage_bytes);
    println!("   - Thread count: {}", final_metrics.system.thread_count);
}
