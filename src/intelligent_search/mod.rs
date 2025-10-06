//! Intelligent Search Module - Simplified Working Implementation
//! 
//! This module implements intelligent search capabilities using a simplified approach
//! that focuses on functionality over complex external dependencies.

pub mod query_generator;
pub mod simple_search_engine;
pub mod mmr_diversifier;
pub mod context_formatter;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for intelligent search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchConfig {
    /// Maximum number of queries to generate
    pub max_queries: usize,
    /// Enable domain-specific expansion
    pub domain_expansion: bool,
    /// Enable technical term extraction
    pub technical_focus: bool,
    /// Enable synonym expansion
    pub synonym_expansion: bool,
    /// Similarity threshold for deduplication
    pub similarity_threshold: f32,
    /// Enable semantic reranking
    pub reranking_enabled: bool,
    /// Enable MMR diversification
    pub mmr_enabled: bool,
    /// MMR lambda parameter (0.0 = pure diversity, 1.0 = pure relevance)
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
            mmr_lambda: 0.7, // Balance between relevance and diversity
        }
    }
}

/// Search result with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchResult {
    /// The content of the result
    pub content: String,
    /// Relevance score
    pub score: f32,
    /// Collection name
    pub collection: String,
    /// Document ID
    pub doc_id: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Score breakdown for debugging
    pub score_breakdown: Option<ScoreBreakdown>,
}

/// Detailed score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Text similarity score
    pub text_similarity: f32,
    /// Term frequency score
    pub term_frequency: f32,
    /// Collection relevance score
    pub collection_relevance: f32,
    /// Final combined score
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
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Simple intelligent search engine
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
            config.max_queries,
        );
        
        let search_engine = simple_search_engine::SimpleSearchEngine::new();
        
        let mmr_diversifier = mmr_diversifier::MMRDiversifier::new(
            config.mmr_lambda,
        );
        
        let context_formatter = context_formatter::ContextFormatter::new(
            400, // max content length
            5,   // max lines per result
            false, // include metadata
        );

        Self {
            config,
            query_generator,
            search_engine,
            mmr_diversifier,
            context_formatter,
        }
    }

    /// Perform intelligent search
    pub async fn search(
        &self,
        query: &str,
        collections: Option<Vec<String>>,
        max_results: Option<usize>,
    ) -> Result<(Vec<IntelligentSearchResult>, SearchMetadata), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Step 1: Generate multiple queries
        let queries = self.query_generator.generate_queries(query);
        
        // Step 2: Search using simple engine
        let mut all_results = Vec::new();
        let collections_to_search = collections.unwrap_or_else(|| vec!["default".to_string()]);
        
        for search_query in &queries {
            let results = self.search_engine.search(
                search_query,
                collections_to_search.clone(),
                max_results.unwrap_or(10),
            ).await?;
            all_results.extend(results);
        }
        
        // Step 3: Deduplication
        let deduplicated_results = self.deduplicate_results(all_results.clone());
        
        // Step 4: MMR diversification (if enabled)
        let diversified_results = if self.config.mmr_enabled {
            self.mmr_diversifier.diversify(&deduplicated_results, max_results.unwrap_or(5))
        } else {
            deduplicated_results.clone().into_iter().take(max_results.unwrap_or(5)).collect()
        };
        
        // Step 5: Create metadata
        let metadata = SearchMetadata {
            total_queries: queries.len(),
            collections_searched: collections_to_search.len(),
            total_results_found: all_results.len(),
            results_after_dedup: deduplicated_results.len(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };
        
        Ok((diversified_results, metadata))
    }

    /// Add documents to the search index
    pub async fn add_documents(
        &mut self,
        documents: Vec<Document>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.search_engine.add_documents(documents).await?;
        Ok(())
    }

    /// Simple deduplication based on content similarity
    fn deduplicate_results(&self, results: Vec<IntelligentSearchResult>) -> Vec<IntelligentSearchResult> {
        let mut unique_results = Vec::new();
        
        for result in results {
            let is_duplicate = unique_results.iter().any(|existing: &IntelligentSearchResult| {
                self.calculate_similarity(&result.content, &existing.content) > self.config.similarity_threshold
            });
            
            if !is_duplicate {
                unique_results.push(result);
            }
        }
        
        unique_results
    }

    /// Calculate simple text similarity
    fn calculate_similarity(&self, content1: &str, content2: &str) -> f32 {
        let content1_lower = content1.to_lowercase();
        let words1: std::collections::HashSet<&str> = content1_lower.split_whitespace().collect();
        let content2_lower = content2.to_lowercase();
        let words2: std::collections::HashSet<&str> = content2_lower.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Get configuration
    pub fn get_config(&self) -> &IntelligentSearchConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: IntelligentSearchConfig) {
        self.config = config;
    }
}

/// Document structure for indexing
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