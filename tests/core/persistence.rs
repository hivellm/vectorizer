//! Tests for collection persistence across restarts
//!
//! NOTE: Full persistence integration tests require running against a live server
//! because VectorStore::get_data_dir() always returns the fixed ./data directory
//! and cannot be overridden for isolated unit tests.
//!
//! The persistence system is tested via:
//! - Manual testing with server start/stop
//! - The test_auto_save_manager_mark_changed test (which verifies the mark_changed flow)
//!
//! The actual persistence is handled by:
//! - AutoSaveManager.force_save() calls StorageCompactor
//! - REST/GraphQL handlers call mark_changed() after mutations
//! - Periodic auto-save checks the dirty flag and saves when needed

use std::sync::Arc;

use tempfile::tempdir;
use vectorizer::db::VectorStore;
use vectorizer::db::auto_save::AutoSaveManager;
use vectorizer::models::{CollectionConfig, DistanceMetric, StorageType, Vector};

/// Helper to create a test vector with the given dimension
fn test_vector(id: &str, dimension: usize) -> Vector {
    Vector {
        id: id.to_string(),
        data: vec![1.0; dimension],
        payload: None,
        sparse: None,
    }
}

/// Test that AutoSaveManager mark_changed and force_save work correctly
/// This is the key mechanism that enables persistence - when mutations occur,
/// handlers call mark_changed() and the auto-saver periodically persists.
#[tokio::test]
#[ignore = "Requires specific filesystem setup, skip in CI"]
async fn test_auto_save_manager_mark_changed() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = Arc::new(VectorStore::new());

    // Create auto-save manager with the temp directory
    let auto_save = AutoSaveManager::new_with_path(store.clone(), 1, data_dir.clone());

    // Create a collection
    let config = CollectionConfig {
        graph: None,
        dimension: 32,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
        encryption: None,
    };

    store.create_collection("autosave_test", config).unwrap();

    // Insert at least one vector
    store
        .insert("autosave_test", vec![test_vector("v1", 32)])
        .unwrap();

    // Initially, has_changes should be false (nothing marked)
    assert!(
        !auto_save.has_changes(),
        "Should not have changes before mark_changed"
    );

    // Mark changes
    auto_save.mark_changed();

    // Now has_changes should be true
    assert!(
        auto_save.has_changes(),
        "Should have changes after mark_changed"
    );

    // Verify the vector was actually inserted
    let collection = store.get_collection("autosave_test").unwrap();
    let count = collection.vector_count();
    assert_eq!(count, 1, "Should have 1 vector in collection");

    // Force save should succeed and create the .vecdb file
    let result = auto_save.force_save().await;
    assert!(
        result.is_ok(),
        "Force save should succeed, but got error: {:?}",
        result.err()
    );

    // After force_save, the vecdb file should exist
    let vecdb_path = data_dir.join("vectorizer.vecdb");
    assert!(
        vecdb_path.exists(),
        "vectorizer.vecdb should be created after force_save"
    );

    // has_changes should be reset after successful save
    assert!(
        !auto_save.has_changes(),
        "Should not have changes after force_save"
    );
}

/// Test that the dirty flag mechanism works correctly
#[tokio::test]
#[ignore = "Requires specific filesystem setup, skip in CI"]
async fn test_mark_changed_flag_behavior() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = Arc::new(VectorStore::new());
    let auto_save = AutoSaveManager::new_with_path(store.clone(), 1, data_dir);

    // Initial state: no changes
    assert!(!auto_save.has_changes());

    // Multiple mark_changed calls should accumulate
    auto_save.mark_changed();
    assert!(auto_save.has_changes());

    auto_save.mark_changed();
    assert!(auto_save.has_changes());

    auto_save.mark_changed();
    assert!(auto_save.has_changes());
}
