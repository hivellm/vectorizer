/**
 * Authentication error for invalid credentials or API keys.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class AuthenticationError extends VectorizerError {
  constructor(message: string = 'Authentication failed', details?: ErrorDetails) {
    super(message, 'AUTH_ERROR', details);
    this.name = 'AuthenticationError';
  }
}

