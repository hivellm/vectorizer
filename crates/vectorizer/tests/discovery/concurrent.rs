//! Multiple concurrent `Discovery::discover` calls against the same store.
//!
//! The pipeline holds an `Arc<VectorStore>` and an
//! `Arc<EmbeddingManager>` and is otherwise stateless per call. Running
//! N concurrent `discover` calls against the same `Discovery` instance
//! must:
//!
//! - Never panic or deadlock (no shared mutable state in the pipeline).
//! - Produce structurally equivalent responses for the same query
//!   (collections_searched is deterministic; chunks/bullets may vary
//!   in order but the set must match).
//! - Not corrupt the underlying `VectorStore` (no inserts in this
//!   test — pure read-side concurrency).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use vectorizer::VectorStore;
use vectorizer::discovery::{Discovery, DiscoveryConfig};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_discover_calls_are_safe_and_consistent() {
    let store = Arc::new(VectorStore::new());
    let manager = manager_with_bm25(64);
    for name in ["vectorizer-docs", "vectorizer-source", "vectorizer-tests"] {
        store
            .create_collection(name, collection_config(64))
            .expect("create");
    }

    let discovery = Arc::new(Discovery::new(DiscoveryConfig::default(), store, manager));

    // Spawn 16 concurrent discover calls. The default config excludes
    // `*-tests` so each call should report 2 collections searched.
    let mut handles = Vec::new();
    for _ in 0..16 {
        let d = Arc::clone(&discovery);
        handles.push(tokio::spawn(async move {
            d.discover("vectorizer features").await
        }));
    }

    let mut collections_seen: Vec<usize> = Vec::new();
    for h in handles {
        let response = h.await.expect("task join").expect("discover call");
        collections_seen.push(response.metrics.collections_searched);
    }

    assert!(!collections_seen.is_empty());
    let first = collections_seen[0];
    assert!(
        collections_seen.iter().all(|&c| c == first),
        "concurrent discover calls must report the same collections_searched count; got {collections_seen:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_discover_with_distinct_queries_does_not_interfere() {
    let store = Arc::new(VectorStore::new());
    let manager = manager_with_bm25(64);
    store
        .create_collection("alpha-docs", collection_config(64))
        .expect("create");
    store
        .create_collection("beta-docs", collection_config(64))
        .expect("create");

    let discovery = Arc::new(Discovery::new(DiscoveryConfig::default(), store, manager));

    let queries = ["alpha", "beta", "gamma"];
    let mut handles = Vec::new();
    for q in queries.iter().cycle().take(24) {
        let d = Arc::clone(&discovery);
        let q = q.to_string();
        handles.push(tokio::spawn(async move { d.discover(&q).await }));
    }

    // Every call must complete without error; the actual collection
    // counts depend on the query (alpha/beta match by name, gamma
    // matches none) but no call may panic or hang.
    for h in handles {
        let r = h.await.expect("task join");
        assert!(r.is_ok(), "every concurrent discover call must succeed");
    }
}
