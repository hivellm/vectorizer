//! Type definitions for the advanced search engine — extracted from the
//! prior monolithic `advanced_search.rs` (phase4_split-advanced-search).
//! All structs/enums are byte-for-byte the same; the parent module
//! re-exports them via `pub use types::*;` so external callers see no
//! change.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Advanced search engine
#[derive(Debug, Clone)]
pub struct AdvancedSearchEngine {
    /// Search configuration
    pub(super) config: SearchConfig,

    /// Search index
    pub(super) index: Arc<RwLock<SearchIndex>>,

    /// Query processor
    pub(super) query_processor: Arc<QueryProcessor>,

    /// Ranking engine
    pub(super) ranking_engine: Arc<RankingEngine>,

    /// Search analytics
    pub(super) analytics: Arc<SearchAnalytics>,

    /// Search suggestions
    pub(super) suggestions: Arc<SearchSuggestions>,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Search modes
    pub modes: SearchModes,

    /// Ranking configuration
    pub ranking: RankingConfig,

    /// Query processing configuration
    pub query_processing: QueryProcessingConfig,

    /// Search analytics configuration
    pub analytics: SearchAnalyticsConfig,

    /// Search suggestions configuration
    pub suggestions: SearchSuggestionsConfig,

    /// Performance configuration
    pub performance: SearchPerformanceConfig,
}

/// Search modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchModes {
    /// Enable text search
    pub enable_text_search: bool,

    /// Enable vector search
    pub enable_vector_search: bool,

    /// Enable hybrid search
    pub enable_hybrid_search: bool,

    /// Enable semantic search
    pub enable_semantic_search: bool,

    /// Enable fuzzy search
    pub enable_fuzzy_search: bool,

    /// Enable faceted search
    pub enable_faceted_search: bool,
}

/// Ranking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    /// Ranking algorithm
    pub algorithm: RankingAlgorithm,

    /// Ranking weights
    pub weights: RankingWeights,

    /// Boost factors
    pub boost_factors: BoostFactors,

    /// Decay factors
    pub decay_factors: DecayFactors,
}

/// Ranking algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RankingAlgorithm {
    /// BM25 ranking
    Bm25,

    /// TF-IDF ranking
    TfIdf,

    /// Learning to rank
    LearningToRank,

    /// Neural ranking
    NeuralRanking,

    /// Hybrid ranking
    Hybrid,
}

/// Ranking weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingWeights {
    /// Text relevance weight
    pub text_relevance: f32,

    /// Vector similarity weight
    pub vector_similarity: f32,

    /// Recency weight
    pub recency: f32,

    /// Popularity weight
    pub popularity: f32,

    /// Quality weight
    pub quality: f32,

    /// User preference weight
    pub user_preference: f32,
}

/// Boost factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostFactors {
    /// Title boost
    pub title_boost: f32,

    /// Description boost
    pub description_boost: f32,

    /// Content boost
    pub content_boost: f32,

    /// Tag boost
    pub tag_boost: f32,

    /// Category boost
    pub category_boost: f32,
}

/// Decay factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayFactors {
    /// Time decay factor
    pub time_decay: f32,

    /// Distance decay factor
    pub distance_decay: f32,

    /// Quality decay factor
    pub quality_decay: f32,
}

/// Query processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryProcessingConfig {
    /// Enable query expansion
    pub enable_query_expansion: bool,

    /// Enable query correction
    pub enable_query_correction: bool,

    /// Enable query normalization
    pub enable_query_normalization: bool,

    /// Enable stop word removal
    pub enable_stop_word_removal: bool,

    /// Enable stemming
    pub enable_stemming: bool,

    /// Enable synonym expansion
    pub enable_synonym_expansion: bool,

    /// Maximum query length
    pub max_query_length: usize,

    /// Query timeout
    pub query_timeout_seconds: u64,
}

