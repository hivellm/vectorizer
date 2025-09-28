/**
 * Batch operation models for the Hive Vectorizer SDK.
 * 
 * This module contains data models for batch operations including
 * batch insert, search, update, and delete operations.
 */

export interface BatchTextRequest {
  /** Text ID */
  id: string;
  /** Text content */
  text: string;
  /** Optional metadata */
  metadata?: Record<string, any>;
}

export interface BatchInsertRequest {
  /** Texts to insert */
  texts: BatchTextRequest[];
  /** Batch configuration (optional) */
  config?: BatchConfig;
}

export interface BatchConfig {
  /** Maximum batch size */
  max_batch_size?: number;
  /** Number of parallel workers */
  parallel_workers?: number;
  /** Whether operations should be atomic */
  atomic?: boolean;
}

export interface BatchResponse {
  /** Whether the operation was successful */
  success: boolean;
  /** Collection name */
  collection: string;
  /** Operation type */
  operation: string;
  /** Total number of operations */
  total_operations: number;
  /** Number of successful operations */
  successful_operations: number;
  /** Number of failed operations */
  failed_operations: number;
  /** Duration in milliseconds */
  duration_ms: number;
  /** Error messages (if any) */
  errors: string[];
}

export interface BatchSearchQuery {
  /** Query text */
  query: string;
  /** Maximum number of results */
  limit?: number;
  /** Minimum score threshold */
  score_threshold?: number;
}

export interface BatchSearchRequest {
  /** Search queries */
  queries: BatchSearchQuery[];
  /** Batch configuration (optional) */
  config?: BatchConfig;
}

export interface BatchSearchResponse {
  /** Whether the operation was successful */
  success: boolean;
  /** Collection name */
  collection: string;
  /** Total number of queries */
  total_queries: number;
  /** Number of successful queries */
  successful_queries: number;
  /** Number of failed queries */
  failed_queries: number;
  /** Duration in milliseconds */
  duration_ms: number;
  /** Search results */
  results: SearchResult[][];
  /** Error messages (if any) */
  errors: string[];
}

export interface BatchUpdateRequest {
  /** Vector updates */
  updates: BatchVectorUpdate[];
  /** Batch configuration (optional) */
  config?: BatchConfig;
}

export interface BatchVectorUpdate {
  /** Vector ID */
  id: string;
  /** New vector data (optional) */
  data?: number[];
  /** New metadata (optional) */
  metadata?: Record<string, any>;
}

export interface BatchDeleteRequest {
  /** Vector IDs to delete */
  vector_ids: string[];
  /** Batch configuration (optional) */
  config?: BatchConfig;
}

// Import SearchResult from existing models
import { SearchResult } from './search-result';
