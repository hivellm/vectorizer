/**
 * Search request model for vector similarity search.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates search request data.
 * 
 * @param {Object} request - Search request to validate
 * @throws {ValidationError} If search request data is invalid
 */
export function validateSearchRequest(request) {
  if (!Array.isArray(request.query_vector) || request.query_vector.length === 0) {
    throw new ValidationError('Search request query vector must be a non-empty array');
  }
  
  if (!request.query_vector.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new ValidationError('Search request query vector must contain only valid numbers');
  }
  
  if (request.limit !== undefined) {
    if (typeof request.limit !== 'number' || request.limit <= 0) {
      throw new ValidationError('Search request limit must be a positive number');
    }
  }
  
  if (request.threshold !== undefined) {
    if (typeof request.threshold !== 'number' || request.threshold < 0 || request.threshold > 1) {
      throw new ValidationError('Search request threshold must be a number between 0 and 1');
    }
  }
  
  if (request.include_metadata !== undefined) {
    if (typeof request.include_metadata !== 'boolean') {
      throw new ValidationError('Search request include_metadata must be a boolean');
    }
  }
}

/**
 * Validates text search request data.
 * 
 * @param {Object} request - Text search request to validate
 * @throws {ValidationError} If text search request data is invalid
 */
export function validateTextSearchRequest(request) {
  if (!request.query || typeof request.query !== 'string') {
    throw new ValidationError('Text search request query must be a non-empty string');
  }
  
  if (request.limit !== undefined) {
    if (typeof request.limit !== 'number' || request.limit <= 0) {
      throw new ValidationError('Text search request limit must be a positive number');
    }
  }
  
  if (request.threshold !== undefined) {
    if (typeof request.threshold !== 'number' || request.threshold < 0 || request.threshold > 1) {
      throw new ValidationError('Text search request threshold must be a number between 0 and 1');
    }
  }
  
  if (request.include_metadata !== undefined) {
    if (typeof request.include_metadata !== 'boolean') {
      throw new ValidationError('Text search request include_metadata must be a boolean');
    }
  }
  
  if (request.model !== undefined) {
    if (typeof request.model !== 'string') {
      throw new ValidationError('Text search request model must be a string');
    }
  }
}
