//! Tests for cache management system

use super::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_cache_manager_creation() {
    let temp_dir = tempdir().unwrap();
    let cache_path = temp_dir.path().join("test_cache");

    let config = CacheConfig {
        cache_path: cache_path.clone(),
        validation_level: ValidationLevel::Basic,
        cleanup: CleanupConfig::default(),
        compression: CompressionConfig::default(),
        max_size_bytes: 1024 * 1024, // 1MB
        ttl_seconds: 3600,           // 1 hour
    };

    let manager = CacheManager::new(config).await.unwrap();
    assert!(manager.get_cache_path().exists());
    assert!(manager.get_metadata_path().exists());
}

#[tokio::test]
async fn test_cache_metadata_operations() {
    let temp_dir = tempdir().unwrap();
    let cache_path = temp_dir.path().join("test_cache");

    let config = CacheConfig {
        cache_path: cache_path.clone(),
        validation_level: ValidationLevel::Basic,
        cleanup: CleanupConfig::default(),
        compression: CompressionConfig::default(),
        max_size_bytes: 1024 * 1024,
        ttl_seconds: 3600,
    };

    let manager = CacheManager::new(config).await.unwrap();

    // Test collection operations
    let collection_info = CollectionCacheInfo::new(
        "test_collection".to_string(),
        "bm25".to_string(),
        "1.0.0".to_string(),
    );

    manager
        .update_collection_info(collection_info.clone())
        .await
        .unwrap();

    assert!(manager.has_collection("test_collection").await);

    let retrieved_info = manager
        .get_collection_info("test_collection")
        .await
        .unwrap();
    assert_eq!(retrieved_info.name, "test_collection");

    // Test cache hit/miss recording
    manager.record_hit().await.unwrap();
    manager.record_miss().await.unwrap();

    let stats = manager.get_stats().await;
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate, 0.5);
}

#[tokio::test]
async fn test_cache_validation() {
    let temp_dir = tempdir().unwrap();
    let cache_path = temp_dir.path().join("test_cache");

    let config = CacheConfig {
        cache_path: cache_path.clone(),
        validation_level: ValidationLevel::Basic,
        cleanup: CleanupConfig::default(),
        compression: CompressionConfig::default(),
        max_size_bytes: 1024 * 1024,
        ttl_seconds: 3600,
    };

    let manager = CacheManager::new(config).await.unwrap();

    // Test validation
    let validation_result = manager.validate().await.unwrap();
    assert!(validation_result.is_valid());
}

#[tokio::test]
async fn test_cache_cleanup() {
    let temp_dir = tempdir().unwrap();
    let cache_path = temp_dir.path().join("test_cache");

    let config = CacheConfig {
        cache_path: cache_path.clone(),
        validation_level: ValidationLevel::Basic,
        cleanup: CleanupConfig {
            enabled: true,
            max_age_seconds: 1, // Very short TTL for testing
            max_size_bytes: 1024,
            interval_seconds: 1,
        },
        compression: CompressionConfig::default(),
        max_size_bytes: 1024 * 1024,
        ttl_seconds: 1, // Very short TTL for testing
    };

    let manager = CacheManager::new(config).await.unwrap();

    // Add a collection
    let collection_info = CollectionCacheInfo::new(
        "test_collection".to_string(),
        "bm25".to_string(),
        "1.0.0".to_string(),
    );

    manager
        .update_collection_info(collection_info)
        .await
        .unwrap();

    // Wait for TTL to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Run cleanup
    let cleanup_result = manager.cleanup().await.unwrap();
    assert!(cleanup_result.removed_collections > 0);
}

#[tokio::test]
async fn test_incremental_processor() {
    let temp_dir = tempdir().unwrap();
    let cache_path = temp_dir.path().join("test_cache");

    let config = CacheConfig {
        cache_path: cache_path.clone(),
        validation_level: ValidationLevel::Basic,
        cleanup: CleanupConfig::default(),
        compression: CompressionConfig::default(),
        max_size_bytes: 1024 * 1024,
        ttl_seconds: 3600,
    };

    let cache_manager = Arc::new(CacheManager::new(config).await.unwrap());
    let incremental_config = IncrementalConfig::default();

    let processor = IncrementalProcessor::new(cache_manager, incremental_config)
        .await
        .unwrap();

    // Test adding a processing task
    let task = ProcessingTask {
        id: "test_task".to_string(),
        operation: ProcessingOperation::IndexFile {
            collection_name: "test_collection".to_string(),
            file_path: PathBuf::from("test_file.txt"),
            change_type: FileChangeEvent::Created(PathBuf::from("test_file.txt")),
        },
        priority: TaskPriority::Normal,
        created_at: chrono::Utc::now(),
        retry_count: 0,
    };

    processor.add_task(task).await.unwrap();

    let queue_size = processor.queue_size().await;
    assert_eq!(queue_size, 1);
}

#[tokio::test]
async fn test_file_hash_calculation() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create a test file
    std::fs::write(&test_file, "Hello, World!").unwrap();

    let validator = CacheValidator::new(ValidationConfig::default());
    let hash = validator.calculate_file_hash(&test_file).await.unwrap();

    // SHA-256 of "Hello, World!" should be consistent
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64); // SHA-256 hex string length
}

#[tokio::test]
async fn test_cache_metadata_serialization() {
    let mut metadata = CacheMetadata::new("1.0.0".to_string());

    // Add a collection
    let collection_info = CollectionCacheInfo::new(
        "test_collection".to_string(),
        "bm25".to_string(),
        "1.0.0".to_string(),
    );

    metadata.update_collection(collection_info);

    // Test serialization
    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: CacheMetadata = serde_json::from_str(&serialized).unwrap();

    assert_eq!(metadata.version, deserialized.version);
    assert_eq!(metadata.collections.len(), deserialized.collections.len());
    assert!(deserialized.has_collection("test_collection"));
}
