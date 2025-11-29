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
  /** Similarity metric used for search (may be returned as 'metric' by API) */
  similarity_metric?: SimilarityMetric;
  /** Alternative field name for similarity metric from API */
  metric?: string;
  /** Optional description */
  description?: string;
  /** Creation timestamp */
  created_at?: string | Date;
  /** Last update timestamp */
  updated_at?: string | Date;
  /** Vector count */
  vector_count?: number;
  /** Document count */
  document_count?: number;
  /** Embedding provider */
  embedding_provider?: string;
  /** Indexing status */
  indexing_status?: Record<string, unknown>;
  /** Normalization config */
  normalization?: Record<string, unknown>;
  /** Quantization config */
  quantization?: Record<string, unknown>;
  /** Size info */
  size?: Record<string, unknown>;
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
export function validateCollection(collection: Collection & { collection?: string }): void {
  // API may return 'collection' instead of 'name'
  const name = collection.name || collection.collection;
  if (!name || typeof name !== 'string') {
    throw new ValidationError('Collection name must be a non-empty string');
  }
  // Normalize the name field
  if (!collection.name && collection.collection) {
    collection.name = collection.collection;
  }

  if (typeof collection.dimension !== 'number' || collection.dimension <= 0) {
    throw new ValidationError('Dimension must be a positive number');
  }

  // Check either similarity_metric or metric field (API may return either)
  const metric = collection.similarity_metric || collection.metric;
  if (metric) {
    const validMetrics = ['cosine', 'euclidean', 'dot_product', 'Cosine', 'Euclidean', 'DotProduct', 'Euclid'];
    if (!validMetrics.includes(metric)) {
      throw new ValidationError(`Invalid similarity metric. Must be one of: ${validMetrics.join(', ')}`);
    }
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
