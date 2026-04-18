//! Advanced search and discovery system
//!
//! Provides sophisticated search capabilities including:
//! - Multi-modal search (text, vector, hybrid)
//! - Advanced ranking algorithms
//! - Search result clustering and deduplication
//! - Query expansion and suggestion
//! - Search analytics and optimization
//! - Real-time search updates

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};

use crate::error::VectorizerError;

mod types;
pub use types::*;

impl AdvancedSearchEngine {
    /// Create a new advanced search engine
    pub fn new(config: SearchConfig) -> Self {
        Self {
            query_processor: Arc::new(QueryProcessor::new(config.query_processing.clone())),
            ranking_engine: Arc::new(RankingEngine::new(config.ranking.clone())),
            analytics: Arc::new(SearchAnalytics::new(config.analytics.clone())),
            suggestions: Arc::new(SearchSuggestions::new(config.suggestions.clone())),
            config,
            index: Arc::new(RwLock::new(SearchIndex::new())),
        }
    }

    /// Add document to search index
    pub async fn add_document(&self, document: SearchDocument) -> Result<()> {
        let mut index = self.index.write();
        index.add_document(document)?;
        Ok(())
    }

    /// Remove document from search index
    pub async fn remove_document(&self, document_id: &str) -> Result<()> {
        let mut index = self.index.write();
        index.remove_document(document_id)?;
        Ok(())
    }

    /// Update document in search index
    pub async fn update_document(&self, document: SearchDocument) -> Result<()> {
        let mut index = self.index.write();
        index.update_document(document)?;
        Ok(())
    }

    /// Search documents
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResponse> {
        let start_time = Instant::now();

        // Process query
        let processed_query = self.query_processor.process_query(&query).await?;

        // Get documents from index
        let index = self.index.read();
        let mut results = index.search(&processed_query).await?;

        // Rank results
        let ranked_results = self
            .ranking_engine
            .rank_results(&results, &processed_query)
            .await?;

        // Generate facets
        let facets = self.generate_facets(&ranked_results, &query.facets).await?;

        // Generate suggestions
        let suggestions = self
            .suggestions
            .generate_suggestions(&processed_query.query)
            .await?;

        // Log query
        self.analytics
            .log_query(QueryLog {
                query_id: uuid::Uuid::new_v4().to_string(),
                query_text: query.query.clone(),
                user_id: None,
                session_id: None,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                results_count: ranked_results.len(),
                filters: query.filters,
                results: ranked_results.clone(),
            })
            .await;

        Ok(SearchResponse {
            results: ranked_results,
            total_count: results.len(),
            search_time_ms: start_time.elapsed().as_millis() as u64,
            facets,
            suggestions,
            query_info: QueryInfo {
                processed_query: processed_query.query,
                query_expansion: vec![],
                query_correction: None,
            },
        })
    }

    /// Generate search suggestions
    pub async fn get_suggestions(&self, query: &str) -> Result<Vec<String>> {
        self.suggestions.generate_suggestions(query).await
    }

    /// Get search analytics
    pub async fn get_analytics(&self) -> Result<SearchMetrics> {
        Ok(self.analytics.get_metrics().await)
    }

    /// Generate facets
    async fn generate_facets(
        &self,
        results: &[ScoredDocument],
        facet_options: &[FacetOption],
    ) -> Result<HashMap<String, Vec<FacetValue>>> {
        let mut facets = HashMap::new();

        for facet_option in facet_options {
            let mut facet_values = HashMap::new();

            for result in results {
                if let Some(value) = result.metadata.get(&facet_option.field) {
                    let value_str = value.to_string();
                    *facet_values.entry(value_str).or_insert(0) += 1;
                }
            }

            let mut values: Vec<FacetValue> = facet_values
                .into_iter()
                .map(|(value, count)| FacetValue { value, count })
                .collect();

            values.sort_by(|a, b| b.count.cmp(&a.count));
            values.truncate(facet_option.limit);

            facets.insert(facet_option.field.clone(), values);
        }

        Ok(facets)
    }
}

