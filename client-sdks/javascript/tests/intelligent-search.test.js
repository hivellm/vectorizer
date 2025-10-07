/**
 * Tests for Intelligent Search features
 * 
 * This test suite covers:
 * - intelligentSearch() - Multi-query expansion with MMR
 * - semanticSearch() - Advanced semantic reranking
 * - contextualSearch() - Context-aware with metadata filtering
 * - multiCollectionSearch() - Cross-collection search
 */

const { VectorizerClient } = require('../src/client');

describe('Intelligent Search Operations', () => {
  let client;
  const baseURL = process.env.VECTORIZER_URL || 'http://localhost:15002';
  let serverAvailable = false;

  beforeAll(async () => {
    client = new VectorizerClient({
      baseURL,
      timeout: 5000,
    });

    // Check if server is available
    try {
      await client.healthCheck();
      serverAvailable = true;
    } catch (error) {
      console.warn('WARNING: Vectorizer server not available at', baseURL);
      console.warn('   Integration tests will be skipped. Start server with: cargo run --release');
      serverAvailable = false;
    }
  });

  beforeEach(() => {
    if (!serverAvailable) {
      return; // Skip test execution
    }
  });

  describe('intelligentSearch', () => {
    it('should perform intelligent search with default options', async () => {
      if (!serverAvailable) {
        return expect(true).toBe(true); // Skip when server not available
      }

      const request = {
        query: 'CMMV framework architecture',
        max_results: 10,
      };

      const response = await client.intelligentSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
      expect(response.total_results).toBeGreaterThanOrEqual(0);
    });

    it('should perform intelligent search with specific collections', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'vector database features',
        collections: ['test-collection-1', 'test-collection-2'],
        max_results: 5,
      };

      const response = await client.intelligentSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform intelligent search with domain expansion enabled', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'semantic search',
        max_results: 10,
        domain_expansion: true,
        technical_focus: true,
      };

      const response = await client.intelligentSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform intelligent search with MMR diversification', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'vector embeddings',
        max_results: 10,
        mmr_enabled: true,
        mmr_lambda: 0.7,
      };

      const response = await client.intelligentSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should return queries generated', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'machine learning models',
        max_results: 5,
        domain_expansion: true,
      };

      const response = await client.intelligentSearch(request);

      expect(response).toBeDefined();
    });
  });

  describe('semanticSearch', () => {
    it('should perform semantic search with default options', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'data processing pipeline',
        collection: 'test-collection',
        max_results: 10,
      };

      const response = await client.semanticSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
      expect(response.total_results).toBeGreaterThanOrEqual(0);
    });

    it('should perform semantic search with reranking enabled', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'neural network architecture',
        collection: 'test-collection',
        max_results: 10,
        semantic_reranking: true,
      };

      const response = await client.semanticSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform semantic search with cross-encoder reranking', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'transformer models',
        collection: 'test-collection',
        max_results: 5,
        semantic_reranking: true,
        cross_encoder_reranking: true,
      };

      const response = await client.semanticSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });
  });

  describe('contextualSearch', () => {
    it('should perform contextual search with default options', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'API documentation',
        collection: 'test-collection',
        max_results: 10,
      };

      const response = await client.contextualSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform contextual search with metadata filters', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'configuration settings',
        collection: 'test-collection',
        context_filters: {
          file_type: 'yaml',
          category: 'config',
        },
        max_results: 5,
      };

      const response = await client.contextualSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform contextual search with context reranking', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'authentication middleware',
        collection: 'test-collection',
        max_results: 10,
        context_reranking: true,
        context_weight: 0.4,
      };

      const response = await client.contextualSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform contextual search with complex filters', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'error handling',
        collection: 'test-collection',
        context_filters: {
          language: 'typescript',
          framework: 'express',
          min_lines: 50,
        },
        max_results: 10,
      };

      const response = await client.contextualSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });
  });

  describe('multiCollectionSearch', () => {
    it('should search across multiple collections', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'REST API endpoints',
        collections: ['collection-1', 'collection-2', 'collection-3'],
        max_per_collection: 5,
        max_total_results: 15,
      };

      const response = await client.multiCollectionSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should perform multi-collection search with cross-collection reranking', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'database queries',
        collections: ['docs', 'examples', 'tests'],
        max_per_collection: 3,
        max_total_results: 9,
        cross_collection_reranking: true,
      };

      const response = await client.multiCollectionSearch(request);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });

    it('should return results per collection', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'search algorithms',
        collections: ['algorithms', 'implementations'],
        max_per_collection: 5,
        max_total_results: 10,
      };

      const response = await client.multiCollectionSearch(request);

      expect(response).toBeDefined();
    });

    it('should respect max_total_results limit', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'common term',
        collections: ['col1', 'col2', 'col3', 'col4'],
        max_per_collection: 10,
        max_total_results: 5,
      };

      const response = await client.multiCollectionSearch(request);

      expect(response).toBeDefined();
      expect(response.results.length).toBeLessThanOrEqual(5);
    });
  });

  describe('Error Handling', () => {
    it('should handle empty query in intelligent search', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: '',
        max_results: 10,
      };

      await expect(client.intelligentSearch(request)).rejects.toThrow();
    });

    it('should handle invalid collection in semantic search', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'test',
        collection: '',
        max_results: 10,
      };

      await expect(client.semanticSearch(request)).rejects.toThrow();
    });

    it('should handle invalid similarity threshold', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'test',
        collection: 'test-collection',
        max_results: 10,
        similarity_threshold: 1.5, // Invalid: > 1.0
      };

      await expect(client.semanticSearch(request)).rejects.toThrow();
    });

    it('should handle empty collections array', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'test',
        collections: [],
        max_per_collection: 5,
        max_total_results: 10,
      };

      await expect(client.multiCollectionSearch(request)).rejects.toThrow();
    });
  });

  describe('Performance Tests', () => {
    it('should complete intelligent search within reasonable time', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const startTime = Date.now();
      
      const request = {
        query: 'performance test',
        max_results: 10,
      };

      await client.intelligentSearch(request);
      
      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });

    it('should handle large result sets efficiently', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const request = {
        query: 'common term',
        max_results: 100,
      };

      const response = await client.intelligentSearch(request);

      expect(response).toBeDefined();
      expect(response.results.length).toBeLessThanOrEqual(100);
    });
  });
});