/// Search analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalyticsConfig {
    /// Enable analytics
    pub enabled: bool,

    /// Analytics retention days
    pub retention_days: u32,

    /// Track query performance
    pub track_performance: bool,

    /// Track user behavior
    pub track_user_behavior: bool,

    /// Track search patterns
    pub track_search_patterns: bool,
}

/// Search suggestions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestionsConfig {
    /// Enable suggestions
    pub enabled: bool,

    /// Suggestion types
    pub suggestion_types: Vec<SuggestionType>,

    /// Maximum suggestions
    pub max_suggestions: usize,

    /// Suggestion timeout
    pub suggestion_timeout_seconds: u64,
}

/// Suggestion types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// Query completion
    QueryCompletion,

    /// Query correction
    QueryCorrection,

    /// Related queries
    RelatedQueries,

    /// Popular queries
    PopularQueries,

    /// Trending queries
    TrendingQueries,
}

/// Search performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPerformanceConfig {
    /// Enable caching
    pub enable_caching: bool,

    /// Cache size
    pub cache_size: usize,

    /// Cache TTL
    pub cache_ttl_seconds: u64,

    /// Enable parallel processing
    pub enable_parallel_processing: bool,

    /// Maximum concurrent queries
    pub max_concurrent_queries: usize,

    /// Query timeout
    pub query_timeout_seconds: u64,
}

/// Search index
#[derive(Debug, Clone)]
pub struct SearchIndex {
    /// Documents
    pub(super) documents: HashMap<String, SearchDocument>,

    /// Inverted index
    pub(super) inverted_index: HashMap<String, Vec<String>>,

    /// Vector index
    pub(super) vector_index: HashMap<String, Vec<f32>>,

    /// Metadata index
    pub(super) metadata_index: HashMap<String, HashMap<String, Vec<String>>>,

    /// Statistics
    pub(super) statistics: SearchIndexStatistics,
}

/// Search document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Document ID
    pub id: String,

    /// Document title
    pub title: String,

    /// Document content
    pub content: String,

    /// Document description
    pub description: Option<String>,

    /// Document tags
    pub tags: Vec<String>,

    /// Document category
    pub category: Option<String>,

    /// Document metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Document vector
    pub vector: Option<Vec<f32>>,

    /// Document score
    pub score: f32,

    /// Document timestamp
    pub timestamp: u64,

    /// Document language
    pub language: Option<String>,
}

/// Search index statistics
#[derive(Debug, Clone, Default)]
pub struct SearchIndexStatistics {
    /// Total documents
    pub total_documents: usize,

    /// Total terms
    pub total_terms: usize,

    /// Average document length
    pub avg_document_length: f32,

    /// Index size in bytes
    pub index_size_bytes: usize,

    /// Last updated
    pub last_updated: u64,
}

/// Query processor
#[derive(Debug)]
pub struct QueryProcessor {
    /// Configuration
    pub(super) config: QueryProcessingConfig,

    /// Stop words
    pub(super) stop_words: HashSet<String>,

    /// Synonyms
    pub(super) synonyms: HashMap<String, Vec<String>>,
}

/// Ranking engine
#[derive(Debug)]
pub struct RankingEngine {
    /// Configuration
    pub(super) config: RankingConfig,

    /// Ranking models
    pub(super) models: HashMap<String, RankingModel>,
}

/// Ranking model
#[derive(Debug)]
pub struct RankingModel {
    /// Model name
    pub name: String,

    /// Model type
    pub model_type: RankingAlgorithm,

    /// Model parameters
    pub parameters: HashMap<String, f32>,
}

/// Search analytics
#[derive(Debug)]
pub struct SearchAnalytics {
    /// Configuration
    pub(super) config: SearchAnalyticsConfig,

    /// Query logs
    pub(super) query_logs: Arc<RwLock<Vec<QueryLog>>>,

    /// Search metrics
    pub(super) metrics: Arc<RwLock<SearchMetrics>>,
}

/// Query log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLog {
    /// Query ID
    pub query_id: String,

    /// Query text
    pub query_text: String,

    /// User ID
    pub user_id: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Query timestamp
    pub timestamp: u64,

    /// Query duration
    pub duration_ms: u64,

    /// Results count
    pub results_count: usize,

    /// Query filters
    pub filters: HashMap<String, serde_json::Value>,

    /// Query results
    pub results: Vec<ScoredDocument>,
}

