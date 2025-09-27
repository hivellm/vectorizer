/**
 * Embedding request model for text embedding generation.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates embedding request data.
 * 
 * @param {Object} request - Embedding request to validate
 * @throws {ValidationError} If embedding request data is invalid
 */
export function validateEmbeddingRequest(request) {
  if (!request.text || typeof request.text !== 'string') {
    throw new ValidationError('Embedding request text must be a non-empty string');
  }
  
  if (request.model && typeof request.model !== 'string') {
    throw new ValidationError('Embedding request model must be a string');
  }
  
  if (request.parameters) {
    if (request.parameters.max_length !== undefined) {
      if (typeof request.parameters.max_length !== 'number' || request.parameters.max_length <= 0) {
        throw new ValidationError('Max length must be a positive number');
      }
    }
    
    if (request.parameters.normalize !== undefined) {
      if (typeof request.parameters.normalize !== 'boolean') {
        throw new ValidationError('Normalize must be a boolean');
      }
    }
    
    if (request.parameters.prefix !== undefined) {
      if (typeof request.parameters.prefix !== 'string') {
        throw new ValidationError('Prefix must be a string');
      }
    }
  }
}

/**
 * Validates embedding response data.
 * 
 * @param {Object} response - Embedding response to validate
 * @throws {ValidationError} If embedding response data is invalid
 */
export function validateEmbeddingResponse(response) {
  if (!Array.isArray(response.embedding) || response.embedding.length === 0) {
    throw new ValidationError('Embedding response embedding must be a non-empty array');
  }
  
  if (!response.embedding.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new ValidationError('Embedding response embedding must contain only valid numbers');
  }
  
  if (!response.model || typeof response.model !== 'string') {
    throw new ValidationError('Embedding response model must be a non-empty string');
  }
  
  if (!response.text || typeof response.text !== 'string') {
    throw new ValidationError('Embedding response text must be a non-empty string');
  }
}
