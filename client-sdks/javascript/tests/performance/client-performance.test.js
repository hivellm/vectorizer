/**
 * Performance tests for VectorizerClient.
 */

import { VectorizerClient } from '../../src/client.js';
import { HttpClient } from '../../src/utils/http-client.js';

// Mock the HTTP client
jest.mock('../../src/utils/http-client.js');

describe('VectorizerClient Performance Tests', () => {
  let client;
  let mockHttpClient;

  beforeEach(() => {
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

    client = new VectorizerClient({
      baseURL: 'http://localhost:15001',
      apiKey: 'test-api-key',
    });
  });

  describe('Batch Operations Performance', () => {
    it('should handle large batch vector insertion', async () => {
      const startTime = Date.now();
      const vectorCount = 1000;
      const dimension = 384;

      // Create large batch of vectors
      const vectors = Array.from({ length: vectorCount }, (_, i) => ({
        data: Array.from({ length: dimension }, () => Math.random()),
        metadata: { id: i, source: `doc-${i}.pdf` }
      }));

      mockHttpClient.post.mockResolvedValueOnce({ inserted: vectorCount });

      const result = await client.insertVectors('test-collection', vectors);
      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(result.inserted).toBe(vectorCount);
      expect(duration).toBeLessThan(1000); // Should complete within 1 second
      expect(mockHttpClient.post).toHaveBeenCalledTimes(1);
    });

    it('should handle concurrent vector insertions', async () => {
      const startTime = Date.now();
      const batchCount = 10;
      const vectorsPerBatch = 100;
      const dimension = 384;

      // Create multiple batches
      const batches = Array.from({ length: batchCount }, (_, batchIndex) =>
        Array.from({ length: vectorsPerBatch }, (_, vectorIndex) => ({
          data: Array.from({ length: dimension }, () => Math.random()),
          metadata: { 
            batch: batchIndex, 
            id: vectorIndex, 
            source: `doc-${batchIndex}-${vectorIndex}.pdf` 
          }
        }))
      );

      // Mock responses for all batches
      batches.forEach(() => {
        mockHttpClient.post.mockResolvedValueOnce({ inserted: vectorsPerBatch });
      });

      // Execute all batches concurrently
      const promises = batches.map(batch => 
        client.insertVectors('test-collection', batch)
      );

      const results = await Promise.all(promises);
      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(results).toHaveLength(batchCount);
      results.forEach(result => {
        expect(result.inserted).toBe(vectorsPerBatch);
      });
      expect(duration).toBeLessThan(2000); // Should complete within 2 seconds
      expect(mockHttpClient.post).toHaveBeenCalledTimes(batchCount);
    });

    it('should handle large search operations', async () => {
      const startTime = Date.now();
      const searchCount = 100;
      const dimension = 384;

      // Create multiple search requests
      const searchRequests = Array.from({ length: searchCount }, () => ({
        query_vector: Array.from({ length: dimension }, () => Math.random()),
        limit: 10,
        include_metadata: true
      }));

      // Mock responses for all searches
      const mockSearchResults = {
        results: Array.from({ length: 10 }, (_, i) => ({
          id: `result-${i}`,
          score: 0.9 - i * 0.01,
          data: Array.from({ length: dimension }, () => Math.random()),
          metadata: { source: `doc-${i}.pdf` }
        })),
        total: 10
      };

      searchRequests.forEach(() => {
        mockHttpClient.post.mockResolvedValueOnce(mockSearchResults);
      });

      // Execute all searches concurrently
      const promises = searchRequests.map(request => 
        client.searchVectors('test-collection', request)
      );

      const results = await Promise.all(promises);
      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(results).toHaveLength(searchCount);
      results.forEach(result => {
        expect(result.results).toHaveLength(10);
        expect(result.total).toBe(10);
      });
      expect(duration).toBeLessThan(3000); // Should complete within 3 seconds
      expect(mockHttpClient.post).toHaveBeenCalledTimes(searchCount);
    });
  });

  describe('Memory Usage Performance', () => {
    it('should handle large vector data efficiently', async () => {
      const dimension = 4096; // Large dimension
      const vectorCount = 100;

      // Create large vectors
      const vectors = Array.from({ length: vectorCount }, (_, i) => ({
        data: Array.from({ length: dimension }, () => Math.random()),
        metadata: { 
          id: i, 
          source: `large-doc-${i}.pdf`,
          description: 'A'.repeat(1000) // Large metadata
        }
      }));

      mockHttpClient.post.mockResolvedValueOnce({ inserted: vectorCount });

      const result = await client.insertVectors('test-collection', vectors);

      expect(result.inserted).toBe(vectorCount);
      // Memory usage should be reasonable (no explicit memory test, but should not crash)
    });

    it('should handle large search results efficiently', async () => {
      const dimension = 4096;
      const resultCount = 1000;

      const mockSearchResults = {
        results: Array.from({ length: resultCount }, (_, i) => ({
          id: `result-${i}`,
          score: 0.9 - i * 0.0001,
          data: Array.from({ length: dimension }, () => Math.random()),
          metadata: { 
            source: `doc-${i}.pdf`,
            description: 'B'.repeat(500) // Large metadata
          }
        })),
        total: resultCount
      };

      mockHttpClient.post.mockResolvedValueOnce(mockSearchResults);

      const searchRequest = {
        query_vector: Array.from({ length: dimension }, () => Math.random()),
        limit: resultCount,
        include_metadata: true
      };

      const result = await client.searchVectors('test-collection', searchRequest);

      expect(result.results).toHaveLength(resultCount);
      expect(result.total).toBe(resultCount);
      // Memory usage should be reasonable
    });
  });

  describe('Network Performance', () => {
    it('should handle high-frequency requests', async () => {
      const startTime = Date.now();
      const requestCount = 500;

      // Mock health check responses
      for (let i = 0; i < requestCount; i++) {
        mockHttpClient.get.mockResolvedValueOnce({ 
          status: 'healthy', 
          timestamp: new Date().toISOString() 
        });
      }

      // Execute all requests concurrently
      const promises = Array.from({ length: requestCount }, () => 
        client.healthCheck()
      );

      const results = await Promise.all(promises);
      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(results).toHaveLength(requestCount);
      results.forEach(result => {
        expect(result.status).toBe('healthy');
      });
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
      expect(mockHttpClient.get).toHaveBeenCalledTimes(requestCount);
    });
  });

  describe('Error Handling Performance', () => {
    it('should handle error scenarios efficiently', async () => {
      const startTime = Date.now();
      const errorCount = 100;

      // Mock error responses
      for (let i = 0; i < errorCount; i++) {
        mockHttpClient.get.mockRejectedValueOnce(new Error(`Error ${i}`));
      }

      // Execute all requests concurrently
      const promises = Array.from({ length: errorCount }, () => 
        client.healthCheck().catch(error => error)
      );

      const results = await Promise.all(promises);
      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(results).toHaveLength(errorCount);
      results.forEach((result, index) => {
        expect(result).toBeInstanceOf(Error);
        expect(result.message).toBe(`Error ${index}`);
      });
      expect(duration).toBeLessThan(2000); // Should complete within 2 seconds
    });

    it('should handle mixed success and error scenarios', async () => {
      const startTime = Date.now();
      const totalRequests = 200;
      const errorRate = 0.3; // 30% error rate
      const errorCount = Math.floor(totalRequests * errorRate);
      const successCount = totalRequests - errorCount;

      // Mock mixed responses
      for (let i = 0; i < successCount; i++) {
        mockHttpClient.get.mockResolvedValueOnce({ status: 'healthy' });
      }
      for (let i = 0; i < errorCount; i++) {
        mockHttpClient.get.mockRejectedValueOnce(new Error(`Error ${i}`));
      }

      // Execute all requests concurrently
      const promises = Array.from({ length: totalRequests }, () => 
        client.healthCheck().catch(error => error)
      );

      const results = await Promise.all(promises);
      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(results).toHaveLength(totalRequests);
      
      const successResults = results.filter(result => !(result instanceof Error));
      const errorResults = results.filter(result => result instanceof Error);

      expect(successResults).toHaveLength(successCount);
      expect(errorResults).toHaveLength(errorCount);
      expect(duration).toBeLessThan(3000); // Should complete within 3 seconds
    });
  });

  describe('Configuration Performance', () => {
    it('should handle frequent configuration updates', () => {
      const startTime = Date.now();
      const updateCount = 100;

      // Update API key multiple times
      for (let i = 0; i < updateCount; i++) {
        client.setApiKey(`api-key-${i}`);
      }

      const endTime = Date.now();
      const duration = endTime - startTime;

      expect(client.getConfig().apiKey).toBe(`api-key-${updateCount - 1}`);
      expect(duration).toBeLessThan(1000); // Should complete within 1 second
      expect(HttpClient).toHaveBeenCalledTimes(updateCount + 1); // Initial + updates
    });
  });
});
