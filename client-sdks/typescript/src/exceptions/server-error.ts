/**
 * Server error for server-side issues.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class ServerError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'SERVER_ERROR', details);
    this.name = 'ServerError';
  }
}

