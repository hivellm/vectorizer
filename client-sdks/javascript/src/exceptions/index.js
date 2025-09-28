/**
 * Exception classes for the Hive Vectorizer SDK.
 * 
 * This module contains all custom exceptions used throughout the SDK
 * for proper error handling and debugging.
 */

export { VectorizerError } from './vectorizer-error.js';

export class AuthenticationError extends VectorizerError {
  constructor(message = 'Authentication failed', details = {}) {
    super(message, 'AUTH_ERROR', details);
    this.name = 'AuthenticationError';
  }
}

export class CollectionNotFoundError extends VectorizerError {
  constructor(collectionName, details = {}) {
    super(`Collection '${collectionName}' not found`, 'COLLECTION_NOT_FOUND', {
      collectionName,
      ...details,
    });
    this.name = 'CollectionNotFoundError';
  }
}

export class ValidationError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'VALIDATION_ERROR', details);
    this.name = 'ValidationError';
  }
}

export class NetworkError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'NETWORK_ERROR', details);
    this.name = 'NetworkError';
  }
}

export class ServerError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'SERVER_ERROR', details);
    this.name = 'ServerError';
  }
}

export class TimeoutError extends VectorizerError {
  constructor(message = 'Request timeout', details = {}) {
    super(message, 'TIMEOUT_ERROR', details);
    this.name = 'TimeoutError';
  }
}

export class RateLimitError extends VectorizerError {
  constructor(message = 'Rate limit exceeded', details = {}) {
    super(message, 'RATE_LIMIT_ERROR', details);
    this.name = 'RateLimitError';
  }
}

export class ConfigurationError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'CONFIGURATION_ERROR', details);
    this.name = 'ConfigurationError';
  }
}

export class EmbeddingError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'EMBEDDING_ERROR', details);
    this.name = 'EmbeddingError';
  }
}

export class SearchError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'SEARCH_ERROR', details);
    this.name = 'SearchError';
  }
}

export class StorageError extends VectorizerError {
  constructor(message, details = {}) {
    super(message, 'STORAGE_ERROR', details);
    this.name = 'StorageError';
  }
}


