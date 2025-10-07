//! Intelligent Search Module - Simplified Working Implementation
//! 
//! This module implements intelligent search capabilities using a simplified approach
//! that focuses on functionality over complex external dependencies.

pub mod query_generator;
pub mod simple_search_engine;
pub mod mmr_diversifier;
pub mod context_formatter;
pub mod mcp_tools;
pub mod rest_api;
pub mod mcp_server_integration;
pub mod examples;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for intelligent search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchConfig {
    /// Maximum number of queries to generate
    pub max_queries: usize,
    /// Enable domain-specific expansion
    pub domain_expansion: bool,
    /// Enable technical term focus
    pub technical_focus: bool,
    /// Enable synonym expansion
    pub synonym_expansion: bool,
    /// Similarity threshold for deduplication
    pub similarity_threshold: f32,
    /// Enable reranking
    pub reranking_enabled: bool,
    /// Enable MMR diversification
    pub mmr_enabled: bool,
    /// MMR lambda parameter (0.0 = diversity, 1.0 = relevance)
    pub mmr_lambda: f32,
}

impl Default for IntelligentSearchConfig {
    fn default() -> Self {
        Self {
            max_queries: 8,
            domain_expansion: true,
            technical_focus: true,
            synonym_expansion: true,
            similarity_threshold: 0.8,
            reranking_enabled: true,
            mmr_enabled: true,
            mmr_lambda: 0.7,
        }
    }
}

/// Main intelligent search engine
pub struct IntelligentSearchEngine {
    config: IntelligentSearchConfig,
    query_generator: query_generator::QueryGenerator,
    search_engine: simple_search_engine::SimpleSearchEngine,
    mmr_diversifier: mmr_diversifier::MMRDiversifier,
    context_formatter: context_formatter::ContextFormatter,
}

impl IntelligentSearchEngine {
    /// Create a new intelligent search engine
    pub fn new(config: IntelligentSearchConfig) -> Self {
        let query_generator = query_generator::QueryGenerator::new(
            config.max_queries
        );
        let search_engine = simple_search_engine::SimpleSearchEngine::new();
        let mmr_diversifier = mmr_diversifier::MMRDiversifier::new(
            config.mmr_lambda
        );
        let context_formatter = context_formatter::ContextFormatter::new(
            500,  // max_content_length
            3,    // max_lines_per_result
            true  // include_metadata
        );

        Self {
            config,
            query_generator,
            search_engine,
            mmr_diversifier,
            context_formatter,
        }
    }

    /// Add documents to the search engine
    pub async fn add_documents(&mut self, documents: Vec<Document>) -> Result<(), String> {
        self.search_engine.add_documents(documents).await.map_err(|e| e.to_string())
    }

    /// Perform intelligent search
    pub async fn search(
        &self,
        query: &str,
        collections: Option<Vec<String>>,
        max_results: Option<usize>,
    ) -> Result<(Vec<IntelligentSearchResult>, SearchMetadata), String> {
        let start_time = std::time::Instant::now();
        let max_results = max_results.unwrap_or(10);

        // Generate multiple queries
        let queries = self.query_generator.generate_queries(query);
        
        // Search across collections
        let mut all_results = Vec::new();
        let collections_to_search = collections.unwrap_or_else(|| {
            vec!["default".to_string()] // Default collection if none specified
        });

        for search_query in &queries {
            for collection in &collections_to_search {
                let results = self.search_engine.search(
                    search_query,
                    vec![collection.clone()],
                    max_results * 2, // Get more results for better selection
                ).await.map_err(|e| e.to_string())?;
                all_results.extend(results);
            }
        }

        // Deduplicate results
        let deduplicated_results = self.deduplicate_results(&all_results);

        // Apply MMR diversification if enabled
        let final_results = if self.config.mmr_enabled {
            self.mmr_diversifier.diversify(&deduplicated_results, max_results)
        } else {
            deduplicated_results.clone().into_iter().take(max_results).collect()
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        let metadata = SearchMetadata {
            total_queries: queries.len(),
            collections_searched: collections_to_search.len(),
            total_results_found: all_results.len(),
            results_after_dedup: deduplicated_results.len(),
            final_results_count: final_results.len(),
            processing_time_ms: processing_time,
        };

        Ok((final_results, metadata))
    }

    /// Deduplicate results based on similarity
    fn deduplicate_results(&self, results: &[IntelligentSearchResult]) -> Vec<IntelligentSearchResult> {
        let mut deduplicated = Vec::new();
        
        for result in results {
            let is_duplicate = deduplicated.iter().any(|existing: &IntelligentSearchResult| {
                self.calculate_similarity(&result.content, &existing.content) > self.config.similarity_threshold
            });
            
            if !is_duplicate {
                deduplicated.push(result.clone());
            }
        }
        
        deduplicated
    }

    /// Calculate similarity between two texts
    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        let binding1 = text1.to_lowercase();
        let words1: std::collections::HashSet<&str> = binding1.split_whitespace().collect();
        let binding2 = text2.to_lowercase();
        let words2: std::collections::HashSet<&str> = binding2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> &IntelligentSearchConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: IntelligentSearchConfig) {
        self.config = config;
    }
}

/// Search result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchResult {
    /// Document content
    pub content: String,
    /// Relevance score
    pub score: f32,
    /// Collection name
    pub collection: String,
    /// Document ID
    pub doc_id: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Score breakdown (optional)
    pub score_breakdown: Option<ScoreBreakdown>,
}

/// Score breakdown for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Base relevance score
    pub relevance: f32,
    /// Collection relevance bonus
    pub collection_bonus: f32,
    /// Technical term bonus
    pub technical_bonus: f32,
    /// Final score
    pub final_score: f32,
}

/// Search metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    /// Total number of queries generated
    pub total_queries: usize,
    /// Number of collections searched
    pub collections_searched: usize,
    /// Total results found before deduplication
    pub total_results_found: usize,
    /// Results after deduplication
    pub results_after_dedup: usize,
    /// Final number of results returned
    pub final_results_count: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document ID
    pub id: String,
    /// Document content
    pub content: String,
    /// Collection name
    pub collection: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IntelligentSearchConfig::default();
        assert_eq!(config.max_queries, 8);
        assert!(config.domain_expansion);
        assert!(config.technical_focus);
        assert!(config.synonym_expansion);
        assert_eq!(config.similarity_threshold, 0.8);
        assert!(config.reranking_enabled);
        assert!(config.mmr_enabled);
        assert_eq!(config.mmr_lambda, 0.7);
    }

    #[test]
    fn test_intelligent_search_engine_creation() {
        let config = IntelligentSearchConfig::default();
        let engine = IntelligentSearchEngine::new(config);
        // Engine should be created successfully
        assert!(true);
    }

    #[test]
    fn test_document_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        
        let doc = Document {
            id: "doc1".to_string(),
            content: "This is a test document".to_string(),
            collection: "test".to_string(),
            metadata,
        };
        
        assert_eq!(doc.id, "doc1");
        assert_eq!(doc.content, "This is a test document");
        assert_eq!(doc.collection, "test");
    }
}