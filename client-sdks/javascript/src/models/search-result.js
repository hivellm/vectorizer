/**
 * Search result model for representing search results.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates search result data.
 * 
 * @param {Object} result - Search result to validate
 * @throws {ValidationError} If search result data is invalid
 */
export function validateSearchResult(result) {
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
 * @param {Object} response - Search response to validate
 * @throws {ValidationError} If search response data is invalid
 */
export function validateSearchResponse(response) {
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
      throw new ValidationError(`Invalid search result at index ${index}: ${error.message}`);
    }
  });
}
