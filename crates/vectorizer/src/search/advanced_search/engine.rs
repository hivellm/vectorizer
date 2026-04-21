//! Outer search engine + index implementation.
//!
//! [`AdvancedSearchEngine`] is the orchestrator — it owns the index,
//! query processor, ranker, analytics, and suggestions, and routes
//! each incoming `SearchQuery` through them in order. [`SearchIndex`]
//! is the in-memory inverted / vector / metadata index the orchestrator
//! reads and writes. The other impls live in sibling files:
//! [`super::query_processor`], [`super::ranker`], [`super::analytics`].

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use parking_lot::RwLock;

use super::types::*;

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
        let results = index.search(&processed_query).await?;

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
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
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
    pub(super) fn new() -> Self {
        Self {
            documents: HashMap::new(),
            inverted_index: HashMap::new(),
            vector_index: HashMap::new(),
            metadata_index: HashMap::new(),
            statistics: SearchIndexStatistics::default(),
        }
    }

    /// Add document to index
    pub(super) fn add_document(&mut self, document: SearchDocument) -> Result<()> {
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
    pub(super) fn remove_document(&mut self, document_id: &str) -> Result<()> {
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
    pub(super) fn update_document(&mut self, document: SearchDocument) -> Result<()> {
        // Remove old document
        self.remove_document(&document.id)?;

        // Add new document
        self.add_document(document)?;

        Ok(())
    }

    /// Search documents
    pub(super) async fn search(&self, query: &ProcessedQuery) -> Result<Vec<ScoredDocument>> {
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
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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
    fn calculate_vector_similarity(&self, _vector: &[f32], _query: &ProcessedQuery) -> f32 {
        // Simplified vector similarity calculation
        // In practice, this would use proper vector similarity metrics
        0.0
    }

    /// Generate snippet
    async fn generate_snippet(
        &self,
        document: &SearchDocument,
        _query: &ProcessedQuery,
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
