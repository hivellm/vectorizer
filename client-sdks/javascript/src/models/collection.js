/**
 * Collection model for representing collections of vectors.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates collection data.
 * 
 * @param {Object} collection - Collection to validate
 * @throws {ValidationError} If collection data is invalid
 */
export function validateCollection(collection) {
  if (!collection.name || typeof collection.name !== 'string') {
    throw new ValidationError('Collection name must be a non-empty string');
  }
  
  if (typeof collection.dimension !== 'number' || collection.dimension <= 0) {
    throw new ValidationError('Dimension must be a positive number');
  }
  
  const validMetrics = ['cosine', 'euclidean', 'dot_product'];
  if (!validMetrics.includes(collection.similarity_metric)) {
    throw new ValidationError(`Invalid similarity metric. Must be one of: ${validMetrics.join(', ')}`);
  }
}

/**
 * Validates create collection request.
 * 
 * @param {Object} request - Create collection request to validate
 * @throws {ValidationError} If request data is invalid
 */
export function validateCreateCollectionRequest(request) {
  if (!request.name || typeof request.name !== 'string') {
    throw new ValidationError('Collection name must be a non-empty string');
  }
  
  if (typeof request.dimension !== 'number' || request.dimension <= 0) {
    throw new ValidationError('Dimension must be a positive number');
  }
  
  if (request.similarity_metric) {
    const validMetrics = ['cosine', 'euclidean', 'dot_product'];
    if (!validMetrics.includes(request.similarity_metric)) {
      throw new ValidationError(`Invalid similarity metric. Must be one of: ${validMetrics.join(', ')}`);
    }
  }
}