/// Search metrics
#[derive(Debug, Clone, Default)]
pub struct SearchMetrics {
    /// Total queries
    pub total_queries: u64,

    /// Average query time
    pub avg_query_time_ms: f64,

    /// Query success rate
    pub query_success_rate: f64,

    /// Popular queries
    pub popular_queries: HashMap<String, u64>,

    /// Query patterns
    pub query_patterns: HashMap<String, u64>,
}

/// Search suggestions
#[derive(Debug)]
pub struct SearchSuggestions {
    /// Configuration
    pub(super) config: SearchSuggestionsConfig,

    /// Suggestion index
    pub(super) suggestion_index: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Query history
    pub(super) query_history: Arc<RwLock<Vec<String>>>,
}

/// Ranker output: a single scored document with enrichment fields
/// (title, snippet, highlighted terms) that don't belong on the
/// canonical `crate::models::SearchResult`. Renamed from `SearchResult`
/// in phase2_unify-search-result-type so callers never accidentally mix
/// this augmented shape with the HNSW scoring shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredDocument {
    /// Document ID
    pub document_id: String,

    /// Document title
    pub title: String,

    /// Document content snippet
    pub snippet: String,

    /// Relevance score
    pub score: f32,

    /// Score breakdown
    pub score_breakdown: ScoreBreakdown,

    /// Document metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Highlighted terms
    pub highlighted_terms: Vec<String>,
}

/// Score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Text relevance score
    pub text_relevance: f32,

    /// Vector similarity score
    pub vector_similarity: f32,

    /// Recency score
    pub recency: f32,

    /// Popularity score
    pub popularity: f32,

    /// Quality score
    pub quality: f32,

    /// Boost score
    pub boost: f32,

    /// Final score
    pub final_score: f32,
}

/// Search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query text
    pub query: String,

    /// Search mode
    pub mode: SearchMode,

    /// Collections to search
    pub collections: Vec<String>,

    /// Maximum results
    pub max_results: usize,

    /// Offset
    pub offset: usize,

    /// Filters
    pub filters: HashMap<String, serde_json::Value>,

    /// Sort options
    pub sort: Option<SortOption>,

    /// Facet options
    pub facets: Vec<FacetOption>,

    /// Highlight options
    pub highlight: Option<HighlightOption>,
}

/// Search modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchMode {
    /// Text search
    Text,

    /// Vector search
    Vector,

    /// Hybrid search
    Hybrid,

    /// Semantic search
    Semantic,
}

/// Sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOption {
    /// Sort field
    pub field: String,

    /// Sort direction
    pub direction: SortDirection,
}

/// Sort directions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending
    Asc,

    /// Descending
    Desc,
}

/// Facet options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetOption {
    /// Facet field
    pub field: String,

    /// Facet limit
    pub limit: usize,
}

/// Highlight options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightOption {
    /// Fields to highlight
    pub fields: Vec<String>,

    /// Highlight tags
    pub tags: HighlightTags,

    /// Maximum fragments
    pub max_fragments: usize,
}

/// Highlight tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightTags {
    /// Opening tag
    pub opening: String,

    /// Closing tag
    pub closing: String,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<ScoredDocument>,

    /// Total results count
    pub total_count: usize,

    /// Search time
    pub search_time_ms: u64,

    /// Facets
    pub facets: HashMap<String, Vec<FacetValue>>,

    /// Suggestions
    pub suggestions: Vec<String>,

    /// Query information
    pub query_info: QueryInfo,
}

/// Facet value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    /// Value
    pub value: String,

    /// Count
    pub count: usize,
}

/// Query information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryInfo {
    /// Processed query
    pub processed_query: String,

    /// Query expansion
    pub query_expansion: Vec<String>,

    /// Query correction
    pub query_correction: Option<String>,
}

