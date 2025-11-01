//! Test helpers for integration tests
//!
//! Provides reusable utilities for:
//! - Server startup and configuration
//! - Collection creation and management
//! - Vector insertion and data generation
//! - Assertion macros for API responses

use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::time::sleep;
use vectorizer::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector,
};
use vectorizer::server::VectorizerServer;

static TEST_PORT: AtomicU16 = AtomicU16::new(15003);

/// Get next available test port
pub fn next_test_port() -> u16 {
    TEST_PORT.fetch_add(1, Ordering::SeqCst)
}

/// Helper to create a test VectorStore
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
    }
}

/// Create a test collection in the store
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
            }
        })
        .collect()
}

/// Insert test vectors into a collection
pub fn insert_test_vectors(
    store: &VectorStore,
    collection_name: &str,
    vectors: Vec<Vector>,
) -> Result<(), vectorizer::error::VectorizerError> {
    store.insert(collection_name, vectors)
}

/// Create a test server instance (doesn't start HTTP server)
#[allow(dead_code)]
pub async fn create_test_server() -> anyhow::Result<VectorizerServer> {
    VectorizerServer::new().await
}

/// Create a test server and start it on a test port
#[allow(dead_code)]
pub async fn create_and_start_test_server() -> anyhow::Result<(VectorizerServer, u16)> {
    let server = create_test_server().await?;
    let port = next_test_port();
    let server_clone = server.clone();

    tokio::spawn(async move {
        if let Err(e) = server_clone.start("127.0.0.1", port).await {
            eprintln!("Test server failed to start: {e}");
        }
    });

    // Wait for server to be ready
    sleep(Duration::from_millis(500)).await;

    Ok((server, port))
}

/// Wait for a condition to become true (polling helper)
#[allow(dead_code)]
pub async fn wait_for<F>(mut condition: F, timeout_ms: u64) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        sleep(Duration::from_millis(50)).await;
    }

    false
}

/// Assert that a collection exists
#[macro_export]
macro_rules! assert_collection_exists {
    ($store:expr, $name:expr) => {
        assert!(
            $store.list_collections().contains(&$name.to_string()),
            "Collection '{}' should exist",
            $name
        );
    };
}

/// Assert that a collection does not exist
#[macro_export]
macro_rules! assert_collection_not_exists {
    ($store:expr, $name:expr) => {
        assert!(
            !$store.list_collections().contains(&$name.to_string()),
            "Collection '{}' should not exist",
            $name
        );
    };
}

/// Assert that a vector exists in a collection
#[macro_export]
macro_rules! assert_vector_exists {
    ($store:expr, $collection:expr, $vector_id:expr) => {
        let collection = $store
            .get_collection($collection)
            .expect("Collection should exist");
        let vector = collection.get_vector($vector_id);
        assert!(
            vector.is_ok(),
            "Vector '{}' should exist in collection '{}'",
            $vector_id,
            $collection
        );
    };
}

/// Assert successful API response
#[macro_export]
macro_rules! assert_api_success {
    ($response:expr) => {
        assert!(
            $response.status().is_success(),
            "API call should succeed, got status: {}",
            $response.status()
        );
    };
}

/// Assert error API response with status code
#[macro_export]
macro_rules! assert_api_error {
    ($response:expr, $expected_status:expr) => {
        assert_eq!(
            $response.status(),
            $expected_status,
            "API call should return error status {}",
            $expected_status
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_vectors() {
        let vectors = generate_test_vectors(10, 128);
        assert_eq!(vectors.len(), 10);
        assert_eq!(vectors[0].data.len(), 128);
        assert_eq!(vectors[0].id, "vec_0");
    }

    #[test]
    fn test_create_test_collection_config() {
        let config = create_test_collection_config(256);
        assert_eq!(config.dimension, 256);
        assert_eq!(config.metric, DistanceMetric::Cosine);
    }
}
