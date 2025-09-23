//! Database integration tests for Vectorizer

use crate::{
    db::VectorStore,
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
};

#[test]
fn test_vector_store_stats_multiple_collections() {
    let store = VectorStore::new();

    let cfg_small = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    let cfg_large = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };

    store.create_collection("small", cfg_small).unwrap();
    store.create_collection("large", cfg_large).unwrap();

    // Insert a few vectors in each
    store
        .insert(
            "small",
            vec![
                Vector::new("s1".to_string(), vec![1.0, 0.0, 0.0]),
                Vector::new("s2".to_string(), vec![0.0, 1.0, 0.0]),
            ],
        )
        .unwrap();

    store
        .insert(
            "large",
            vec![
                Vector::new("l1".to_string(), vec![0.1; 64]),
                Vector::new("l2".to_string(), vec![0.2; 64]),
                Vector::new("l3".to_string(), vec![0.3; 64]),
            ],
        )
        .unwrap();

    let stats = store.stats();
    assert_eq!(stats.collection_count, 2);
    assert_eq!(stats.total_vectors, 5);
    assert!(stats.total_memory_bytes > 0);
}

#[test]
fn test_payload_serialization_nested() {
    let store = VectorStore::new();
    let cfg = CollectionConfig {
        dimension: 4,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("nested", cfg).unwrap();

    let payload = Payload::from_value(serde_json::json!({
        "meta": {
            "source": "unit_test",
            "tags": ["nested", "json", {"k": "v"}],
            "score": 0.87
        }
    }))
    .unwrap();

    store
        .insert(
            "nested",
            vec![Vector::with_payload(
                "n1".to_string(),
                vec![0.0, 1.0, 2.0, 3.0],
                payload,
            )],
        )
        .unwrap();

    let got = store.get_vector("nested", "n1").unwrap();
    let meta = &got.payload.unwrap().data["meta"];
    assert_eq!(meta["source"], "unit_test");
    assert_eq!(meta["tags"][0], "nested");
    assert_eq!(meta["tags"][2]["k"], "v");
}
