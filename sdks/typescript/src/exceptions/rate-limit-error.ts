/**
 * Rate limit error for API rate limiting.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class RateLimitError extends VectorizerError {
  constructor(message: string = 'Rate limit exceeded', details?: ErrorDetails) {
    super(message, 'RATE_LIMIT_ERROR', details);
    this.name = 'RateLimitError';
  }
}
