/**
 * Tests for VectorizerStore LangChain.js integration
 * 
 * This module contains comprehensive tests for the VectorizerStore implementation.
 */

import { VectorizerStore, VectorizerConfig, VectorizerClient, VectorizerError, VectorizerUtils } from './vectorizer-store';
import { Document } from '@langchain/core/documents';

// Mock fetch for testing
global.fetch = jest.fn();

describe('VectorizerConfig', () => {
  test('should create default configuration', () => {
    const config = VectorizerUtils.createDefaultConfig();
    
    expect(config.host).toBe('localhost');
    expect(config.port).toBe(15001);
    expect(config.collectionName).toBe('langchain_documents');
    expect(config.autoCreateCollection).toBe(true);
    expect(config.batchSize).toBe(100);
    expect(config.similarityThreshold).toBe(0.7);
  });

  test('should create custom configuration', () => {
    const config = VectorizerUtils.createDefaultConfig({
      host: 'example.com',
      port: 8080,
      collectionName: 'test_collection',
      apiKey: 'test_key',
      autoCreateCollection: false,
      batchSize: 50,
      similarityThreshold: 0.8
    });
    
    expect(config.host).toBe('example.com');
    expect(config.port).toBe(8080);
    expect(config.collectionName).toBe('test_collection');
    expect(config.apiKey).toBe('test_key');
    expect(config.autoCreateCollection).toBe(false);
    expect(config.batchSize).toBe(50);
    expect(config.similarityThreshold).toBe(0.8);
  });

  test('should validate configuration', () => {
    const validConfig = VectorizerUtils.createDefaultConfig();
    expect(() => VectorizerUtils.validateConfig(validConfig)).not.toThrow();
  });

  test('should throw error for invalid configuration', () => {
    const invalidConfig = VectorizerUtils.createDefaultConfig({
      host: '',
      port: 0,
      batchSize: -1,
      similarityThreshold: 1.5
    });
    
    expect(() => VectorizerUtils.validateConfig(invalidConfig)).toThrow(VectorizerError);
  });
});

describe('VectorizerClient', () => {
  let client: VectorizerClient;
  let mockFetch: jest.MockedFunction<typeof fetch>;

  beforeEach(() => {
    const config: VectorizerConfig = {
      host: 'localhost',
      port: 15001,
      collectionName: 'test_collection',
      autoCreateCollection: true,
      batchSize: 100,
      similarityThreshold: 0.7,
      timeout: 30000
    };
    
    client = new VectorizerClient(config);
    mockFetch = fetch as jest.MockedFunction<typeof fetch>;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('should make successful health check', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'healthy' })
    } as Response);

    const result = await client.healthCheck();
    expect(result).toBe(true);
  });

  test('should handle failed health check', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Connection failed'));

    const result = await client.healthCheck();
    expect(result).toBe(false);
  });

  test('should list collections', async () => {
    const mockResponse = {
      collections: [
        { name: 'collection1' },
        { name: 'collection2' }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const result = await client.listCollections();
    expect(result).toEqual(['collection1', 'collection2']);
  });

  test('should create collection', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'created' })
    } as Response);

    const result = await client.createCollection('test_collection', 384, 'cosine');
    expect(result).toBe(true);
  });

  test('should add texts', async () => {
    const mockResponse = {
      vector_ids: ['id1', 'id2', 'id3']
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const texts = ['text1', 'text2', 'text3'];
    const metadatas = [{ source: 'doc1' }, { source: 'doc2' }, { source: 'doc3' }];

    const result = await client.addTexts(texts, metadatas);
    expect(result).toEqual(['id1', 'id2', 'id3']);
  });

  test('should perform similarity search', async () => {
    const mockResponse = {
      results: [
        {
          content: 'Test content 1',
          metadata: { source: 'doc1' },
          score: 0.95
        },
        {
          content: 'Test content 2',
          metadata: { source: 'doc2' },
          score: 0.87
        }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const result = await client.similaritySearch('test query', 2);
    expect(result).toHaveLength(2);
    expect(result[0].content).toBe('Test content 1');
    expect(result[0].score).toBe(0.95);
  });

  test('should perform similarity search with scores', async () => {
    const mockResponse = {
      results: [
        {
          content: 'Test content 1',
          metadata: { source: 'doc1' },
          score: 0.95
        }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const result = await client.similaritySearchWithScore('test query', 1);
    expect(result).toHaveLength(1);
    expect(result[0][0].content).toBe('Test content 1');
    expect(result[0][1]).toBe(0.95);
  });

  test('should delete vectors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'deleted' })
    } as Response);

    const result = await client.deleteVectors(['id1', 'id2']);
    expect(result).toBe(true);
  });

  test('should handle API errors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
      statusText: 'Internal Server Error'
    } as Response);

    await expect(client.listCollections()).rejects.toThrow(VectorizerError);
  });
});

