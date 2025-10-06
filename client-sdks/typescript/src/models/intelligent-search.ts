/**
 * Intelligent Search Models for the Hive Vectorizer SDK.
 * 
 * This module contains data models for the advanced intelligent search features
 * including multi-query expansion, semantic reranking, and contextual search.
 */

/**
 * Intelligent Search Request
 */
export interface IntelligentSearchRequest {
  /** Search query */
  query: string;
  /** Collections to search (optional - searches all if not specified) */
  collections?: string[];
  /** Maximum number of results */
  max_results?: number;
  /** Enable domain expansion */
  domain_expansion?: boolean;
  /** Enable technical focus */
  technical_focus?: boolean;
  /** Enable MMR diversification */
  mmr_enabled?: boolean;
  /** MMR balance parameter (0.0-1.0) */
  mmr_lambda?: number;
}

/**
 * Semantic Search Request
 */
export interface SemanticSearchRequest {
  /** Search query */
  query: string;
  /** Collection to search */
  collection: string;
  /** Maximum number of results */
  max_results?: number;
  /** Enable semantic reranking */
  semantic_reranking?: boolean;
  /** Enable cross-encoder reranking */
  cross_encoder_reranking?: boolean;
  /** Minimum similarity threshold */
  similarity_threshold?: number;
}

/**
 * Contextual Search Request
 */
export interface ContextualSearchRequest {
  /** Search query */
  query: string;
  /** Collection to search */
  collection: string;
  /** Metadata-based context filters */
  context_filters?: Record<string, any>;
  /** Maximum number of results */
  max_results?: number;
  /** Enable context-aware reranking */
  context_reranking?: boolean;
  /** Weight of context factors (0.0-1.0) */
  context_weight?: number;
}

/**
 * Multi-Collection Search Request
 */
export interface MultiCollectionSearchRequest {
  /** Search query */
  query: string;
  /** Collections to search */
  collections: string[];
  /** Maximum results per collection */
  max_per_collection?: number;
  /** Maximum total results */
  max_total_results?: number;
  /** Enable cross-collection reranking */
  cross_collection_reranking?: boolean;
}

/**
 * Intelligent Search Result
 */
export interface IntelligentSearchResult {
  /** Result ID */
  id: string;
  /** Similarity score */
  score: number;
  /** Result content */
  content: string;
  /** Metadata */
  metadata?: Record<string, any>;
  /** Collection name */
  collection?: string;
  /** Query used for this result */
  query_used?: string;
}

/**
 * Intelligent Search Response
 */
export interface IntelligentSearchResponse {
  /** Search results */
  results: IntelligentSearchResult[];
  /** Total number of results found */
  total_results: number;
  /** Search duration in milliseconds */
  duration_ms: number;
  /** Queries generated */
  queries_generated?: string[];
  /** Collections searched */
  collections_searched?: string[];
  /** Search metadata */
  metadata?: {
    domain_expansion_enabled?: boolean;
    technical_focus_enabled?: boolean;
    mmr_enabled?: boolean;
    mmr_lambda?: number;
  };
}

/**
 * Semantic Search Response
 */
export interface SemanticSearchResponse {
  /** Search results */
  results: IntelligentSearchResult[];
  /** Total number of results found */
  total_results: number;
  /** Search duration in milliseconds */
  duration_ms: number;
  /** Collection searched */
  collection: string;
  /** Search metadata */
  metadata?: {
    semantic_reranking_enabled?: boolean;
    cross_encoder_reranking_enabled?: boolean;
    similarity_threshold?: number;
  };
}

/**
 * Contextual Search Response
 */
export interface ContextualSearchResponse {
  /** Search results */
  results: IntelligentSearchResult[];
  /** Total number of results found */
  total_results: number;
  /** Search duration in milliseconds */
  duration_ms: number;
  /** Collection searched */
  collection: string;
  /** Context filters applied */
  context_filters?: Record<string, any>;
  /** Search metadata */
  metadata?: {
    context_reranking_enabled?: boolean;
    context_weight?: number;
  };
}

/**
 * Multi-Collection Search Response
 */
export interface MultiCollectionSearchResponse {
  /** Search results */
  results: IntelligentSearchResult[];
  /** Total number of results found */
  total_results: number;
  /** Search duration in milliseconds */
  duration_ms: number;
  /** Collections searched */
  collections_searched: string[];
  /** Results per collection */
  results_per_collection?: Record<string, number>;
  /** Search metadata */
  metadata?: {
    cross_collection_reranking_enabled?: boolean;
    max_per_collection?: number;
    max_total_results?: number;
  };
}
