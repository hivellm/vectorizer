/**
 * Configuration error for invalid configuration.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class ConfigurationError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'CONFIGURATION_ERROR', details);
    this.name = 'ConfigurationError';
  }
}

