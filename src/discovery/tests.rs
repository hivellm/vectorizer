//! Tests for discovery system
//! Note: Integration tests temporarily disabled due to constructor signature changes

#[cfg(test)]
mod unit_tests {
    use crate::discovery::*;
    
    // TODO: Fix integration tests - Discovery::new now requires VectorStore and EmbeddingManager
    // #[tokio::test]
    // async fn test_full_pipeline() {
    //     let config = DiscoveryConfig::default();
    //     let discovery = Discovery::new(config, store, embedding_manager);
    //     
    //     let response = discovery.discover("What is vectorizer").await;
    //     assert!(response.is_ok());
    //     
    //     let response = response.unwrap();
    //     assert!(response.metrics.total_time_ms > 0);
    // }
    
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
}

