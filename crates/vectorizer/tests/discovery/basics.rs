//! Happy-path Discovery pipeline integration test.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use vectorizer::VectorStore;
use vectorizer::discovery::{Discovery, DiscoveryConfig};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

/// Build an `EmbeddingManager` with BM25 wired as the default provider —
/// the same pattern `MCPToolHandler::new_with_store` uses, so this test
/// fixture exercises the same plumbing the live MCP / RPC paths do.
fn manager_with_bm25(dim: usize) -> Arc<EmbeddingManager> {
    let mut manager = EmbeddingManager::new();
    manager.register_provider("bm25".to_string(), Box::new(Bm25Embedding::new(dim)));
    manager
        .set_default_provider("bm25")
        .expect("bm25 provider just registered");
    Arc::new(manager)
}

fn collection_config(dim: usize) -> CollectionConfig {
    CollectionConfig {
        dimension: dim,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: None,
        encryption: None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn discover_against_empty_store_returns_empty_response() {
    let store = Arc::new(VectorStore::new());
    let manager = manager_with_bm25(64);

    let discovery = Discovery::new(DiscoveryConfig::default(), store, manager);
    let response = discovery.discover("anything").await.expect("discover");

    // No collections → no chunks. Pipeline still completes and populates
    // the metrics struct, which is the contract callers rely on.
    assert!(response.chunks.is_empty(), "no collections → no chunks");
    assert!(response.bullets.is_empty(), "no chunks → no bullets");
    assert_eq!(response.metrics.collections_searched, 0);
    assert_eq!(response.metrics.chunks_found, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn discover_with_one_indexed_collection_populates_metrics() {
    let store = Arc::new(VectorStore::new());
    let manager = manager_with_bm25(64);
    store
        .create_collection("vectorizer-docs", collection_config(64))
        .expect("create collection");

    let discovery = Discovery::new(DiscoveryConfig::default(), store, manager);
    let response = discovery
        .discover("vectorizer features")
        .await
        .expect("discover");

    // The collection name contains a query term ("vectorizer"), so the
    // filter step lets it through. The store has zero vectors, so the
    // search step returns empty — the pipeline doesn't crash on an
    // empty collection and reports it correctly in metrics.
    assert_eq!(
        response.metrics.collections_searched, 1,
        "collection-name match should keep the single collection in scope"
    );
    assert!(
        response.metrics.queries_generated >= 1,
        "expand_queries should produce at least the original query"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn discover_excludes_blacklisted_collection_names() {
    let store = Arc::new(VectorStore::new());
    let manager = manager_with_bm25(64);
    // Default DiscoveryConfig excludes `*-test` and `*-backup`. Create
    // both a candidate and a blacklisted collection with names that
    // match the query so only the exclude-list filtering separates
    // them.
    store
        .create_collection("vectorizer-docs", collection_config(64))
        .expect("create candidate");
    store
        .create_collection("vectorizer-test", collection_config(64))
        .expect("create blacklisted");
    store
        .create_collection("vectorizer-backup", collection_config(64))
        .expect("create blacklisted backup");

    let discovery = Discovery::new(DiscoveryConfig::default(), store, manager);
    let response = discovery.discover("vectorizer").await.expect("discover");

    assert_eq!(
        response.metrics.collections_searched, 1,
        "default exclude list (*-test, *-backup) must drop the two blacklisted collections"
    );
}
