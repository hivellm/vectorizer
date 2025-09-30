/**
 * Collection model for representing collections of vectors.
 */

import { ValidationError } from '../exceptions';

export type SimilarityMetric = 'cosine' | 'euclidean' | 'dot_product';

export interface Collection {
  /** Collection name */
  name: string;
  /** Vector dimension */
  dimension: number;
  /** Similarity metric used for search */
  similarity_metric: SimilarityMetric;
  /** Optional description */
  description?: string;
  /** Creation timestamp */
  created_at?: Date;
  /** Last update timestamp */
  updated_at?: Date;
}

export interface CreateCollectionRequest {
  /** Collection name */
  name: string;
  /** Vector dimension */
  dimension: number;
  /** Similarity metric used for search */
  similarity_metric?: SimilarityMetric;
  /** Optional description */
  description?: string;
}

export interface UpdateCollectionRequest {
  /** Optional description */
  description?: string;
  /** Similarity metric used for search */
  similarity_metric?: SimilarityMetric;
}

/**
 * Validates collection data.
 * 
 * @param collection - Collection to validate
 * @throws {Error} If collection data is invalid
 */
export function validateCollection(collection: Collection): void {
  if (!collection.name || typeof collection.name !== 'string') {
    throw new ValidationError('Collection name must be a non-empty string');
  }
  
  if (typeof collection.dimension !== 'number' || collection.dimension <= 0) {
    throw new ValidationError('Dimension must be a positive number');
  }
  
  const validMetrics: SimilarityMetric[] = ['cosine', 'euclidean', 'dot_product'];
  if (!validMetrics.includes(collection.similarity_metric)) {
    throw new ValidationError(`Invalid similarity metric. Must be one of: ${validMetrics.join(', ')}`);
  }
}

/**
 * Validates create collection request.
 * 
 * @param request - Create collection request to validate
 * @throws {Error} If request data is invalid
 */
export function validateCreateCollectionRequest(request: CreateCollectionRequest): void {
  if (!request.name || typeof request.name !== 'string') {
    throw new ValidationError('Collection name must be a non-empty string');
  }
  
  if (typeof request.dimension !== 'number' || request.dimension <= 0) {
    throw new ValidationError('Dimension must be a positive number');
  }
  
  if (request.similarity_metric) {
    const validMetrics: SimilarityMetric[] = ['cosine', 'euclidean', 'dot_product'];
    if (!validMetrics.includes(request.similarity_metric)) {
      throw new ValidationError(`Invalid similarity metric. Must be one of: ${validMetrics.join(', ')}`);
    }
  }
}
