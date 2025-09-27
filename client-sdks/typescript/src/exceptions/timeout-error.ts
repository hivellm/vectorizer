/**
 * Timeout error for request timeouts.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class TimeoutError extends VectorizerError {
  constructor(message: string = 'Request timeout', details?: ErrorDetails) {
    super(message, 'TIMEOUT_ERROR', details);
    this.name = 'TimeoutError';
  }
}
