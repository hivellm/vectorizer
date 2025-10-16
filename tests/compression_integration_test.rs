//! Integration tests for the compression system

use vectorizer::{
    VectorStore,
    models::{CollectionConfig, DistanceMetric, HnswConfig, Vector},
    storage::{detect_format, StorageFormat, StorageReader, StorageCompactor, vecdb_path},
};
use tempfile::TempDir;
use std::path::PathBuf;

/// Helper to create a test data directory
fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Helper to create a test collection with vectors
fn create_test_collection(store: &VectorStore, name: &str, vector_count: usize) {
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: vectorizer::models::QuantizationConfig::default(),
        compression: Default::default(),
        normalization: None,
    };
    
    store.create_collection(name, config).expect("Failed to create collection");
    
    // Add test vectors
    let vectors: Vec<Vector> = (0..vector_count)
        .map(|i| Vector {
            id: format!("vec_{}", i),
            data: vec![i as f32 / vector_count as f32; 128],
            payload: None,
        })
        .collect();
    
    store.insert(name, vectors).expect("Failed to insert vectors");
}

#[test]
fn test_first_load_and_compact() {
    // Create a temporary directory for the test
    let temp_dir = setup_test_dir();
    
    // Mock the data directory by setting it temporarily
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    
    let store = VectorStore::new();
    
    // Create test collections
    create_test_collection(&store, "test-collection-1", 10);
    create_test_collection(&store, "test-collection-2", 15);
    
    // Save collections (simulates raw file creation)
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    
    // Manually save collections to raw format
    for collection_name in store.list_collections() {
        let path = data_dir.join(format!("{}_vector_store.bin", collection_name));
        // Simulate saving (in real scenario, auto-save would do this)
        std::fs::write(&path, b"test data").expect("Failed to write test file");
    }
    
    // Now compact to .vecdb
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    let index = compactor.compact_all_with_cleanup(true).expect("Compaction failed");
    
    // Verify .vecdb was created
    assert!(vecdb_path(&data_dir).exists(), "vectorizer.vecdb should exist");
    
    // Verify format is now Compact
    assert_eq!(detect_format(&data_dir), StorageFormat::Compact);
    
    // Verify raw files were removed
    let raw_files: Vec<_> = std::fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_str()
                .map(|s| s.ends_with("_vector_store.bin"))
                .unwrap_or(false)
        })
        .collect();
    
    assert_eq!(raw_files.len(), 0, "Raw files should be removed after compaction");
    
    // Verify index
    assert!(index.collection_count() >= 2, "Should have at least 2 collections");
}

#[test]
fn test_load_from_vecdb() {
    let temp_dir = setup_test_dir();
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    
    // First, create collections in a store
    let store1 = VectorStore::new();
    create_test_collection(&store1, "persistent-collection", 20);
    
    // Use compact_from_memory to properly serialize the collection
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_from_memory(&store1).expect("Compaction from memory failed");
    
    // Verify .vecdb exists
    assert!(vecdb_path(&data_dir).exists(), "vectorizer.vecdb should exist");
    
    // Now create a new VectorStore and load from .vecdb
    let store2 = VectorStore::new();
    
    // Verify we can read from .vecdb
    let reader = StorageReader::new(&data_dir).expect("Failed to create reader");
    let collections = reader.list_collections().expect("Failed to list collections");
    
    // We should be able to extract collections in memory
    let extracted = reader.extract_all_collections().expect("Failed to extract");
    
    assert!(extracted.len() > 0, "Should have extracted collections from .vecdb");
    assert!(collections.contains(&"persistent-collection".to_string()), "Should have persistent-collection");
}

#[test]
fn test_compact_if_changed() {
    let temp_dir = setup_test_dir();
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    
    // Create initial .vecdb
    let raw_file = data_dir.join("test_vector_store.bin");
    std::fs::write(&raw_file, b"initial data").expect("Failed to write");
    
    let mut compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_all().expect("Initial compaction failed");
    
    // First check - should not compact (no changes)
    let result1 = compactor.compact_if_changed().expect("Check failed");
    assert!(result1.is_none(), "Should not compact when no changes");
    
    // Modify a raw file (simulate new data)
    std::thread::sleep(std::time::Duration::from_millis(100));
    std::fs::write(&raw_file, b"modified data").expect("Failed to modify");
    
    // Second check - should compact (changes detected)
    let result2 = compactor.compact_if_changed().expect("Check failed");
    assert!(result2.is_some(), "Should compact when changes detected");
}

#[test]
fn test_error_recovery() {
    let temp_dir = setup_test_dir();
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    
    // Create a valid .vecdb first with actual data
    let raw_file = data_dir.join("test_vector_store.bin");
    std::fs::write(&raw_file, b"valid data").expect("Failed to write");
    
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_all().expect("Initial compaction failed");
    
    let vecdb = vecdb_path(&data_dir);
    let original_vecdb = std::fs::read(&vecdb).expect("Failed to read .vecdb");
    let original_size = original_vecdb.len();
    
    // Remove the raw file - this means the next compact will create an empty archive
    std::fs::remove_file(&raw_file).ok();
    
    // Try to compact with no raw files - will create empty archive
    let result = compactor.compact_all();
    assert!(result.is_ok(), "Compaction should succeed even with no files");
    
    // Verify .vecdb still exists (but may be empty)
    assert!(vecdb.exists(), ".vecdb should still exist");
    
    // The new .vecdb will be an empty ZIP archive, which is different from the original
    // This is correct behavior - the system can handle empty archives
    let current_vecdb = std::fs::read(&vecdb).expect(".vecdb should still exist");
    
    // Both should be valid ZIP files (original non-empty, current empty)
    assert!(original_size > 22, "Original should be non-empty ZIP");
    assert!(current_vecdb.len() >= 22, "Current should be valid (possibly empty) ZIP");
}

#[test]
fn test_atomic_update() {
    let temp_dir = setup_test_dir();
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    
    // Create initial data
    let raw_file = data_dir.join("test_vector_store.bin");
    std::fs::write(&raw_file, b"atomic test data").expect("Failed to write");
    
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    
    // Compact multiple times
    for i in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&raw_file, format!("data version {}", i)).expect("Failed to write");
        compactor.compact_all().expect("Compaction failed");
        
        // Verify .vecdb exists and is valid
        let vecdb = vecdb_path(&data_dir);
        assert!(vecdb.exists(), ".vecdb should exist after each compaction");
        
        // Verify we can read it
        let reader = StorageReader::new(&data_dir).expect("Should be able to read after compaction");
        assert!(reader.list_collections().is_ok(), "Should be able to list collections");
    }
}



