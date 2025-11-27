/**
 * Tests for Qdrant advanced features (1.14.x)
 * 
 * Tests for Snapshots, Sharding, Cluster Management, Query API, Search Groups/Matrix
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { VectorizerClient } from '../src/client';

const TEST_SERVER_URL = process.env.VECTORIZER_URL || 'http://localhost:15002';
const client = new VectorizerClient({ baseURL: TEST_SERVER_URL });

describe('Qdrant Snapshots API', () => {
  it('should list collection snapshots', async () => {
    try {
      const result = await client.qdrantListCollectionSnapshots('test_collection');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        // Skip if server not running
        return;
      }
      throw error;
    }
  });

  it('should create collection snapshot', async () => {
    try {
      const result = await client.qdrantCreateCollectionSnapshot('test_collection');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should delete collection snapshot', async () => {
    try {
      const result = await client.qdrantDeleteCollectionSnapshot('test_collection', 'test_snapshot');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should recover collection snapshot', async () => {
    try {
      const result = await client.qdrantRecoverCollectionSnapshot('test_collection', 'snapshots/test.snapshot');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should list all snapshots', async () => {
    try {
      const result = await client.qdrantListAllSnapshots();
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should create full snapshot', async () => {
    try {
      const result = await client.qdrantCreateFullSnapshot();
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });
});

describe('Qdrant Sharding API', () => {
  it('should list shard keys', async () => {
    try {
      const result = await client.qdrantListShardKeys('test_collection');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should create shard key', async () => {
    try {
      const shardKey = { shard_key: 'test_key' };
      const result = await client.qdrantCreateShardKey('test_collection', shardKey);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should delete shard key', async () => {
    try {
      const shardKey = { shard_key: 'test_key' };
      const result = await client.qdrantDeleteShardKey('test_collection', shardKey);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });
});

describe('Qdrant Cluster Management API', () => {
  it('should get cluster status', async () => {
    try {
      const result = await client.qdrantGetClusterStatus();
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should recover cluster', async () => {
    try {
      const result = await client.qdrantClusterRecover();
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should remove peer', async () => {
    try {
      const result = await client.qdrantRemovePeer('test_peer_123');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should list metadata keys', async () => {
    try {
      const result = await client.qdrantListMetadataKeys();
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should get metadata key', async () => {
    try {
      const result = await client.qdrantGetMetadataKey('test_key');
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should update metadata key', async () => {
    try {
      const value = { value: 'test_value' };
      const result = await client.qdrantUpdateMetadataKey('test_key', value);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });
});

describe('Qdrant Query API', () => {
  it('should query points', async () => {
    try {
      const request = {
        query: {
          vector: Array(384).fill(0).map((_, i) => i * 0.001)
        },
        limit: 10
      };
      const result = await client.qdrantQueryPoints('test_collection', request);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should batch query points', async () => {
    try {
      const request = {
        searches: [
          {
            query: {
              vector: Array(384).fill(0).map((_, i) => i * 0.001)
            },
            limit: 10
          }
        ]
      };
      const result = await client.qdrantBatchQueryPoints('test_collection', request);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should query points groups', async () => {
    try {
      const request = {
        query: {
          vector: Array(384).fill(0).map((_, i) => i * 0.001)
        },
        group_by: 'category',
        group_size: 3,
        limit: 10
      };
      const result = await client.qdrantQueryPointsGroups('test_collection', request);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });
});

describe('Qdrant Search Groups and Matrix API', () => {
  it('should search points groups', async () => {
    try {
      const request = {
        vector: Array(384).fill(0).map((_, i) => i * 0.001),
        group_by: 'category',
        group_size: 3,
        limit: 10
      };
      const result = await client.qdrantSearchPointsGroups('test_collection', request);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should search matrix pairs', async () => {
    try {
      const request = {
        sample: 10,
        limit: 5
      };
      const result = await client.qdrantSearchMatrixPairs('test_collection', request);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should search matrix offsets', async () => {
    try {
      const request = {
        sample: 10,
        limit: 5
      };
      const result = await client.qdrantSearchMatrixOffsets('test_collection', request);
      expect(result).toBeDefined();
      expect(typeof result).toBe('object');
    } catch (error: any) {
      if (error.message?.includes('ECONNREFUSED') || error.message?.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });
});

