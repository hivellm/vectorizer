//! Persistence tests for Vectorizer

use crate::{
    db::VectorStore,
    models::{CollectionConfig, DistanceMetric, HnswConfig, Vector},
};
use tempfile::tempdir;

#[test]
fn test_persistence_save_load() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("test_persistence", config).unwrap();

    // Insert test vectors
    let test_vectors = vec![
        Vector::new("vec1".to_string(), vec![1.0, 2.0, 3.0]),
        Vector::new("vec2".to_string(), vec![4.0, 5.0, 6.0]),
        Vector::new("vec3".to_string(), vec![7.0, 8.0, 9.0]),
    ];
    store.insert("test_persistence", test_vectors).unwrap();

    // Save to file
    let temp_dir = tempdir().unwrap();
    let save_path = temp_dir.path().join("persistence_test.vdb");
    store.save(&save_path).unwrap();

    // Load from file
    let loaded_store = VectorStore::load(&save_path).unwrap();

    // Verify vectors were actually saved and loaded
    let vec1 = loaded_store.get_vector("test_persistence", "vec1").unwrap();
    assert_eq!(vec1.data, vec![1.0, 2.0, 3.0]);

    let vec2 = loaded_store.get_vector("test_persistence", "vec2").unwrap();
    assert_eq!(vec2.data, vec![4.0, 5.0, 6.0]);

    let vec3 = loaded_store.get_vector("test_persistence", "vec3").unwrap();
    assert_eq!(vec3.data, vec![7.0, 8.0, 9.0]);

    // Verify collection metadata
    let metadata = loaded_store
        .get_collection_metadata("test_persistence")
        .unwrap();
    assert_eq!(metadata.vector_count, 3);
}
