/**
 * Search request model for vector similarity search.
 */

export interface SearchRequest {
  /** Query vector for similarity search */
  query_vector: number[];
  /** Maximum number of results to return */
  limit?: number;
  /** Minimum similarity threshold */
  threshold?: number;
  /** Whether to include metadata in results */
  include_metadata?: boolean;
  /** Optional filter for metadata */
  filter?: Record<string, unknown>;
}

export interface TextSearchRequest {
  /** Text query for semantic search */
  query: string;
  /** Maximum number of results to return */
  limit?: number;
  /** Minimum similarity threshold */
  threshold?: number;
  /** Whether to include metadata in results */
  include_metadata?: boolean;
  /** Optional filter for metadata */
  filter?: Record<string, unknown>;
  /** Optional model to use for embedding */
  model?: string;
}

/**
 * Validates search request data.
 * 
 * @param request - Search request to validate
 * @throws {Error} If search request data is invalid
 */
export function validateSearchRequest(request: SearchRequest): void {
  if (!Array.isArray(request.query_vector) || request.query_vector.length === 0) {
    throw new Error('Search request query vector must be a non-empty array');
  }
  
  if (!request.query_vector.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new Error('Search request query vector must contain only valid numbers');
  }
  
  if (request.limit !== undefined) {
    if (typeof request.limit !== 'number' || request.limit <= 0) {
      throw new Error('Search request limit must be a positive number');
    }
  }
  
  if (request.threshold !== undefined) {
    if (typeof request.threshold !== 'number' || request.threshold < 0 || request.threshold > 1) {
      throw new Error('Search request threshold must be a number between 0 and 1');
    }
  }
  
  if (request.include_metadata !== undefined) {
    if (typeof request.include_metadata !== 'boolean') {
      throw new Error('Search request include_metadata must be a boolean');
    }
  }
}

/**
 * Validates text search request data.
 * 
 * @param request - Text search request to validate
 * @throws {Error} If text search request data is invalid
 */
export function validateTextSearchRequest(request: TextSearchRequest): void {
  if (!request.query || typeof request.query !== 'string') {
    throw new Error('Text search request query must be a non-empty string');
  }
  
  if (request.limit !== undefined) {
    if (typeof request.limit !== 'number' || request.limit <= 0) {
      throw new Error('Text search request limit must be a positive number');
    }
  }
  
  if (request.threshold !== undefined) {
    if (typeof request.threshold !== 'number' || request.threshold < 0 || request.threshold > 1) {
      throw new Error('Text search request threshold must be a number between 0 and 1');
    }
  }
  
  if (request.include_metadata !== undefined) {
    if (typeof request.include_metadata !== 'boolean') {
      throw new Error('Text search request include_metadata must be a boolean');
    }
  }
  
  if (request.model !== undefined) {
    if (typeof request.model !== 'string') {
      throw new Error('Text search request model must be a string');
    }
  }
}
