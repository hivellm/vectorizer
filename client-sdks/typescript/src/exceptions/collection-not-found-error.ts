/**
 * Collection not found error.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class CollectionNotFoundError extends VectorizerError {
  constructor(collectionName: string, details?: ErrorDetails) {
    super(`Collection '${collectionName}' not found`, 'COLLECTION_NOT_FOUND', {
      collectionName,
      ...details,
    });
    this.name = 'CollectionNotFoundError';
  }
}

