//! Tests for discovery system
//!
//! Note: Integration tests for Discovery pipeline are in tests/discovery_integration.rs
//! Unit tests here focus on individual helper functions

#[cfg(test)]
mod unit_tests {
    use crate::discovery::*;

    #[test]
    fn test_filter_and_expand_functions() {
        // Test filter_collections with empty inputs
        let collections = vec![];
        let filtered = filter_collections("test query", &[], &[], &collections);
        assert!(filtered.is_ok());

        // Test expand_queries_baseline
        let config = ExpansionConfig::default();
        let queries = expand_queries_baseline("test query", &config);
        assert!(queries.is_ok());
        assert!(!queries.unwrap().is_empty());
    }
}
