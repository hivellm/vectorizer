/**
 * Vector model for representing vectors with metadata.
 */

import { ValidationError } from '../exceptions/index.js';

/**
 * Validates vector data.
 * 
 * @param {Object} vector - Vector to validate
 * @throws {ValidationError} If vector data is invalid
 */
export function validateVector(vector) {
  if (!vector.id || typeof vector.id !== 'string') {
    throw new ValidationError('Vector ID must be a non-empty string');
  }
  
  if (!Array.isArray(vector.data) || vector.data.length === 0) {
    throw new ValidationError('Vector data must be a non-empty array');
  }
  
  if (!vector.data.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new ValidationError('Vector data must contain only valid numbers');
  }
}

/**
 * Validates create vector request.
 * 
 * @param {Object} request - Create vector request to validate
 * @throws {ValidationError} If request data is invalid
 */
export function validateCreateVectorRequest(request) {
  if (!Array.isArray(request.data) || request.data.length === 0) {
    throw new ValidationError('Vector data must be a non-empty array');
  }
  
  if (!request.data.every(x => typeof x === 'number' && !isNaN(x))) {
    throw new ValidationError('Vector data must contain only valid numbers');
  }
}
