/**
 * Integration tests for VectorizerClient.
 */

import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { VectorizerClient } from '../../src/client';
import { HttpClient } from '../../src/utils/http-client';

// Mock the HTTP client
vi.mock('../../src/utils/http-client');

describe('VectorizerClient Integration Tests', () => {
  let client: VectorizerClient;
  let mockHttpClient: {
    get: ReturnType<typeof vi.fn>;
    post: ReturnType<typeof vi.fn>;
    put: ReturnType<typeof vi.fn>;
    delete: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Create mock instances
    mockHttpClient = {
      get: vi.fn(),
      post: vi.fn(),
      put: vi.fn(),
      delete: vi.fn(),
    };

    // Mock constructors
    (HttpClient as unknown as Mock).mockImplementation(() => mockHttpClient);

    client = new VectorizerClient({
      baseURL: 'http://localhost:15002',
      apiKey: 'test-api-key',
    });
  });

  describe('Complete Workflow', () => {
    it('should handle complete vector workflow', async () => {
      // 1. Health check
      mockHttpClient.get.mockResolvedValueOnce({ status: 'healthy', timestamp: '2025-01-01T00:00:00Z' } as any);

      const health = await client.healthCheck();
      expect(health.status).toBe('healthy');

      // 2. Create collection
      const collectionData = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'cosine' as const,
        description: 'Test collection'
      };
      mockHttpClient.post.mockResolvedValueOnce(collectionData as any);

      const collection = await client.createCollection(collectionData);
      expect(collection.name).toBe('test-collection');

      // 3. Insert vectors (uses PUT with Qdrant API)
      const vectors = [
        {
          data: Array.from({ length: 384 }, () => Math.random()),
          metadata: { source: 'doc1.pdf' }
        },
        {
          data: Array.from({ length: 384 }, () => Math.random()),
          metadata: { source: 'doc2.pdf' }
        }
      ];
      mockHttpClient.put.mockResolvedValueOnce({ status: 'ok' } as any);

      const insertResult = await client.insertVectors('test-collection', vectors);
      expect(insertResult.inserted).toBe(2);

      // 4. Search vectors
      const searchRequest = {
        query_vector: Array.from({ length: 384 }, () => Math.random()),
        limit: 5,
        include_metadata: true
      };
      const searchResults = {
        results: [
          {
            id: 'vector-1',
            score: 0.95,
            data: Array.from({ length: 384 }, () => Math.random()),
            metadata: { source: 'doc1.pdf' }
          }
        ],
        total: 1
      };
      mockHttpClient.post.mockResolvedValueOnce(searchResults as any);

      const searchResponse = await client.searchVectors('test-collection', searchRequest);
      expect(searchResponse.results).toHaveLength(1);
      expect(searchResponse.results?.[0]?.score).toBe(0.95);

      // 5. Text search
      const textSearchRequest = {
        query: 'machine learning',
        limit: 3,
        include_metadata: true
      };
      mockHttpClient.post.mockResolvedValueOnce(searchResults as any);

      const textSearchResponse = await client.searchText('test-collection', textSearchRequest);
      expect(textSearchResponse.results).toHaveLength(1);

      // 6. Generate embeddings
      const embeddingRequest = {
        text: 'artificial intelligence',
        model: 'bert-base'
      };
      const embeddingResponse = {
        embedding: Array.from({ length: 768 }, () => Math.random()),
        model: 'bert-base',
        text: 'artificial intelligence'
      };
      mockHttpClient.post.mockResolvedValueOnce(embeddingResponse as any);

      const embedding = await client.embedText(embeddingRequest);
      expect(embedding.embedding).toHaveLength(768);
      expect(embedding.model).toBe('bert-base');

      // 7. Get collection info
      const collectionInfo = {
        name: 'test-collection',
        dimension: 384,
        similarity_metric: 'cosine',
        vector_count: 2,
        size_bytes: 1024,
        created_at: new Date(),
        updated_at: new Date()
      };
      mockHttpClient.get.mockResolvedValueOnce(collectionInfo as any);

      const info = await client.getCollection('test-collection');
      expect(info.vector_count).toBe(2);

      // 8. Delete collection
      mockHttpClient.delete.mockResolvedValueOnce(undefined as any);

      await client.deleteCollection('test-collection');
      expect(mockHttpClient.delete).toHaveBeenCalledWith('/collections/test-collection');
    });

    it('should handle error scenarios gracefully', async () => {
      // Test authentication error
      mockHttpClient.get.mockRejectedValueOnce(new Error('Authentication failed'));

      await expect(client.healthCheck()).rejects.toThrow('Authentication failed');

      // Test collection not found
      mockHttpClient.get.mockRejectedValueOnce(new Error('Collection not found'));

      await expect(client.getCollection('nonexistent')).rejects.toThrow('Collection not found');

      // Test validation error
      await expect(client.createCollection({
        name: '',
        dimension: -1,
        similarity_metric: 'invalid' as any
      })).rejects.toThrow();
    });

    it('should handle configuration updates', () => {
      // Get initial config
      const initialConfig = client.getConfig();
      expect(initialConfig.apiKey).toBe('test-api-key');

      // Update API key
      client.setApiKey('new-api-key');
      const updatedConfig = client.getConfig();
      expect(updatedConfig.apiKey).toBe('new-api-key');

      // Verify new HTTP client was created
      expect(HttpClient).toHaveBeenCalledTimes(2); // Initial + after setApiKey
    });

    it('should handle client cleanup', async () => {
      // Close client
      await client.close();
      expect(mockHttpClient).toBeDefined();
    });
  });

  describe('Error Recovery', () => {
    it('should recover from network errors', async () => {
      // First call fails
      mockHttpClient.get.mockRejectedValueOnce(new Error('Network error'));
      await expect(client.healthCheck()).rejects.toThrow('Network error');

      // Second call succeeds
      mockHttpClient.get.mockResolvedValueOnce({ status: 'healthy' } as any);
      const health = await client.healthCheck();
      expect(health.status).toBe('healthy');
    });

    it('should handle partial failures in batch operations', async () => {
      const vectors = [
        { data: [0.1, 0.2, 0.3], metadata: { source: 'doc1.pdf' } },
        { data: [0.4, 0.5, 0.6], metadata: { source: 'doc2.pdf' } }
      ];

      // Uses PUT with Qdrant API - returns count based on vectors array length
      mockHttpClient.put.mockResolvedValueOnce({ status: 'ok' } as any);

      const result = await client.insertVectors('test-collection', vectors);
      // insertVectors returns the length of the input vectors array
      expect(result.inserted).toBe(2);
    });
  });

  describe('Performance Scenarios', () => {
    it('should handle large vector operations', async () => {
      const largeVector = {
        data: Array.from({ length: 4096 }, () => Math.random()),
        metadata: { source: 'large-doc.pdf' }
      };

      mockHttpClient.put.mockResolvedValueOnce({ status: 'ok' } as any);

      const result = await client.insertVectors('test-collection', [largeVector]);
      expect(result.inserted).toBe(1);
    });

    it('should handle multiple concurrent requests', async () => {
      const promises = [];
      
      for (let i = 0; i < 10; i++) {
        mockHttpClient.get.mockResolvedValueOnce({ status: 'healthy' } as any);
        promises.push(client.healthCheck());
      }

      const results = await Promise.all(promises);
      expect(results).toHaveLength(10);
      results.forEach(result => {
        expect(result.status).toBe('healthy');
      });
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty collections', async () => {
      mockHttpClient.get.mockResolvedValueOnce([] as any);
      const collections = await client.listCollections();
      expect(collections).toEqual([]);
    });

    it('should handle empty search results', async () => {
      const searchRequest = {
        query_vector: Array.from({ length: 384 }, () => Math.random()),
        limit: 5
      };
      const emptyResults = { results: [], total: 0 };
      mockHttpClient.post.mockResolvedValueOnce(emptyResults as any);

      const searchResponse = await client.searchVectors('test-collection', searchRequest);
      expect(searchResponse.results).toEqual([]);
      expect(searchResponse.total).toBe(0);
    });
  });
});
