//! Test helpers for integration tests
//!
//! Provides reusable utilities for:
//! - Server startup and configuration
//! - Collection creation and management
//! - Vector insertion and data generation
//! - Assertion macros for API responses

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use vectorizer::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
    StorageType, Vector,
};

#[allow(dead_code)]
static TEST_PORT: AtomicU16 = AtomicU16::new(15003);

/// Get next available test port
#[allow(dead_code)]
pub fn next_test_port() -> u16 {
    TEST_PORT.fetch_add(1, Ordering::SeqCst)
}

/// Helper to create a test VectorStore
#[allow(dead_code)]
pub fn create_test_store() -> Arc<VectorStore> {
    Arc::new(VectorStore::new())
}

/// Helper to create a test EmbeddingManager with BM25
#[allow(dead_code)]
pub fn create_test_embedding_manager() -> anyhow::Result<EmbeddingManager> {
    let mut manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25")?;
    Ok(manager)
}

/// Helper to create a test collection with default config
#[allow(dead_code)]
pub fn create_test_collection_config(dimension: usize) -> CollectionConfig {
    CollectionConfig {
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
    }
}

/// Create a test collection in the store
#[allow(dead_code)]
pub fn create_test_collection(
    store: &VectorStore,
    name: &str,
    dimension: usize,
) -> Result<(), vectorizer::error::VectorizerError> {
    let config = create_test_collection_config(dimension);
    store.create_collection(name, config)
}

/// Create a test collection with custom config
#[allow(dead_code)]
pub fn create_test_collection_with_config(
    store: &VectorStore,
    name: &str,
    config: CollectionConfig,
) -> Result<(), vectorizer::error::VectorizerError> {
    store.create_collection(name, config)
}

/// Generate test vectors with specified count and dimension
#[allow(dead_code)]
pub fn generate_test_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| {
            let mut data = vec![0.0; dimension];
            // Fill with some pattern to make vectors unique
            for (j, item) in data.iter_mut().enumerate().take(dimension) {
                *item = (i * dimension + j) as f32 * 0.001;
            }
            // Normalize
            let norm: f32 = data.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut data {
                    *x /= norm;
                }
            }
            let payload_value = serde_json::json!({
                "index": i,
                "text": format!("Test vector {i}"),
            });
            Vector {
                id: format!("vec_{i}"),
                data,
                payload: Some(vectorizer::models::Payload::new(payload_value)),
                ..Default::default()
            }
        })
        .collect()
}

/// Insert test vectors into a collection
#[allow(dead_code)]
pub fn insert_test_vectors(
    store: &VectorStore,
    collection_name: &str,
    vectors: Vec<Vector>,
) -> Result<(), vectorizer::error::VectorizerError> {
    store.insert(collection_name, vectors)
}

/// Assert that a collection exists
#[allow(unused_macros)] // May be unused in some test files
macro_rules! assert_collection_exists {
    ($store:expr, $name:expr) => {
        assert!(
            $store.list_collections().contains(&$name.to_string()),
            "Collection '{}' should exist",
            $name
        );
    };
}

/// Assert that a vector exists in a collection
#[allow(unused_macros)] // May be unused in some test files
macro_rules! assert_vector_exists {
    ($store:expr, $collection:expr, $id:expr) => {
        assert!(
            $store.get_vector($collection, $id).is_ok(),
            "Vector '{}' should exist in collection '{}'",
            $id,
            $collection
        );
    };
}