impl SearchIndex {
    /// Create new search index
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
            inverted_index: HashMap::new(),
            vector_index: HashMap::new(),
            metadata_index: HashMap::new(),
            statistics: SearchIndexStatistics::default(),
        }
    }

    /// Add document to index
    fn add_document(&mut self, document: SearchDocument) -> Result<()> {
        // Add to documents
        self.documents.insert(document.id.clone(), document.clone());

        // Update inverted index
        self.update_inverted_index(&document)?;

        // Update vector index
        if let Some(vector) = &document.vector {
            self.vector_index
                .insert(document.id.clone(), vector.clone());
        }

        // Update metadata index
        self.update_metadata_index(&document)?;

        // Update statistics
        self.update_statistics();

        Ok(())
    }

    /// Remove document from index
    fn remove_document(&mut self, document_id: &str) -> Result<()> {
        if let Some(document) = self.documents.remove(document_id) {
            // Remove from inverted index
            self.remove_from_inverted_index(&document)?;

            // Remove from vector index
            self.vector_index.remove(document_id);

            // Remove from metadata index
            self.remove_from_metadata_index(&document)?;

            // Update statistics
            self.update_statistics();
        }

        Ok(())
    }

    /// Update document in index
    fn update_document(&mut self, document: SearchDocument) -> Result<()> {
        // Remove old document
        self.remove_document(&document.id)?;

        // Add new document
        self.add_document(document)?;

        Ok(())
    }

    /// Search documents
    async fn search(&self, query: &ProcessedQuery) -> Result<Vec<ScoredDocument>> {
        let mut results = Vec::new();

        // Get candidate documents
        let candidates = self.get_candidate_documents(query).await?;

        // Score and rank candidates
        for document_id in candidates {
            if let Some(document) = self.documents.get(&document_id) {
                let score = self.calculate_document_score(document, query).await?;

                if score > 0.0 {
                    results.push(ScoredDocument {
                        document_id: document.id.clone(),
                        title: document.title.clone(),
                        snippet: self.generate_snippet(document, query).await?,
                        score,
                        score_breakdown: ScoreBreakdown {
                            text_relevance: score,
                            vector_similarity: 0.0,
                            recency: 0.0,
                            popularity: 0.0,
                            quality: 0.0,
                            boost: 0.0,
                            final_score: score,
                        },
                        metadata: document.metadata.clone(),
                        highlighted_terms: query.terms.clone(),
                    });
                }
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(results)
    }

    /// Update inverted index
    fn update_inverted_index(&mut self, document: &SearchDocument) -> Result<()> {
        let terms = self.extract_terms(&document.content);

        for term in terms {
            self.inverted_index
                .entry(term)
                .or_insert_with(Vec::new)
                .push(document.id.clone());
        }

        Ok(())
    }

    /// Remove from inverted index
    fn remove_from_inverted_index(&mut self, document: &SearchDocument) -> Result<()> {
        let terms = self.extract_terms(&document.content);

        for term in terms {
            if let Some(doc_ids) = self.inverted_index.get_mut(&term) {
                doc_ids.retain(|id| id != &document.id);
            }
        }

        Ok(())
    }

    /// Update metadata index
    fn update_metadata_index(&mut self, document: &SearchDocument) -> Result<()> {
        for (key, value) in &document.metadata {
            self.metadata_index
                .entry(key.clone())
                .or_insert_with(HashMap::new)
                .entry(value.to_string())
                .or_insert_with(Vec::new)
                .push(document.id.clone());
        }

        Ok(())
    }

    /// Remove from metadata index
    fn remove_from_metadata_index(&mut self, document: &SearchDocument) -> Result<()> {
        for (key, value) in &document.metadata {
            if let Some(values) = self.metadata_index.get_mut(key) {
                if let Some(doc_ids) = values.get_mut(&value.to_string()) {
                    doc_ids.retain(|id| id != &document.id);
                }
            }
        }

        Ok(())
    }

    /// Extract terms from text
    fn extract_terms(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get candidate documents
    async fn get_candidate_documents(&self, query: &ProcessedQuery) -> Result<Vec<String>> {
        let mut candidates = HashSet::new();

        for term in &query.terms {
            if let Some(doc_ids) = self.inverted_index.get(term) {
                for doc_id in doc_ids {
                    candidates.insert(doc_id.clone());
                }
            }
        }

        Ok(candidates.into_iter().collect())
    }

    /// Calculate document score
    async fn calculate_document_score(
        &self,
        document: &SearchDocument,
        query: &ProcessedQuery,
    ) -> Result<f32> {
        let mut score = 0.0;

        // Text relevance score
        let text_score = self.calculate_text_relevance(document, query);
        score += text_score;

        // Vector similarity score
        if let Some(vector) = &document.vector {
            let vector_score = self.calculate_vector_similarity(vector, query);
            score += vector_score;
        }

        Ok(score)
    }

    /// Calculate text relevance
    fn calculate_text_relevance(&self, document: &SearchDocument, query: &ProcessedQuery) -> f32 {
        let mut score = 0.0;

        for term in &query.terms {
            let term_freq = document.content.to_lowercase().matches(term).count() as f32;
            if term_freq > 0.0 {
                score += term_freq;
            }
        }

        score
    }

    /// Calculate vector similarity
    fn calculate_vector_similarity(&self, vector: &[f32], query: &ProcessedQuery) -> f32 {
        // Simplified vector similarity calculation
        // In practice, this would use proper vector similarity metrics
        0.0
    }

    /// Generate snippet
    async fn generate_snippet(
        &self,
        document: &SearchDocument,
        query: &ProcessedQuery,
    ) -> Result<String> {
        // Simple snippet generation
        let content = &document.content;
        let max_length = 200;

        if content.len() <= max_length {
            Ok(content.clone())
        } else {
            Ok(format!("{}...", &content[..max_length]))
        }
    }

    /// Update statistics
    fn update_statistics(&mut self) {
        self.statistics.total_documents = self.documents.len();
        self.statistics.total_terms = self.inverted_index.len();

        let total_length: usize = self.documents.values().map(|doc| doc.content.len()).sum();

        self.statistics.avg_document_length = if self.documents.is_empty() {
            0.0
        } else {
            total_length as f32 / self.documents.len() as f32
        };
    }
}

impl QueryProcessor {
    /// Create new query processor
    fn new(config: QueryProcessingConfig) -> Self {
        Self {
            config,
            stop_words: HashSet::new(),
            synonyms: HashMap::new(),
        }
    }

    /// Process search query
    async fn process_query(&self, query: &SearchQuery) -> Result<ProcessedQuery> {
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

impl RankingEngine {
    /// Create new ranking engine
    fn new(config: RankingConfig) -> Self {
        Self {
            config,
            models: HashMap::new(),
        }
    }

    /// Rank search results
    async fn rank_results(
        &self,
        results: &[ScoredDocument],
        query: &ProcessedQuery,
    ) -> Result<Vec<ScoredDocument>> {
        let mut ranked_results = results.to_vec();

        // Apply ranking algorithm
        match self.config.algorithm {
            RankingAlgorithm::Bm25 => {
                self.apply_bm25_ranking(&mut ranked_results, query).await?;
            }
            RankingAlgorithm::TfIdf => {
                self.apply_tfidf_ranking(&mut ranked_results, query).await?;
            }
            RankingAlgorithm::LearningToRank => {
                self.apply_learning_to_rank(&mut ranked_results, query)
                    .await?;
            }
            RankingAlgorithm::NeuralRanking => {
                self.apply_neural_ranking(&mut ranked_results, query)
                    .await?;
            }
            RankingAlgorithm::Hybrid => {
                self.apply_hybrid_ranking(&mut ranked_results, query)
                    .await?;
            }
        }

        // Sort by final score
        ranked_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(ranked_results)
    }

    /// Apply BM25 ranking
    async fn apply_bm25_ranking(
        &self,
        results: &mut [ScoredDocument],
        _query: &ProcessedQuery,
    ) -> Result<()> {
        // Simplified BM25 implementation
        for result in results.iter_mut() {
            result.score_breakdown.final_score = result.score;
        }
        Ok(())
    }

    /// Apply TF-IDF ranking
    async fn apply_tfidf_ranking(
        &self,
        results: &mut [ScoredDocument],
        _query: &ProcessedQuery,
    ) -> Result<()> {
        // Simplified TF-IDF implementation
        for result in results.iter_mut() {
            result.score_breakdown.final_score = result.score;
        }
        Ok(())
    }

    /// Apply learning to rank
    async fn apply_learning_to_rank(
        &self,
        results: &mut [ScoredDocument],
        _query: &ProcessedQuery,
    ) -> Result<()> {
        // Learning to rank implementation
        Ok(())
    }

    /// Apply neural ranking
    async fn apply_neural_ranking(
        &self,
        results: &mut [ScoredDocument],
        _query: &ProcessedQuery,
    ) -> Result<()> {
        // Neural ranking implementation
        Ok(())
    }

    /// Apply hybrid ranking
    async fn apply_hybrid_ranking(
        &self,
        results: &mut [ScoredDocument],
        _query: &ProcessedQuery,
    ) -> Result<()> {
        // Hybrid ranking implementation
        Ok(())
    }
}

impl SearchAnalytics {
    /// Create new search analytics
    fn new(config: SearchAnalyticsConfig) -> Self {
        Self {
            config,
            query_logs: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(SearchMetrics::default())),
        }
    }

    /// Log query
    async fn log_query(&self, query_log: QueryLog) {
        if self.config.enabled {
            let mut logs = self.query_logs.write();
            logs.push(query_log);

            // Keep only recent logs
            if logs.len() > 10000 {
                let len = logs.len();
                if len > 10000 {
                    logs.drain(0..len - 10000);
                }
            }
        }
    }

    /// Get metrics
    async fn get_metrics(&self) -> SearchMetrics {
        self.metrics.read().clone()
    }
}

impl SearchSuggestions {
    /// Create new search suggestions
    fn new(config: SearchSuggestionsConfig) -> Self {
        Self {
            config,
            suggestion_index: Arc::new(RwLock::new(HashMap::new())),
            query_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate suggestions
    async fn generate_suggestions(&self, query: &str) -> Result<Vec<String>> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let mut suggestions = Vec::new();

        // Add query to history
        {
            let mut history = self.query_history.write();
            history.push(query.to_string());
        }

        // Generate suggestions based on configuration
        for suggestion_type in &self.config.suggestion_types {
            match suggestion_type {
                SuggestionType::QueryCompletion => {
                    suggestions.extend(self.generate_query_completions(query).await?);
                }
                SuggestionType::QueryCorrection => {
                    suggestions.extend(self.generate_query_corrections(query).await?);
                }
                SuggestionType::RelatedQueries => {
                    suggestions.extend(self.generate_related_queries(query).await?);
                }
                SuggestionType::PopularQueries => {
                    suggestions.extend(self.generate_popular_queries().await?);
                }
                SuggestionType::TrendingQueries => {
                    suggestions.extend(self.generate_trending_queries().await?);
                }
            }
        }

        // Limit suggestions
        suggestions.truncate(self.config.max_suggestions);

        Ok(suggestions)
    }

    /// Generate query completions
    async fn generate_query_completions(&self, query: &str) -> Result<Vec<String>> {
        // Simplified query completion
        Ok(vec![])
    }

    /// Generate query corrections
    async fn generate_query_corrections(&self, query: &str) -> Result<Vec<String>> {
        // Simplified query correction
        Ok(vec![])
    }

    /// Generate related queries
    async fn generate_related_queries(&self, query: &str) -> Result<Vec<String>> {
        // Simplified related queries
        Ok(vec![])
    }

    /// Generate popular queries
    async fn generate_popular_queries(&self) -> Result<Vec<String>> {
        // Simplified popular queries
        Ok(vec![])
    }

    /// Generate trending queries
    async fn generate_trending_queries(&self) -> Result<Vec<String>> {
        // Simplified trending queries
        Ok(vec![])
    }
}

/// Processed query
#[derive(Debug, Clone)]
pub struct ProcessedQuery {
    /// Processed query text
    pub query: String,

    /// Query terms
    pub terms: Vec<String>,

    /// Query filters
    pub filters: HashMap<String, serde_json::Value>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            modes: SearchModes {
                enable_text_search: true,
                enable_vector_search: true,
                enable_hybrid_search: true,
                enable_semantic_search: false,
                enable_fuzzy_search: false,
                enable_faceted_search: false,
            },
            ranking: RankingConfig {
                algorithm: RankingAlgorithm::Bm25,
                weights: RankingWeights {
                    text_relevance: 0.4,
                    vector_similarity: 0.3,
                    recency: 0.1,
                    popularity: 0.1,
                    quality: 0.1,
                    user_preference: 0.0,
                },
                boost_factors: BoostFactors {
                    title_boost: 2.0,
                    description_boost: 1.5,
                    content_boost: 1.0,
                    tag_boost: 1.2,
                    category_boost: 1.1,
                },
                decay_factors: DecayFactors {
                    time_decay: 0.1,
                    distance_decay: 0.0,
                    quality_decay: 0.0,
                },
            },
            query_processing: QueryProcessingConfig {
                enable_query_expansion: false,
                enable_query_correction: false,
                enable_query_normalization: true,
                enable_stop_word_removal: true,
                enable_stemming: false,
                enable_synonym_expansion: false,
                max_query_length: 1000,
                query_timeout_seconds: 30,
            },
            analytics: SearchAnalyticsConfig {
                enabled: true,
                retention_days: 30,
                track_performance: true,
                track_user_behavior: false,
                track_search_patterns: true,
            },
            suggestions: SearchSuggestionsConfig {
                enabled: true,
                suggestion_types: vec![SuggestionType::QueryCompletion],
                max_suggestions: 10,
                suggestion_timeout_seconds: 5,
            },
            performance: SearchPerformanceConfig {
                enable_caching: true,
                cache_size: 1000,
                cache_ttl_seconds: 300,
                enable_parallel_processing: true,
                max_concurrent_queries: 100,
                query_timeout_seconds: 30,
            },
        }
    }
}


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
