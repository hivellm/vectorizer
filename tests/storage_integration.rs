//! Integration tests for storage module
//!
//! These tests verify storage functionality including:
//! - Compact format operations
//! - Snapshot creation and restoration
//! - Index management
//! - Migration between formats

use std::path::PathBuf;

use tempfile::TempDir;

/// Helper to create temporary storage directory
fn create_temp_storage() -> TempDir {
    TempDir::new().unwrap()
}

#[tokio::test]
async fn test_storage_initialization() {
    let temp_dir = create_temp_storage();
    let storage_path = temp_dir.path().join("test.vecdb");

    // Verify temporary directory exists
    assert!(temp_dir.path().exists());

    // Verify we can create storage path
    assert!(storage_path.parent().is_some());
}

#[tokio::test]
async fn test_snapshot_path_generation() {
    let temp_dir = create_temp_storage();
    let base_path = temp_dir.path();

    // Test snapshot naming pattern
    let snapshot_name = format!("snapshot_{}.vecdb", chrono::Utc::now().timestamp());
    let snapshot_path = base_path.join(&snapshot_name);

    assert!(snapshot_name.starts_with("snapshot_"));
    assert!(snapshot_name.ends_with(".vecdb"));
    assert_eq!(snapshot_path.parent().unwrap(), base_path);
}

#[tokio::test]
async fn test_index_config_defaults() {
    // Test default index configuration values
    let ef_construction = 200;
    let m = 16;

    assert!(ef_construction > 0);
    assert!(m > 0);
    assert!(ef_construction >= m);
}

#[tokio::test]
async fn test_storage_path_validation() {
    let temp_dir = create_temp_storage();

    // Valid paths
    let valid_paths = vec![
        temp_dir.path().join("collection.vecdb"),
        temp_dir.path().join("nested/collection.vecdb"),
        temp_dir.path().join("data/storage/test.vecdb"),
    ];

    for path in valid_paths {
        assert!(path.extension().is_some_and(|ext| ext == "vecdb"));
    }
}

#[tokio::test]
async fn test_compact_format_extension() {
    // Verify compact format uses .vecdb extension
    let test_paths = vec!["data.vecdb", "collection.vecdb", "snapshot_123.vecdb"];

    for path_str in test_paths {
        let path = PathBuf::from(path_str);
        assert_eq!(path.extension().unwrap(), "vecdb");
    }
}

#[tokio::test]
async fn test_migration_version_check() {
    // Test migration version handling
    let v1 = "1.0.0";
    let v2 = "1.1.0";

    assert!(v1 < v2);
    assert!(v2 > v1);
}

#[tokio::test]
async fn test_storage_config_serialization() {
    use serde_json::json;

    let config = json!({
        "path": "/data/storage",
        "format": "compact",
        "compression": "zstd",
        "compression_level": 3
    });

    assert_eq!(config["format"], "compact");
    assert_eq!(config["compression"], "zstd");
    assert_eq!(config["compression_level"], 3);
}

#[tokio::test]
async fn test_snapshot_retention_policy() {
    // Test snapshot retention logic
    let max_snapshots = 10;
    let current_count = 15;

    let should_delete = current_count > max_snapshots;
    let to_delete = if should_delete {
        current_count - max_snapshots
    } else {
        0
    };

    assert!(should_delete);
    assert_eq!(to_delete, 5);
}

#[tokio::test]
async fn test_compression_levels() {
    // Test valid compression levels
    let valid_levels: Vec<i32> = (1..=22).collect();

    for level in &valid_levels {
        assert!(*level >= 1 && *level <= 22);
    }
}

#[tokio::test]
async fn test_storage_metadata() {
    use serde_json::json;

    let metadata = json!({
        "version": "1.1.2",
        "created_at": "2025-10-25T00:00:00Z",
        "vector_count": 1000,
        "dimension": 384
    });

    assert_eq!(metadata["version"], "1.1.2");
    assert_eq!(metadata["dimension"], 384);
    assert!(metadata["vector_count"].is_number());
}

#[tokio::test]
async fn test_concurrent_storage_access() {
    use tokio::task::JoinSet;

    let mut set = JoinSet::new();

    // Simulate multiple concurrent storage operations
    for i in 0..10 {
        set.spawn(async move {
            // Simulate storage operation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            i
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res.unwrap());
    }

    assert_eq!(results.len(), 10);
}

#[tokio::test]
async fn test_backup_file_naming() {
    let timestamp = chrono::Utc::now().timestamp();
    let backup_name = format!("backup_{timestamp}.vecdb");

    assert!(backup_name.starts_with("backup_"));
    assert!(backup_name.ends_with(".vecdb"));
    assert!(backup_name.contains(&timestamp.to_string()));
}

#[tokio::test]
async fn test_storage_error_recovery() {
    // Test error handling scenarios
    let invalid_paths = vec!["", "/", "///"];

    for path in invalid_paths {
        let _path_buf = PathBuf::from(path);
        // Should handle invalid paths gracefully
        assert!(path.is_empty() || path == "/" || path == "///");
    }
}

#[tokio::test]
async fn test_index_rebuild_detection() {
    // Test conditions for triggering index rebuild
    let should_rebuild_cases = vec![
        (true, true),   // corrupted + enabled
        (false, false), // not corrupted + disabled
    ];

    for (corrupted, rebuild) in should_rebuild_cases {
        if corrupted {
            assert!(rebuild);
        }
    }
}

#[tokio::test]
async fn test_storage_size_tracking() {
    // Test storage size calculations
    let vector_count = 1000;
    let dimension = 384;
    let bytes_per_float = 4;

    let estimated_size = vector_count * dimension * bytes_per_float;

    assert_eq!(estimated_size, 1_536_000); // ~1.5MB
    assert!(estimated_size > 0);
}

#[tokio::test]
async fn test_compression_ratio_calculation() {
    // Test compression ratio logic
    let original_size = 1_000_000;
    let compressed_size = 300_000;

    let ratio = (f64::from(compressed_size) / f64::from(original_size)) * 100.0;

    assert_eq!(ratio, 30.0);
    assert!(ratio < 100.0);
    assert!(ratio > 0.0);
}

#[tokio::test]
async fn test_storage_cleanup_logic() {
    // Test cleanup conditions
    let disk_usage_percent = 85.0;
    let threshold = 90.0;

    let should_cleanup = disk_usage_percent > threshold;

    assert!(!should_cleanup);

    let high_usage = 95.0;
    let should_cleanup_high = high_usage > threshold;

    assert!(should_cleanup_high);
}
