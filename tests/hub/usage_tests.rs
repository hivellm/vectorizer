//! Usage tracking tests for HiveHub integration

use vectorizer::hub::usage::UsageMetrics;

#[test]
fn test_usage_metrics_default() {
    let metrics = UsageMetrics::new();

    assert_eq!(metrics.vectors_inserted, 0);
    assert_eq!(metrics.vectors_deleted, 0);
    assert_eq!(metrics.storage_added, 0);
    assert_eq!(metrics.storage_freed, 0);
    assert_eq!(metrics.search_count, 0);
    assert_eq!(metrics.collections_created, 0);
    assert_eq!(metrics.collections_deleted, 0);
    assert_eq!(metrics.api_requests, 0);
}

#[test]
fn test_usage_metrics_record_insert() {
    let mut metrics = UsageMetrics::new();
    metrics.record_insert(100, 1024 * 1024); // 100 vectors, 1MB

    assert_eq!(metrics.vectors_inserted, 100);
    assert_eq!(metrics.storage_added, 1024 * 1024);
    assert_eq!(metrics.api_requests, 1);

    // Record another insert
    metrics.record_insert(50, 512 * 1024);

    assert_eq!(metrics.vectors_inserted, 150);
    assert_eq!(metrics.storage_added, 1536 * 1024);
    assert_eq!(metrics.api_requests, 2);
}

#[test]
fn test_usage_metrics_record_delete() {
    let mut metrics = UsageMetrics::new();
    metrics.record_delete(50, 512 * 1024);

    assert_eq!(metrics.vectors_deleted, 50);
    assert_eq!(metrics.storage_freed, 512 * 1024);
    assert_eq!(metrics.api_requests, 1);
}

#[test]
fn test_usage_metrics_record_search() {
    let mut metrics = UsageMetrics::new();

    for _ in 0..10 {
        metrics.record_search();
    }

    assert_eq!(metrics.search_count, 10);
    assert_eq!(metrics.api_requests, 10);
}

#[test]
fn test_usage_metrics_record_collection_operations() {
    let mut metrics = UsageMetrics::new();

    metrics.record_collection_create();
    metrics.record_collection_create();
    metrics.record_collection_delete();

    assert_eq!(metrics.collections_created, 2);
    assert_eq!(metrics.collections_deleted, 1);
    assert_eq!(metrics.api_requests, 3);
}

#[test]
fn test_usage_metrics_net_vectors() {
    let mut metrics = UsageMetrics::new();
    metrics.record_insert(100, 0);
    metrics.record_delete(30, 0);

    assert_eq!(metrics.net_vectors(), 70);

    // More deletes than inserts
    let mut metrics2 = UsageMetrics::new();
    metrics2.record_insert(10, 0);
    metrics2.record_delete(50, 0);

    assert_eq!(metrics2.net_vectors(), -40);
}

#[test]
fn test_usage_metrics_net_storage() {
    let mut metrics = UsageMetrics::new();
    metrics.record_insert(0, 1000);
    metrics.record_delete(0, 300);

    assert_eq!(metrics.net_storage(), 700);

    // More freed than added
    let mut metrics2 = UsageMetrics::new();
    metrics2.record_insert(0, 100);
    metrics2.record_delete(0, 500);

    assert_eq!(metrics2.net_storage(), -400);
}

#[test]
fn test_usage_metrics_net_collections() {
    let mut metrics = UsageMetrics::new();
    metrics.record_collection_create();
    metrics.record_collection_create();
    metrics.record_collection_delete();

    assert_eq!(metrics.net_collections(), 1);

    // More deletes than creates
    let mut metrics2 = UsageMetrics::new();
    metrics2.record_collection_create();
    metrics2.record_collection_delete();
    metrics2.record_collection_delete();

    assert_eq!(metrics2.net_collections(), -1);
}

#[test]
fn test_usage_metrics_merge() {
    let mut metrics1 = UsageMetrics::new();
    metrics1.record_insert(100, 1000);
    metrics1.record_search();

    let mut metrics2 = UsageMetrics::new();
    metrics2.record_insert(50, 500);
    metrics2.record_delete(20, 200);
    metrics2.record_collection_create();

    metrics1.merge(&metrics2);

    assert_eq!(metrics1.vectors_inserted, 150);
    assert_eq!(metrics1.vectors_deleted, 20);
    assert_eq!(metrics1.storage_added, 1500);
    assert_eq!(metrics1.storage_freed, 200);
    assert_eq!(metrics1.search_count, 1);
    assert_eq!(metrics1.collections_created, 1);
    assert_eq!(metrics1.api_requests, 5); // 2 from metrics1 + 3 from metrics2
}

#[test]
fn test_usage_metrics_has_changes() {
    let empty = UsageMetrics::new();
    assert!(!empty.has_changes());

    // Inserts count as changes
    let mut with_inserts = UsageMetrics::new();
    with_inserts.record_insert(1, 100);
    assert!(with_inserts.has_changes());

    // Deletes count as changes
    let mut with_deletes = UsageMetrics::new();
    with_deletes.record_delete(1, 100);
    assert!(with_deletes.has_changes());

    // Collection creation counts as changes
    let mut with_collection = UsageMetrics::new();
    with_collection.record_collection_create();
    assert!(with_collection.has_changes());

    // Search alone does not count as changes (no data modification)
    let mut with_search_only = UsageMetrics::new();
    with_search_only.record_search();
    assert!(!with_search_only.has_changes());

    // Generic request alone does not count as changes
    let mut with_request_only = UsageMetrics::new();
    with_request_only.record_request();
    assert!(!with_request_only.has_changes());
}

#[test]
fn test_usage_metrics_serialization() {
    let mut metrics = UsageMetrics::new();
    metrics.record_insert(100, 1000);
    metrics.record_search();

    // Should be serializable
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("vectors_inserted"));
    assert!(json.contains("100"));

    // Should be deserializable
    let deserialized: UsageMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.vectors_inserted, 100);
    assert_eq!(deserialized.storage_added, 1000);
}
