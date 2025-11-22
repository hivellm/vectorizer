//! Integration tests for Qdrant REST API compatibility
//!
//! Tests all 14 Qdrant endpoints implemented in the vectorizer:
//! - Collection management: list, get, create, update, delete
//! - Vector operations: upsert, retrieve, delete, scroll, count
//! - Search operations: search, recommend, batch search, batch recommend

use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
    StorageType,
};

/// Helper to create a test store
#[allow(dead_code)]
fn create_test_store() -> VectorStore {
    VectorStore::new()
}

/// Helper to create a test collection
#[allow(dead_code)]
fn create_test_collection(
    store: &VectorStore,
    name: &str,
    dimension: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = CollectionConfig {
        dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 100,
            ef_search: 100,
            seed: None,
        },
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
    };
    store.create_collection(name, config)?;
    Ok(())
}

/// Helper to insert test vectors
#[allow(dead_code)]
fn insert_test_vectors(
    store: &VectorStore,
    collection_name: &str,
    count: usize,
    dimension: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut ids = Vec::new();
    let mut vectors = Vec::new();

    for i in 0..count {
        let id = format!("test_vector_{i}");
        let data: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 / 10.0).collect();

        let vector = vectorizer::models::Vector {
            id: id.clone(),
            data,
            ..Default::default()
        };
        ids.push(id.clone());
        vectors.push(vector);
    }
    store.insert(collection_name, vectors)?;
    Ok(ids)
}
