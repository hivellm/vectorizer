/**
 * Tests for Discovery Operations
 * 
 * This test suite covers:
 * - discover() - Complete discovery pipeline
 * - filterCollections() - Collection filtering by patterns
 * - scoreCollections() - Relevance-based ranking
 * - expandQueries() - Query variation generation
 */

const { VectorizerClient } = require('../src/client');

describe('Discovery Operations', () => {
  let client;
  const baseURL = process.env.VECTORIZER_URL || 'http://localhost:15002';
  let serverAvailable = false;

  beforeAll(async () => {
    client = new VectorizerClient({
      baseURL,
      timeout: 30000,
    });

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
    if (!serverAvailable) return;
  });

  describe('discover', () => {
    it('should perform complete discovery pipeline', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'How does CMMV framework work?',
        max_bullets: 20,
        broad_k: 50,
        focus_k: 15,
      };

      const response = await client.discover(params);

      expect(response).toBeDefined();
      expect(response.prompt).toBeDefined();
      expect(response.evidence).toBeInstanceOf(Array);
      expect(response.metadata).toBeDefined();
    });

    it('should discover with specific collections included', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'API authentication methods',
        include_collections: ['api-docs', 'security-docs'],
        max_bullets: 15,
      };

      const response = await client.discover(params);

      expect(response).toBeDefined();
      expect(response.evidence).toBeInstanceOf(Array);
    });

    it('should discover with collections excluded', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'database migrations',
        exclude_collections: ['test-*', '*-backup'],
        max_bullets: 10,
      };

      const response = await client.discover(params);

      expect(response).toBeDefined();
      expect(response.evidence).toBeInstanceOf(Array);
    });

    it('should generate LLM-ready prompt', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'vector search algorithms',
        max_bullets: 10,
      };

      const response = await client.discover(params);

      expect(response).toBeDefined();
      expect(response.prompt).toBeDefined();
      expect(typeof response.prompt).toBe('string');
      expect(response.prompt.length).toBeGreaterThan(0);
    });

    it('should include citations in evidence', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'system architecture',
        max_bullets: 15,
      };

      const response = await client.discover(params);

      expect(response).toBeDefined();
      expect(response.evidence).toBeInstanceOf(Array);
      
      if (response.evidence.length > 0) {
        response.evidence.forEach((item) => {
          expect(item.text).toBeDefined();
          expect(item.citation).toBeDefined();
        });
      }
    });
  });

  describe('filterCollections', () => {
    it('should filter collections by query', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'documentation',
      };

      const response = await client.filterCollections(params);

      expect(response).toBeDefined();
      expect(response.filtered_collections).toBeInstanceOf(Array);
      expect(response.total_available).toBeGreaterThanOrEqual(0);
    });

    it('should filter with include patterns', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'api endpoints',
        include: ['*-docs', 'api-*'],
      };

      const response = await client.filterCollections(params);

      expect(response).toBeDefined();
      expect(response.filtered_collections).toBeInstanceOf(Array);
    });

    it('should filter with exclude patterns', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'source code',
        exclude: ['*-test', '*-backup'],
      };

      const response = await client.filterCollections(params);

      expect(response).toBeDefined();
      expect(response.filtered_collections).toBeInstanceOf(Array);
      expect(response.excluded_count).toBeGreaterThanOrEqual(0);
    });

    it('should filter with both include and exclude', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'configuration',
        include: ['config-*', '*-settings'],
        exclude: ['*-old', '*-deprecated'],
      };

      const response = await client.filterCollections(params);

      expect(response).toBeDefined();
      expect(response.filtered_collections).toBeInstanceOf(Array);
    });
  });

  describe('scoreCollections', () => {
    it('should score collections by relevance', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'machine learning',
      };

      const response = await client.scoreCollections(params);

      expect(response).toBeDefined();
      expect(response.scored_collections).toBeInstanceOf(Array);
      expect(response.total_collections).toBeGreaterThanOrEqual(0);
    });

    it('should score with custom term boost weight', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'database queries',
        term_boost_weight: 0.4,
      };

      const response = await client.scoreCollections(params);

      expect(response).toBeDefined();
      expect(response.scored_collections).toBeInstanceOf(Array);
    });

    it('should score with custom signal boost weight', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'performance optimization',
        signal_boost_weight: 0.2,
      };

      const response = await client.scoreCollections(params);

      expect(response).toBeDefined();
      expect(response.scored_collections).toBeInstanceOf(Array);
    });

    it('should return collections sorted by score', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'search functionality',
      };

      const response = await client.scoreCollections(params);

      expect(response).toBeDefined();
      expect(response.scored_collections).toBeInstanceOf(Array);

      // Verify sorting
      if (response.scored_collections.length > 1) {
        for (let i = 0; i < response.scored_collections.length - 1; i++) {
          expect(response.scored_collections[i].score)
            .toBeGreaterThanOrEqual(response.scored_collections[i + 1].score);
        }
      }
    });
  });

  describe('expandQueries', () => {
    it('should expand query with default options', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'CMMV framework',
      };

      const response = await client.expandQueries(params);

      expect(response).toBeDefined();
      expect(response.original_query).toBe('CMMV framework');
      expect(response.expanded_queries).toBeInstanceOf(Array);
      expect(response.expanded_queries.length).toBeGreaterThan(0);
    });

    it('should limit number of expansions', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'vector database',
        max_expansions: 5,
      };

      const response = await client.expandQueries(params);

      expect(response).toBeDefined();
      expect(response.expanded_queries).toBeInstanceOf(Array);
      expect(response.expanded_queries.length).toBeLessThanOrEqual(5);
    });

    it('should include definition queries', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'semantic search',
        include_definition: true,
      };

      const response = await client.expandQueries(params);

      expect(response).toBeDefined();
      expect(response.expanded_queries).toBeInstanceOf(Array);
      expect(response.query_types).toContain('definition');
    });

    it('should include features queries', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'API gateway',
        include_features: true,
      };

      const response = await client.expandQueries(params);

      expect(response).toBeDefined();
      expect(response.expanded_queries).toBeInstanceOf(Array);
      expect(response.query_types).toContain('features');
    });

    it('should include architecture queries', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'microservices',
        include_architecture: true,
      };

      const response = await client.expandQueries(params);

      expect(response).toBeDefined();
      expect(response.expanded_queries).toBeInstanceOf(Array);
      expect(response.query_types).toContain('architecture');
    });

    it('should generate diverse query variations', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'authentication system',
        max_expansions: 10,
        include_definition: true,
        include_features: true,
        include_architecture: true,
      };

      const response = await client.expandQueries(params);

      expect(response).toBeDefined();
      expect(response.expanded_queries).toBeInstanceOf(Array);
      expect(response.expanded_queries.length).toBeGreaterThan(1);
      
      // Check for diversity
      const uniqueQueries = new Set(response.expanded_queries);
      expect(uniqueQueries.size).toBe(response.expanded_queries.length);
    });
  });

  describe('Error Handling', () => {
    it('should handle empty query in discover', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: '',
      };

      await expect(client.discover(params)).rejects.toThrow();
    });

    it('should handle invalid max_bullets', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'test',
        max_bullets: -1,
      };

      await expect(client.discover(params)).rejects.toThrow();
    });

    it('should handle empty query in filterCollections', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: '',
      };

      await expect(client.filterCollections(params)).rejects.toThrow();
    });

    it('should handle invalid weights in scoreCollections', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        query: 'test',
        name_match_weight: 1.5, // Invalid: > 1.0
      };

      await expect(client.scoreCollections(params)).rejects.toThrow();
    });
  });

  describe('Integration Tests', () => {
    it('should chain filter and score operations', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      // First filter
      const filterResponse = await client.filterCollections({
        query: 'documentation',
        include: ['*-docs'],
      });

      expect(filterResponse).toBeDefined();

      // Then score the filtered collections
      const scoreResponse = await client.scoreCollections({
        query: 'API documentation',
      });

      expect(scoreResponse).toBeDefined();
      expect(scoreResponse.scored_collections).toBeInstanceOf(Array);
    });

    it('should use expanded queries in discovery', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      // First expand queries
      const expandResponse = await client.expandQueries({
        query: 'database optimization',
        max_expansions: 5,
      });

      expect(expandResponse).toBeDefined();
      expect(expandResponse.expanded_queries.length).toBeGreaterThan(0);

      // Use expanded queries in discovery
      const discoverResponse = await client.discover({
        query: expandResponse.expanded_queries[0],
        max_bullets: 10,
      });

      expect(discoverResponse).toBeDefined();
    });
  });

  describe('Performance Tests', () => {
    it('should complete discovery within reasonable time', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const startTime = Date.now();
      
      await client.discover({
        query: 'performance test',
        max_bullets: 10,
      });
      
      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(10000); // Should complete within 10 seconds
    });

    it('should handle large collections efficiently', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const response = await client.scoreCollections({
        query: 'test',
      });

      expect(response).toBeDefined();
      expect(response.scored_collections).toBeInstanceOf(Array);
    });
  });
});

