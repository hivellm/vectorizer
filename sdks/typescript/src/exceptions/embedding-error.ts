/**
 * Embedding error for embedding generation issues.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class EmbeddingError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'EMBEDDING_ERROR', details);
    this.name = 'EmbeddingError';
  }
}