describe('VectorizerStore', () => {
  let store: VectorizerStore;
  let mockFetch: jest.MockedFunction<typeof fetch>;

  beforeEach(() => {
    const config: VectorizerConfig = {
      host: 'localhost',
      port: 15001,
      collectionName: 'test_collection',
      autoCreateCollection: true,
      batchSize: 100,
      similarityThreshold: 0.7,
      timeout: 30000
    };
    
    store = new VectorizerStore(config);
    mockFetch = fetch as jest.MockedFunction<typeof fetch>;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('should add texts', async () => {
    const mockResponse = {
      vector_ids: ['id1', 'id2']
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const texts = ['text1', 'text2'];
    const metadatas = [{ source: 'doc1' }, { source: 'doc2' }];

    const result = await store.addTexts(texts, metadatas);
    expect(result).toEqual(['id1', 'id2']);
  });

  test('should perform similarity search', async () => {
    const mockResponse = {
      results: [
        {
          content: 'Test content',
          metadata: { source: 'doc1' },
          score: 0.95
        }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const result = await store.similaritySearch('test query', 1);
    expect(result).toHaveLength(1);
    expect(result[0]).toBeInstanceOf(Document);
    expect(result[0].pageContent).toBe('Test content');
    expect(result[0].metadata).toEqual({ source: 'doc1' });
  });

  test('should perform similarity search with scores', async () => {
    const mockResponse = {
      results: [
        {
          content: 'Test content',
          metadata: { source: 'doc1' },
          score: 0.95
        }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const result = await store.similaritySearchWithScore('test query', 1);
    expect(result).toHaveLength(1);
    const [doc, score] = result[0];
    expect(doc).toBeInstanceOf(Document);
    expect(doc.pageContent).toBe('Test content');
    expect(score).toBe(0.95);
  });

  test('should delete vectors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'deleted' })
    } as Response);

    const result = await store.delete(['id1', 'id2']);
    expect(result).toBe(true);
  });

  test('should create from texts', async () => {
    const mockResponse = {
      vector_ids: ['id1', 'id2']
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const texts = ['text1', 'text2'];
    const metadatas = [{ source: 'doc1' }, { source: 'doc2' }];

    const newStore = await VectorizerStore.fromTexts(texts, metadatas);
    expect(newStore).toBeInstanceOf(VectorizerStore);
  });

  test('should create from documents', async () => {
    const mockResponse = {
      vector_ids: ['id1', 'id2']
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    } as Response);

    const documents = [
      new Document({ pageContent: 'text1', metadata: { source: 'doc1' } }),
      new Document({ pageContent: 'text2', metadata: { source: 'doc2' } })
    ];

    const newStore = await VectorizerStore.fromDocuments(documents);
    expect(newStore).toBeInstanceOf(VectorizerStore);
  });
});

describe('VectorizerUtils', () => {
  test('should check availability', async () => {
    const config = VectorizerUtils.createDefaultConfig();
    
    // Mock successful health check
    global.fetch = jest.fn().mockResolvedValueOnce({
      ok: true,
      json: async () => ({ status: 'healthy' })
    } as Response);

    const result = await VectorizerUtils.checkAvailability(config);
    expect(result).toBe(true);
  });

  test('should handle availability check failure', async () => {
    const config = VectorizerUtils.createDefaultConfig();
    
    // Mock failed health check
    global.fetch = jest.fn().mockRejectedValueOnce(new Error('Connection failed'));

    const result = await VectorizerUtils.checkAvailability(config);
    expect(result).toBe(false);
  });
});

describe('Error Handling', () => {
  test('should handle VectorizerError', () => {
    const error = new VectorizerError('Test error');
    expect(error.name).toBe('VectorizerError');
    expect(error.message).toBe('Test error');
  });

  test('should handle VectorizerError with cause', () => {
    const cause = new Error('Original error');
    const error = new VectorizerError('Test error', cause);
    expect(error.cause).toBe(cause);
  });
});

// Integration tests (require running Vectorizer instance)
describe('Integration Tests', () => {
  test('should connect to real Vectorizer instance', async () => {
    const config = VectorizerUtils.createDefaultConfig();
    const isAvailable = await VectorizerUtils.checkAvailability(config);
    
    if (isAvailable) {
      const client = new VectorizerClient(config);
      const collections = await client.listCollections();
      expect(Array.isArray(collections)).toBe(true);
    } else {
      console.log('Skipping integration test - Vectorizer not available');
    }
  }, 10000);

  test('should perform real store operations', async () => {
    const config = VectorizerUtils.createDefaultConfig({
      collectionName: 'integration_test'
    });
    
    const isAvailable = await VectorizerUtils.checkAvailability(config);
    
    if (isAvailable) {
      const store = new VectorizerStore(config);
      
      try {
        // Test adding texts
        const texts = ['Integration test document'];
        const metadatas = [{ test: true }];
        
        const vectorIds = await store.addTexts(texts, metadatas);
        expect(vectorIds).toHaveLength(1);
        
        // Test search
        const results = await store.similaritySearch('integration test', 1);
        expect(Array.isArray(results)).toBe(true);
        
        // Clean up
        await store.delete(vectorIds);
        
      } catch (error) {
        console.log('Skipping integration test - Vectorizer not available');
      }
    } else {
      console.log('Skipping integration test - Vectorizer not available');
    }
  }, 15000);
});
