/**
 * Tests for VectorizerError and all exception classes.
 */

import {
  VectorizerError,
  AuthenticationError,
  CollectionNotFoundError,
  ValidationError,
  NetworkError,
  ServerError,
  TimeoutError,
  RateLimitError,
  ConfigurationError,
  EmbeddingError,
  SearchError,
  StorageError,
} from '../../src/exceptions/index.js';

describe('Exception Classes', () => {
  describe('VectorizerError', () => {
    it('should create error with message only', () => {
      const error = new VectorizerError('Test error');
      
      expect(error.message).toBe('Test error');
      expect(error.name).toBe('VectorizerError');
      expect(error.errorCode).toBeUndefined();
      expect(error.details).toEqual({});
    });

    it('should create error with message and error code', () => {
      const error = new VectorizerError('Test error', 'TEST_ERROR');
      
      expect(error.message).toBe('Test error');
      expect(error.name).toBe('VectorizerError');
      expect(error.errorCode).toBe('TEST_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create error with message, error code, and details', () => {
      const details = { field: 'test', value: 123 };
      const error = new VectorizerError('Test error', 'TEST_ERROR', details);
      
      expect(error.message).toBe('Test error');
      expect(error.name).toBe('VectorizerError');
      expect(error.errorCode).toBe('TEST_ERROR');
      expect(error.details).toEqual(details);
    });

    it('should return correct string representation with error code', () => {
      const error = new VectorizerError('Test error', 'TEST_ERROR');
      
      expect(error.toString()).toBe('[TEST_ERROR] Test error');
    });

    it('should return correct string representation without error code', () => {
      const error = new VectorizerError('Test error');
      
      expect(error.toString()).toBe('Test error');
    });

    it('should be instance of Error', () => {
      const error = new VectorizerError('Test error');
      
      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(VectorizerError);
    });
  });

  describe('AuthenticationError', () => {
    it('should create authentication error with default message', () => {
      const error = new AuthenticationError();
      
      expect(error.message).toBe('Authentication failed');
      expect(error.name).toBe('AuthenticationError');
      expect(error.errorCode).toBe('AUTH_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create authentication error with custom message', () => {
      const error = new AuthenticationError('Invalid API key');
      
      expect(error.message).toBe('Invalid API key');
      expect(error.name).toBe('AuthenticationError');
      expect(error.errorCode).toBe('AUTH_ERROR');
    });

    it('should create authentication error with details', () => {
      const details = { apiKey: 'invalid-key' };
      const error = new AuthenticationError('Invalid API key', details);
      
      expect(error.message).toBe('Invalid API key');
      expect(error.details).toEqual(details);
    });

    it('should be instance of VectorizerError', () => {
      const error = new AuthenticationError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(AuthenticationError);
    });
  });

  describe('CollectionNotFoundError', () => {
    it('should create collection not found error with default message', () => {
      const error = new CollectionNotFoundError();
      
      expect(error.message).toBe('Collection not found');
      expect(error.name).toBe('CollectionNotFoundError');
      expect(error.errorCode).toBe('COLLECTION_NOT_FOUND');
      expect(error.details).toEqual({});
    });

    it('should create collection not found error with collection name', () => {
      const error = new CollectionNotFoundError('test-collection');
      
      expect(error.message).toBe("Collection 'test-collection' not found");
      expect(error.name).toBe('CollectionNotFoundError');
      expect(error.errorCode).toBe('COLLECTION_NOT_FOUND');
      expect(error.details).toEqual({ collectionName: 'test-collection' });
    });

    it('should create collection not found error with details', () => {
      const details = { collectionName: 'test-collection', available: ['col1', 'col2'] };
      const error = new CollectionNotFoundError('test-collection', details);
      
      expect(error.message).toBe("Collection 'test-collection' not found");
      expect(error.details).toEqual(details);
    });

    it('should be instance of VectorizerError', () => {
      const error = new CollectionNotFoundError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(CollectionNotFoundError);
    });
  });

  describe('ValidationError', () => {
    it('should create validation error with default message', () => {
      const error = new ValidationError();
      
      expect(error.message).toBe('Validation failed');
      expect(error.name).toBe('ValidationError');
      expect(error.errorCode).toBe('VALIDATION_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create validation error with custom message', () => {
      const error = new ValidationError('Invalid input data');
      
      expect(error.message).toBe('Invalid input data');
      expect(error.name).toBe('ValidationError');
      expect(error.errorCode).toBe('VALIDATION_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new ValidationError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(ValidationError);
    });
  });

  describe('NetworkError', () => {
    it('should create network error with default message', () => {
      const error = new NetworkError();
      
      expect(error.message).toBe('Network error');
      expect(error.name).toBe('NetworkError');
      expect(error.errorCode).toBe('NETWORK_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create network error with custom message', () => {
      const error = new NetworkError('Connection timeout');
      
      expect(error.message).toBe('Connection timeout');
      expect(error.name).toBe('NetworkError');
      expect(error.errorCode).toBe('NETWORK_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new NetworkError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(NetworkError);
    });
  });

  describe('ServerError', () => {
    it('should create server error with default message', () => {
      const error = new ServerError();
      
      expect(error.message).toBe('Server error');
      expect(error.name).toBe('ServerError');
      expect(error.errorCode).toBe('SERVER_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create server error with custom message', () => {
      const error = new ServerError('Internal server error');
      
      expect(error.message).toBe('Internal server error');
      expect(error.name).toBe('ServerError');
      expect(error.errorCode).toBe('SERVER_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new ServerError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(ServerError);
    });
  });

  describe('TimeoutError', () => {
    it('should create timeout error with default message', () => {
      const error = new TimeoutError();
      
      expect(error.message).toBe('Request timeout');
      expect(error.name).toBe('TimeoutError');
      expect(error.errorCode).toBe('TIMEOUT_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create timeout error with custom message', () => {
      const error = new TimeoutError('Operation timed out after 30 seconds');
      
      expect(error.message).toBe('Operation timed out after 30 seconds');
      expect(error.name).toBe('TimeoutError');
      expect(error.errorCode).toBe('TIMEOUT_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new TimeoutError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(TimeoutError);
    });
  });

  describe('RateLimitError', () => {
    it('should create rate limit error with default message', () => {
      const error = new RateLimitError();
      
      expect(error.message).toBe('Rate limit exceeded');
      expect(error.name).toBe('RateLimitError');
      expect(error.errorCode).toBe('RATE_LIMIT_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create rate limit error with custom message', () => {
      const error = new RateLimitError('Too many requests per minute');
      
      expect(error.message).toBe('Too many requests per minute');
      expect(error.name).toBe('RateLimitError');
      expect(error.errorCode).toBe('RATE_LIMIT_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new RateLimitError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(RateLimitError);
    });
  });

  describe('ConfigurationError', () => {
    it('should create configuration error with default message', () => {
      const error = new ConfigurationError();
      
      expect(error.message).toBe('Configuration error');
      expect(error.name).toBe('ConfigurationError');
      expect(error.errorCode).toBe('CONFIGURATION_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create configuration error with custom message', () => {
      const error = new ConfigurationError('Invalid configuration file');
      
      expect(error.message).toBe('Invalid configuration file');
      expect(error.name).toBe('ConfigurationError');
      expect(error.errorCode).toBe('CONFIGURATION_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new ConfigurationError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(ConfigurationError);
    });
  });

  describe('EmbeddingError', () => {
    it('should create embedding error with default message', () => {
      const error = new EmbeddingError();
      
      expect(error.message).toBe('Embedding generation failed');
      expect(error.name).toBe('EmbeddingError');
      expect(error.errorCode).toBe('EMBEDDING_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create embedding error with custom message', () => {
      const error = new EmbeddingError('Failed to generate embedding for text');
      
      expect(error.message).toBe('Failed to generate embedding for text');
      expect(error.name).toBe('EmbeddingError');
      expect(error.errorCode).toBe('EMBEDDING_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new EmbeddingError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(EmbeddingError);
    });
  });

  describe('SearchError', () => {
    it('should create search error with default message', () => {
      const error = new SearchError();
      
      expect(error.message).toBe('Search operation failed');
      expect(error.name).toBe('SearchError');
      expect(error.errorCode).toBe('SEARCH_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create search error with custom message', () => {
      const error = new SearchError('Failed to search vectors');
      
      expect(error.message).toBe('Failed to search vectors');
      expect(error.name).toBe('SearchError');
      expect(error.errorCode).toBe('SEARCH_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new SearchError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(SearchError);
    });
  });

  describe('StorageError', () => {
    it('should create storage error with default message', () => {
      const error = new StorageError();
      
      expect(error.message).toBe('Storage operation failed');
      expect(error.name).toBe('StorageError');
      expect(error.errorCode).toBe('STORAGE_ERROR');
      expect(error.details).toEqual({});
    });

    it('should create storage error with custom message', () => {
      const error = new StorageError('Failed to save vector to storage');
      
      expect(error.message).toBe('Failed to save vector to storage');
      expect(error.name).toBe('StorageError');
      expect(error.errorCode).toBe('STORAGE_ERROR');
    });

    it('should be instance of VectorizerError', () => {
      const error = new StorageError();
      
      expect(error).toBeInstanceOf(VectorizerError);
      expect(error).toBeInstanceOf(StorageError);
    });
  });
});
