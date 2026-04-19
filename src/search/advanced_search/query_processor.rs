//! Query normalization, stop-word removal, stemming, synonym expansion.
//!
//! [`QueryProcessor`] is consumed by the orchestrator in
//! [`super::engine`]; it converts a raw [`SearchQuery`] into a
//! [`ProcessedQuery`] that the index and ranker can work with.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::types::*;

/// Processed query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedQuery {
    /// Processed query text
    pub query: String,

    /// Query terms
    pub terms: Vec<String>,

    /// Query filters
    pub filters: HashMap<String, serde_json::Value>,
}

impl QueryProcessor {
    /// Create new query processor
    pub(super) fn new(config: QueryProcessingConfig) -> Self {
        Self {
            config,
            stop_words: HashSet::new(),
            synonyms: HashMap::new(),
        }
    }

    /// Process search query
    pub(super) async fn process_query(&self, query: &SearchQuery) -> Result<ProcessedQuery> {
        let mut processed_query = ProcessedQuery {
            query: query.query.clone(),
            terms: Vec::new(),
            filters: query.filters.clone(),
        };

        // Normalize query
        if self.config.enable_query_normalization {
            processed_query.query = self.normalize_query(&processed_query.query);
        }

        // Remove stop words
        if self.config.enable_stop_word_removal {
            processed_query.query = self.remove_stop_words(&processed_query.query);
        }

        // Extract terms
        processed_query.terms = self.extract_terms(&processed_query.query);

        // Apply stemming
        if self.config.enable_stemming {
            processed_query.terms = self.apply_stemming(&processed_query.terms);
        }

        // Expand synonyms
        if self.config.enable_synonym_expansion {
            processed_query.terms = self.expand_synonyms(&processed_query.terms);
        }

        Ok(processed_query)
    }

    /// Normalize query
    fn normalize_query(&self, query: &str) -> String {
        query.to_lowercase()
    }

    /// Remove stop words
    fn remove_stop_words(&self, query: &str) -> String {
        query
            .split_whitespace()
            .filter(|word| !self.stop_words.contains(*word))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Extract terms
    fn extract_terms(&self, query: &str) -> Vec<String> {
        query.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Apply stemming
    fn apply_stemming(&self, terms: &[String]) -> Vec<String> {
        // Simplified stemming - in practice would use proper stemming algorithm
        terms.to_vec()
    }

    /// Expand synonyms
    fn expand_synonyms(&self, terms: &[String]) -> Vec<String> {
        let mut expanded_terms = terms.to_vec();

        for term in terms {
            if let Some(synonyms) = self.synonyms.get(term) {
                expanded_terms.extend(synonyms.clone());
            }
        }

        expanded_terms
    }
}
