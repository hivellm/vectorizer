//! Tests for empty collection cleanup functionality
#![allow(clippy::uninlined_format_args, unused_variables)]

use vectorizer::VectorStore;
use vectorizer::models::{CollectionConfig, Vector};

#[test]
fn test_is_collection_empty() {
    let store = VectorStore::new_cpu_only();

    // Create an empty collection
    let config = CollectionConfig::default();
    store.create_collection("empty_collection", config).unwrap();

    // Check that it's empty
    assert!(store.is_collection_empty("empty_collection").unwrap());

    // Add a vector
    let vector = Vector::new("vec1".to_string(), vec![1.0; 512]);
    store.insert("empty_collection", vec![vector]).unwrap();

    // Check that it's no longer empty
    assert!(!store.is_collection_empty("empty_collection").unwrap());
}

#[test]
fn test_list_empty_collections() {
    let store = VectorStore::new_cpu_only();
    let config = CollectionConfig::default();

    // Create multiple collections
    store.create_collection("empty1", config.clone()).unwrap();
    store.create_collection("empty2", config.clone()).unwrap();
    store.create_collection("not_empty", config).unwrap();

    // Add vector to one
    let vector = Vector::new("vec1".to_string(), vec![1.0; 512]);
    store.insert("not_empty", vec![vector]).unwrap();

    // List empty collections
    let empty = store.list_empty_collections();

    assert_eq!(empty.len(), 2);
    assert!(empty.contains(&"empty1".to_string()));
    assert!(empty.contains(&"empty2".to_string()));
    assert!(!empty.contains(&"not_empty".to_string()));
}

#[test]
fn test_cleanup_empty_collections_dry_run() {
    let store = VectorStore::new_cpu_only();
    let config = CollectionConfig::default();

    // Create empty collections with unique names
    store
        .create_collection("dry_run_empty1", config.clone())
        .unwrap();
    store.create_collection("dry_run_empty2", config).unwrap();

    // List only our test collections (store may have preexisting collections)
    let empty = store.list_empty_collections();
    let our_empty: Vec<_> = empty.iter().filter(|c| c.starts_with("dry_run_")).collect();
    assert_eq!(
        our_empty.len(),
        2,
        "Expected 2 dry_run_* collections, got {:?}",
        our_empty
    );

    // Dry run should include at least our 2 empty collections
    let count = store.cleanup_empty_collections(true).unwrap();
    assert!(
        count >= 2,
        "Expected at least 2 empty collections to cleanup, got {}",
        count
    );

    // Collections should still exist after dry run
    let after = store.list_collections();
    assert!(after.contains(&"dry_run_empty1".to_string()));
    assert!(after.contains(&"dry_run_empty2".to_string()));
}

#[test]
fn test_cleanup_empty_collections() {
    let store = VectorStore::new_cpu_only();
    let config = CollectionConfig::default();

    // Count initial collections
    let initial_count = store.list_collections().len();
    let initial_empty_count = store.list_empty_collections().len();

    // Create some empty and non-empty collections with unique names
    store
        .create_collection("cleanup_empty1", config.clone())
        .unwrap();
    store
        .create_collection("cleanup_empty2", config.clone())
        .unwrap();
    store
        .create_collection("cleanup_not_empty", config)
        .unwrap();

    // Add vector to one
    let vector = Vector::new("vec1".to_string(), vec![1.0; 512]);
    store.insert("cleanup_not_empty", vec![vector]).unwrap();

    // We added 2 empty collections
    let empty_before = store.list_empty_collections();
    assert_eq!(
        empty_before.len(),
        initial_empty_count + 2,
        "Expected {} empty collections (initial {} + 2 new), got {}",
        initial_empty_count + 2,
        initial_empty_count,
        empty_before.len()
    );

    // Cleanup should delete empty ones
    let count = store.cleanup_empty_collections(false).unwrap();
    assert_eq!(
        count,
        initial_empty_count + 2,
        "Expected to cleanup {} empty collections",
        initial_empty_count + 2
    );

    // Only non-empty collections should remain
    let collections = store.list_collections();
    assert!(
        collections.contains(&"cleanup_not_empty".to_string()),
        "Non-empty collection should remain"
    );
    assert!(
        !collections.contains(&"cleanup_empty1".to_string()),
        "Empty collection should be deleted"
    );
    assert!(
        !collections.contains(&"cleanup_empty2".to_string()),
        "Empty collection should be deleted"
    );
}

#[test]
fn test_cleanup_with_no_empty_collections() {
    let store = VectorStore::new_cpu_only();
    let config = CollectionConfig::default();

    // First cleanup any existing empty collections
    store.cleanup_empty_collections(false).ok();

    // Count initial non-empty collections
    let initial_count = store.list_collections().len();

    // Create collection with vectors
    store.create_collection("no_cleanup_test", config).unwrap();
    let vector = Vector::new("vec1".to_string(), vec![1.0; 512]);
    store.insert("no_cleanup_test", vec![vector]).unwrap();

    // Cleanup should do nothing (no empty collections)
    let count = store.cleanup_empty_collections(false).unwrap();
    assert_eq!(count, 0, "No empty collections should be cleaned up");

    // Our collection should still exist
    let collections = store.list_collections();
    assert!(collections.contains(&"no_cleanup_test".to_string()));
    assert_eq!(collections.len(), initial_count + 1);
}

#[test]
fn test_is_collection_empty_nonexistent() {
    let store = VectorStore::new_cpu_only();

    // Should error on nonexistent collection
    let result = store.is_collection_empty("nonexistent");
    assert!(result.is_err());
}
