/**
 * Unit tests for the SDK tier-demotion surface (issue #265):
 * `deleteVector`, `deleteVectors`, `moveToCollection`.
 *
 * Verifies the wire shape (URL + body) the SDK emits, plus that the
 * server's response decodes into the expected report types.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { VectorsClient } from '../src/client/vectors';
import type { DeleteReport, MoveReport } from '../src/models/tier-demotion';

interface MockTransport {
  get: ReturnType<typeof vi.fn>;
  post: ReturnType<typeof vi.fn>;
  put: ReturnType<typeof vi.fn>;
  delete: ReturnType<typeof vi.fn>;
}

function createMockTransport(): MockTransport {
  return {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  };
}

describe('VectorsClient — tier demotion (issue #265)', () => {
  let mockTransport: MockTransport;
  let client: VectorsClient;

  beforeEach(() => {
    mockTransport = createMockTransport();
    // Inject the mock transport via the BaseClient `transport` config knob.
    client = new VectorsClient({ transport: mockTransport as never });
  });

  describe('deleteVector', () => {
    it('issues DELETE /collections/{c}/vectors/{id}', async () => {
      mockTransport.delete.mockResolvedValue(undefined);

      await client.deleteVector('cortex.consolidation.fp32', 'vec-1');

      expect(mockTransport.delete).toHaveBeenCalledTimes(1);
      expect(mockTransport.delete).toHaveBeenCalledWith(
        '/collections/cortex.consolidation.fp32/vectors/vec-1',
      );
    });
  });

  describe('deleteVectors', () => {
    it('posts to /batch_delete with collection + ids and decodes DeleteReport', async () => {
      const serverReply: DeleteReport = {
        collection: 'cortex.consolidation.fp32',
        count: 3,
        deleted: 2,
        failed: 1,
        results: [
          { id: 'vec-1', status: 'ok', index: 0 },
          { id: 'vec-2', status: 'ok', index: 1 },
          { id: 'missing', status: 'error', error: 'not found', index: 2 },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.deleteVectors('cortex.consolidation.fp32', [
        'vec-1',
        'vec-2',
        'missing',
      ]);

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith('/batch_delete', {
        collection: 'cortex.consolidation.fp32',
        ids: ['vec-1', 'vec-2', 'missing'],
      });
      expect(report).toEqual(serverReply);
      expect(report.deleted).toBe(2);
      expect(report.results[2].status).toBe('error');
    });
  });

  describe('moveToCollection', () => {
    it('posts to /collections/{src}/vectors/move with destination + ids', async () => {
      const serverReply: MoveReport = {
        src: 'cortex.consolidation.fp32',
        dst: 'cortex.consolidation.pq',
        requested: 2,
        moved: 2,
        failed: 0,
        results: [
          { id: 'vec-1', status: 'ok' },
          { id: 'vec-2', status: 'ok' },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.moveToCollection(
        'cortex.consolidation.fp32',
        'cortex.consolidation.pq',
        ['vec-1', 'vec-2'],
      );

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/cortex.consolidation.fp32/vectors/move',
        { destination: 'cortex.consolidation.pq', ids: ['vec-1', 'vec-2'] },
      );
      expect(report).toEqual(serverReply);
      expect(report.moved).toBe(2);
    });

    it('surfaces per-id failures in results without aborting', async () => {
      const serverReply: MoveReport = {
        src: 'src',
        dst: 'dst',
        requested: 3,
        moved: 1,
        failed: 2,
        results: [
          { id: 'vec-1', status: 'ok' },
          { id: 'vec-missing', status: 'missing_in_src', error: 'not found' },
          {
            id: 'vec-bad-dim',
            status: 'dst_insert_failed',
            error: 'dimension mismatch',
          },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.moveToCollection('src', 'dst', [
        'vec-1',
        'vec-missing',
        'vec-bad-dim',
      ]);

      const statuses = report.results.map((r) => r.status);
      expect(statuses).toEqual(['ok', 'missing_in_src', 'dst_insert_failed']);
      expect(report.moved).toBe(1);
      expect(report.failed).toBe(2);
    });
  });
});
