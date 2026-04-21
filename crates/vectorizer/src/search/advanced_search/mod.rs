//! Advanced search and discovery system.
//!
//! Provides sophisticated search capabilities including:
//! - Multi-modal search (text, vector, hybrid)
//! - Advanced ranking algorithms
//! - Search result clustering and deduplication
//! - Query expansion and suggestion
//! - Search analytics and optimization
//! - Real-time search updates
//!
//! # Layout
//!
//! The engine is split across sibling files so each concern is
//! reviewable in isolation and no single file exceeds the 600-line
//! threshold:
//!
//! - [`types`] — every struct / enum the public surface needs
//! - [`engine`] — `impl AdvancedSearchEngine` + `impl SearchIndex`
//!   (orchestrator + in-memory index)
//! - [`query_processor`] — `impl QueryProcessor` (normalization,
//!   stop-word removal, stemming, synonym expansion) + the
//!   `ProcessedQuery` bridge type
//! - [`ranker`] — `impl RankingEngine` (BM25 / TF-IDF / LTR / neural /
//!   hybrid)
//! - [`analytics`] — `impl SearchAnalytics` + `impl SearchSuggestions`
//!
//! Cross-engine wiring lives in [`engine`] — each engine exposes only
//! `pub(super)` methods, so the orchestrator is the single call site.

mod analytics;
mod engine;
mod query_processor;
mod ranker;
mod types;

pub use query_processor::ProcessedQuery;
pub use types::*;

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
