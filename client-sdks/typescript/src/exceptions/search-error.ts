/**
 * Search error for search operation issues.
 */

import { VectorizerError, ErrorDetails } from './vectorizer-error';

export class SearchError extends VectorizerError {
  constructor(message: string, details?: ErrorDetails) {
    super(message, 'SEARCH_ERROR', details);
    this.name = 'SearchError';
  }
}

