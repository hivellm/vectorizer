/**
 * Validation error for invalid input data.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class ValidationError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'VALIDATION_ERROR', details);
    this.name = 'ValidationError';
  }
}
