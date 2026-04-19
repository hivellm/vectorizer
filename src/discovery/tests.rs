//! Tests for discovery system.

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod unit_tests {
    use std::sync::Arc;

    use crate::VectorStore;
    use crate::discovery::*;
    use crate::embedding::EmbeddingManager;

    #[test]
    fn test_filter_score_expand() {
        let collections = vec![];

        let filtered = filter_collections("test query", &[], &[], &collections);
        assert!(filtered.is_ok());

        let config = ExpansionConfig::default();
        let queries = expand_queries_baseline("test query", &config);
        assert!(queries.is_ok());
        assert!(!queries.unwrap().is_empty());
    }

    /// Integration test restored after the `Discovery::new` signature change.
    /// The pipeline runs against an empty `VectorStore` + a minimally
    /// configured `EmbeddingManager` (BM25 as the default provider —
    /// same pattern as `MCPToolHandler::new_with_store`). There are no
    /// collections to search, so the discovery call returns an empty
    /// result set but populates the metrics shape.
    #[tokio::test]
    async fn discovery_pipeline_runs_against_empty_store() {
        let config = DiscoveryConfig::default();
        let store = Arc::new(VectorStore::new());

        let mut manager = EmbeddingManager::new();
        let bm25 = crate::embedding::Bm25Embedding::new(512);
        manager.register_provider("bm25".to_string(), Box::new(bm25));
        manager
            .set_default_provider("bm25")
            .expect("bm25 registered");
        let embedding_manager = Arc::new(manager);

        let discovery = Discovery::new(config, store, embedding_manager);

        let response = discovery
            .discover("what is vectorizer")
            .await
            .expect("discover call");

        // With no collections indexed, the pipeline still returns — just
        // with an empty result set — and the metrics struct is populated.
        assert!(response.chunks.is_empty());
        // Per DiscoveryMetrics: total_time_ms is a u64; may be 0 on a very
        // fast run, so we only assert the field exists / pipeline didn't
        // short-circuit before metrics population.
        let _ = response.metrics.total_time_ms;
    }
}
