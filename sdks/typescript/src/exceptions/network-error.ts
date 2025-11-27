/**
 * Network error for connection and communication issues.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class NetworkError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'NETWORK_ERROR', details);
    this.name = 'NetworkError';
  }
}

