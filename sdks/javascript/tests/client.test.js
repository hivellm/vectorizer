/**
 * Tests for the VectorizerClient class.
 */

import { VectorizerClient } from '../src/client.js';
import { HttpClient } from '../src/utils/http-client.js';

// Mock the HTTP client
jest.mock('../src/utils/http-client.js');

describe('VectorizerClient', () => {
  let client;
  let mockHttpClient;

  beforeEach(() => {
    // Reset mocks
    jest.clearAllMocks();

    // Create mock instances
    mockHttpClient = {
      get: jest.fn(),
      post: jest.fn(),
      put: jest.fn(),
      delete: jest.fn(),
    };

    // Mock constructors
    HttpClient.mockImplementation(() => mockHttpClient);

    // Create client
    client = new VectorizerClient({
      baseURL: 'http://localhost:15002',
      apiKey: 'test-api-key',
    });
  });

  describe('constructor', () => {
    it('should create client with default config', () => {
      const defaultClient = new VectorizerClient();
      expect(defaultClient).toBeInstanceOf(VectorizerClient);
    });

    it('should create client with custom config', () => {
      const customClient = new VectorizerClient({
        baseURL: 'http://custom:8080',
        apiKey: 'custom-key',
        timeout: 60000,
      });
      expect(customClient).toBeInstanceOf(VectorizerClient);
    });
  });

  describe('healthCheck', () => {
    it('should return health status', async () => {
      const mockResponse = { status: 'healthy', timestamp: '2025-01-01T00:00:00Z' };
      mockHttpClient.get.mockResolvedValue(mockResponse);

      const result = await client.healthCheck();

      expect(mockHttpClient.get).toHaveBeenCalledWith('/health');
      expect(result).toEqual(mockResponse);
    });

    it('should handle errors', async () => {
      const error = new Error('Connection failed');
      mockHttpClient.get.mockRejectedValue(error);

      await expect(client.healthCheck()).rejects.toThrow('Connection failed');
    });
  });

  describe('getDatabaseStats', () => {
    it('should return database statistics', async () => {
      const mockResponse = {
        total_collections: 2,
        total_vectors: 100,
        total_size_bytes: 1024000,
        uptime_seconds: 3600,
        collections: [],
      };
      mockHttpClient.get.mockResolvedValue(mockResponse);

      const result = await client.getDatabaseStats();

      expect(mockHttpClient.get).toHaveBeenCalledWith('/stats');
      expect(result).toEqual(mockResponse);
    });
  });

  describe('listCollections', () => {
    it('should return list of collections', async () => {
      const mockResponse = [
        { name: 'collection1', dimension: 384, similarity_metric: 'cosine' },
        { name: 'collection2', dimension: 768, similarity_metric: 'euclidean' },
      ];
      mockHttpClient.get.mockResolvedValue(mockResponse);

      const result = await client.listCollections();

      expect(mockHttpClient.get).toHaveBeenCalledWith('/collections');
      expect(result).toEqual(mockResponse);
    });
  });

  describe('createCollection', () => {
    it('should create a new collection', async () => {
      const request = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'cosine',
        description: 'Test collection',
      };
      const mockResponse = { ...request, created_at: new Date() };
      mockHttpClient.post.mockResolvedValue(mockResponse);

      const result = await client.createCollection(request);

      expect(mockHttpClient.post).toHaveBeenCalledWith('/collections', request);
      expect(result).toEqual(mockResponse);
    });

    it('should validate collection request', async () => {
      const invalidRequest = {
        name: '',
        dimension: -1,
        similarity_metric: 'invalid',
      };

      await expect(client.createCollection(invalidRequest)).rejects.toThrow();
    });
  });

  describe('insertVectors', () => {
    it('should insert vectors into collection', async () => {
      const vectors = [
        { data: [0.1, 0.2, 0.3], metadata: { source: 'doc1' } },
        { data: [0.4, 0.5, 0.6], metadata: { source: 'doc2' } },
      ];
      const mockResponse = { inserted: 2 };
      mockHttpClient.post.mockResolvedValue(mockResponse);

      const result = await client.insertVectors('test-collection', vectors);

      expect(mockHttpClient.post).toHaveBeenCalledWith(
        '/collections/test-collection/vectors',
        { vectors }
      );
      expect(result).toEqual(mockResponse);
    });

    it('should validate vector data', async () => {
      const invalidVectors = [
        { data: [], metadata: { source: 'doc1' } }, // Empty data
      ];

      await expect(client.insertVectors('test-collection', invalidVectors)).rejects.toThrow();
    });
  });

  describe('searchVectors', () => {
    it('should search for similar vectors', async () => {
      const request = {
        query_vector: [0.1, 0.2, 0.3],
        limit: 5,
        threshold: 0.8,
        include_metadata: true,
      };
      const mockResponse = {
        results: [
          { id: '1', score: 0.95, data: [0.1, 0.2, 0.3], metadata: { source: 'doc1' } },
        ],
        total: 1,
      };
      mockHttpClient.post.mockResolvedValue(mockResponse);

      const result = await client.searchVectors('test-collection', request);

      expect(mockHttpClient.post).toHaveBeenCalledWith(
        '/collections/test-collection/search',
        request
      );
      expect(result).toEqual(mockResponse);
    });

    it('should validate search request', async () => {
      const invalidRequest = {
        query_vector: [], // Empty vector
        limit: -1, // Invalid limit
      };

      await expect(client.searchVectors('test-collection', invalidRequest)).rejects.toThrow();
    });
  });

  describe('searchText', () => {
    it('should search using text query', async () => {
      const request = {
        query: 'machine learning',
        limit: 5,
        include_metadata: true,
      };
      const mockResponse = {
        results: [
          { id: '1', score: 0.92, data: [0.1, 0.2, 0.3], metadata: { source: 'doc1' } },
        ],
        total: 1,
      };
      mockHttpClient.post.mockResolvedValue(mockResponse);

      const result = await client.searchText('test-collection', request);

      expect(mockHttpClient.post).toHaveBeenCalledWith(
        '/collections/test-collection/search/text',
        request
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('embedText', () => {
    it('should generate text embeddings', async () => {
      const request = {
        text: 'machine learning algorithms',
        model: 'bert-base',
      };
      const mockResponse = {
        embedding: Array.from({ length: 768 }, () => Math.random()),
        model: 'bert-base',
        text: 'machine learning algorithms',
      };
      mockHttpClient.post.mockResolvedValue(mockResponse);

      const result = await client.embedText(request);

      expect(mockHttpClient.post).toHaveBeenCalledWith('/embed', request);
      expect(result).toEqual(mockResponse);
    });
  });


  describe('utility methods', () => {
    it('should get configuration', () => {
      const config = client.getConfig();

      expect(config).toHaveProperty('baseURL');
      expect(config).toHaveProperty('apiKey');
      expect(config).toHaveProperty('timeout');
    });

    it('should update API key', () => {
      const newApiKey = 'new-api-key';

      client.setApiKey(newApiKey);

      expect(client.getConfig().apiKey).toBe(newApiKey);
    });

    it('should close client', async () => {
      await client.close();

      expect(mockHttpClient).toBeDefined();
    });
  });
});
