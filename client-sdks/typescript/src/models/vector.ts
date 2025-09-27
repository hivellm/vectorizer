/**
 * Vector model for representing vectors with metadata.
 */

import { ValidationError } from '../exceptions';

export interface Vector {
  /** Unique identifier for the vector */
  id: string;
  /** Vector data as an array of numbers */
  data: number[];
  /** Optional metadata associated with the vector */
  metadata?: Record<string, unknown>;
}

export interface CreateVectorRequest {
  /** Vector data as an array of numbers */
  data: number[];
  /** Optional metadata associated with the vector */
  metadata?: Record<string, unknown>;
}

export interface UpdateVectorRequest {
  /** Vector data as an array of numbers */
  data?: number[];
  /** Optional metadata associated with the vector */
  metadata?: Record<string, unknown>;
}

/**
 * Validates vector data.
 * 
 * @param vector - Vector to validate
 * @throws {Error} If vector data is invalid
 */
export function validateVector(vector: Vector): void {
  if (!vector.id || typeof vector.id !== 'string') {
    throw new ValidationError('Vector ID must be a non-empty string');
  }
  
  if (!Array.isArray(vector.data) || vector.data.length === 0) {
    throw new ValidationError('Vector data must be a non-empty array');
  }
  
  if (!vector.data.every(x => typeof x === 'number' && !isNaN(x) && isFinite(x))) {
    throw new ValidationError('Vector data must contain only valid numbers');
  }
}

/**
 * Validates create vector request.
 * 
 * @param request - Create vector request to validate
 * @throws {Error} If request data is invalid
 */
export function validateCreateVectorRequest(request: CreateVectorRequest): void {
  if (!Array.isArray(request.data) || request.data.length === 0) {
    throw new ValidationError('Vector data must be a non-empty array');
  }
  
  if (!request.data.every(x => typeof x === 'number' && !isNaN(x) && isFinite(x))) {
    throw new ValidationError('Vector data must contain only valid numbers');
  }
}
