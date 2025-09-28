/**
 * Search result model for representing search results.
 */

import { ValidationError } from '../exceptions';

export interface SearchResult {
  /** Vector ID */
  id: string;
  /** Similarity score */
  score: number;
  /** Vector data */
  data: number[];
  /** Optional metadata */
  metadata?: Record<string, unknown>;
}

export interface SearchResponse {
  /** Search results */
  results: SearchResult[];
  /** Total number of results */
  total: number;
  /** Search query used */
  query?: string;
  /** Search parameters */
  parameters?: {
    limit?: number;
    threshold?: number;
    include_metadata?: boolean;
  };
}

/**
 * Validates search result data.
 * 
 * @param result - Search result to validate
 * @throws {Error} If search result data is invalid
 */
export function validateSearchResult(result: SearchResult): void {
  if (!result.id || typeof result.id !== 'string') {
    throw new ValidationError('Search result ID must be a non-empty string');
  }
  
  if (typeof result.score !== 'number' || isNaN(result.score)) {
    throw new ValidationError('Search result score must be a valid number');
  }
  
  if (!Array.isArray(result.data) || result.data.length === 0) {
    throw new ValidationError('Search result data must be a non-empty array');
  }
  
  if (!result.data.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new ValidationError('Search result data must contain only valid numbers');
  }
}

/**
 * Validates search response data.
 * 
 * @param response - Search response to validate
 * @throws {Error} If search response data is invalid
 */
export function validateSearchResponse(response: SearchResponse): void {
  if (!Array.isArray(response.results)) {
    throw new ValidationError('Search response results must be an array');
  }
  
  if (typeof response.total !== 'number' || response.total < 0) {
    throw new ValidationError('Search response total must be a non-negative number');
  }
  
  response.results.forEach((result, index) => {
    try {
      validateSearchResult(result);
    } catch (error) {
      throw new ValidationError(`Invalid search result at index ${index}: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  });
}
