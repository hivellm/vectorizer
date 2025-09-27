/**
 * Storage error for storage operation issues.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class StorageError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'STORAGE_ERROR', details);
    this.name = 'StorageError';
  }
}
