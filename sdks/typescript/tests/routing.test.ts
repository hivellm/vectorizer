/**
 * Tests for Master/Replica routing functionality
 */

import { VectorizerClient } from '../src/client';
import { ReadPreference } from '../src/types';

// Mock fetch for testing
global.fetch = jest.fn();

describe('Master/Replica Routing', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => ({ success: true }),
      text: async () => 'OK',
      status: 200,
    });
  });

  describe('Operation Classification', () => {
    it('should route write operations to master', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001', 'http://replica2:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      // Test insertTexts (write operation)
      await client.insertTexts('test-collection', [
        { id: '1', text: 'test', metadata: {} },
      ]);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('http://master:15001'),
        expect.any(Object)
      );
    });

    it('should route read operations based on preference', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001', 'http://replica2:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [] }),
        status: 200,
      });

      // Test search (read operation)
      await client.searchVectors('test-collection', [0.1, 0.2, 0.3]);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringMatching(/http:\/\/replica[12]:15001/),
        expect.any(Object)
      );
    });

    it('should classify all write operations correctly', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      const writeOps = [
        () => client.insertTexts('test', [{ id: '1', text: 'test', metadata: {} }]),
        () => client.insertVectors('test', [{ id: '1', vector: [0.1], metadata: {} }]),
        () => client.updateVector('test', '1', { vector: [0.2] }),
        () => client.deleteVector('test', '1'),
        () => client.createCollection('new-collection', { dimension: 512 }),
        () => client.deleteCollection('test'),
      ];

      for (const op of writeOps) {
        (global.fetch as jest.Mock).mockClear();
        await op();
        expect(global.fetch).toHaveBeenCalledWith(
          expect.stringContaining('http://master:15001'),
          expect.any(Object)
        );
      }
    });

    it('should classify all read operations correctly', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [], collections: [] }),
        status: 200,
      });

      const readOps = [
        () => client.searchVectors('test', [0.1]),
        () => client.listCollections(),
        () => client.getCollectionInfo('test'),
      ];

      for (const op of readOps) {
        (global.fetch as jest.Mock).mockClear();
        await op();
        expect(global.fetch).toHaveBeenCalledWith(
          expect.stringContaining('http://replica1:15001'),
          expect.any(Object)
        );
      }
    });
  });

  describe('Round-Robin Load Balancing', () => {
    it('should distribute reads across replicas evenly', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: [
            'http://replica1:15001',
            'http://replica2:15001',
            'http://replica3:15001',
          ],
        },
        readPreference: ReadPreference.Replica,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [] }),
        status: 200,
      });

      const calls: string[] = [];

      for (let i = 0; i < 6; i++) {
        await client.searchVectors('test', [0.1]);
        const lastCall = (global.fetch as jest.Mock).mock.calls[i];
        calls.push(lastCall[0]);
      }

      // Verify each replica was called exactly twice
      expect(calls.filter(url => url.includes('replica1')).length).toBe(2);
      expect(calls.filter(url => url.includes('replica2')).length).toBe(2);
      expect(calls.filter(url => url.includes('replica3')).length).toBe(2);

      // Verify sequential distribution
      expect(calls[0]).toContain('replica1');
      expect(calls[1]).toContain('replica2');
      expect(calls[2]).toContain('replica3');
      expect(calls[3]).toContain('replica1');
      expect(calls[4]).toContain('replica2');
      expect(calls[5]).toContain('replica3');
    });
  });

  describe('Read Preference', () => {
    it('should respect readPreference: master', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001'],
        },
        readPreference: ReadPreference.Master,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [] }),
        status: 200,
      });

      await client.searchVectors('test', [0.1]);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('http://master:15001'),
        expect.any(Object)
      );
    });

    it('should respect readPreference: replica', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [] }),
        status: 200,
      });

      await client.searchVectors('test', [0.1]);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('http://replica1:15001'),
        expect.any(Object)
      );
    });
  });

  describe('Read Preference Override', () => {
    it('should allow per-operation override to master', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [] }),
        status: 200,
      });

      // First call without override - should go to replica
      await client.searchVectors('test', [0.1]);
      expect((global.fetch as jest.Mock).mock.calls[0][0]).toContain('replica1');

      // Second call with override - should go to master
      await client.searchVectors('test', [0.1], { readPreference: ReadPreference.Master });
      expect((global.fetch as jest.Mock).mock.calls[1][0]).toContain('master');

      // Third call without override - should go back to replica
      await client.searchVectors('test', [0.1]);
      expect((global.fetch as jest.Mock).mock.calls[2][0]).toContain('replica1');
    });

    it('should support withMaster context', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ results: [], success: true }),
        status: 200,
      });

      await client.withMaster(async (masterClient) => {
        // Both operations should go to master
        await masterClient.insertTexts('test', [{ id: '1', text: 'test', metadata: {} }]);
        await masterClient.searchVectors('test', [0.1]);

        expect((global.fetch as jest.Mock).mock.calls[0][0]).toContain('master');
        expect((global.fetch as jest.Mock).mock.calls[1][0]).toContain('master');
      });

      // Operation outside context should go to replica
      (global.fetch as jest.Mock).mockClear();
      await client.searchVectors('test', [0.1]);
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('replica1'),
        expect.any(Object)
      );
    });
  });

  describe('Backward Compatibility', () => {
    it('should work with single baseURL configuration', async () => {
      const client = new VectorizerClient({
        baseURL: 'http://localhost:15001',
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ success: true }),
        status: 200,
      });

      // All operations should go to the single URL
      await client.insertTexts('test', [{ id: '1', text: 'test', metadata: {} }]);
      expect((global.fetch as jest.Mock).mock.calls[0][0]).toContain('localhost:15001');

      await client.searchVectors('test', [0.1]);
      expect((global.fetch as jest.Mock).mock.calls[1][0]).toContain('localhost:15001');
    });

    it('should not break existing code', async () => {
      // This test ensures backward compatibility
      const oldStyleClient = new VectorizerClient({
        baseURL: 'http://localhost:15001',
        apiKey: 'test-key',
      });

      (global.fetch as jest.Mock).mockResolvedValue({
        ok: true,
        json: async () => ({ success: true }),
        status: 200,
      });

      // Should work exactly as before
      await oldStyleClient.createCollection('test', { dimension: 512 });
      expect(global.fetch).toHaveBeenCalled();
    });
  });

  describe('Error Handling', () => {
    it('should fallback to next replica on failure', async () => {
      const client = new VectorizerClient({
        hosts: {
          master: 'http://master:15001',
          replicas: ['http://replica1:15001', 'http://replica2:15001'],
        },
        readPreference: ReadPreference.Replica,
      });

      // First replica fails, second succeeds
      (global.fetch as jest.Mock)
        .mockRejectedValueOnce(new Error('Connection refused'))
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ results: [] }),
          status: 200,
        });

      await expect(client.searchVectors('test', [0.1])).resolves.toBeDefined();

      // Should have tried replica1 first, then replica2
      expect(global.fetch).toHaveBeenCalledTimes(2);
    });
  });
});

