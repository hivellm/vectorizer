/**
 * Unit tests for the SDK tier-control surface (phase13):
 * `deleteByFilter`, `bulkUpdateMetadata`, `copyVectors`, `setVectorExpiry`
 * on `VectorsClient`, and `reencodeCollection`, `setCollectionTtl` on
 * `CollectionsClient`.
 *
 * Verifies the wire shape (URL + body) the SDK emits, plus that the
 * server's response decodes into the expected report types.
 * Pattern mirrors `tests/tier-demotion.test.ts`.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { VectorsClient } from '../src/client/vectors';
import { CollectionsClient } from '../src/client/collections';
import type { DeleteByFilterReport, BulkUpdateReport, CopyReport, ReencodeJob } from '../src/models/tier-control';

interface MockTransport {
  get: ReturnType<typeof vi.fn>;
  post: ReturnType<typeof vi.fn>;
  put: ReturnType<typeof vi.fn>;
  patch: ReturnType<typeof vi.fn>;
  delete: ReturnType<typeof vi.fn>;
  postFormData: ReturnType<typeof vi.fn>;
}

function createMockTransport(): MockTransport {
  return {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    patch: vi.fn(),
    delete: vi.fn(),
    postFormData: vi.fn(),
  };
}

// ---------------------------------------------------------------------------
// VectorsClient — tier control
// ---------------------------------------------------------------------------

describe('VectorsClient — tier control (phase13)', () => {
  let mockTransport: MockTransport;
  let client: VectorsClient;

  beforeEach(() => {
    mockTransport = createMockTransport();
    client = new VectorsClient({ transport: mockTransport as never });
  });

  describe('deleteByFilter', () => {
    it('posts to /collections/{c}/vectors/delete_by_filter with filter body', async () => {
      const serverReply: DeleteByFilterReport = {
        scanned: 100,
        matched: 3,
        deleted: 2,
        results: [
          { id: 'vec-1', status: 'deleted' },
          { id: 'vec-2', status: 'deleted' },
          { id: 'vec-3', status: 'error', error: 'not found' },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const filter = { must: [{ key: 'tier', match: { value: 'cold' } }] };
      const report = await client.deleteByFilter('hot-collection', filter);

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/hot-collection/vectors/delete_by_filter',
        { filter },
      );
      expect(report.scanned).toBe(100);
      expect(report.matched).toBe(3);
      expect(report.deleted).toBe(2);
      expect(report.results).toHaveLength(3);
    });

    it('returns the full report from server', async () => {
      const serverReply: DeleteByFilterReport = {
        scanned: 50,
        matched: 50,
        deleted: 50,
        results: [],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.deleteByFilter('my-col', { must: [] });
      expect(report).toEqual(serverReply);
    });
  });

  describe('bulkUpdateMetadata', () => {
    it('posts to /collections/{c}/vectors/bulk_update_metadata with filter and patch', async () => {
      const serverReply: BulkUpdateReport = {
        scanned: 50,
        matched: 5,
        updated: 5,
        results: [{ id: 'vec-1', status: 'updated' }],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const filter = { must: [{ key: 'source', match: { value: 'legacy' } }] };
      const patch = { migrated: true, source: null };
      const report = await client.bulkUpdateMetadata('my-collection', filter, patch);

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/my-collection/vectors/bulk_update_metadata',
        { filter, patch },
      );
      expect(report.updated).toBe(5);
      expect(report.matched).toBe(5);
      expect(report.scanned).toBe(50);
    });

    it('surfaces per-id results without aborting the batch', async () => {
      const serverReply: BulkUpdateReport = {
        scanned: 10,
        matched: 3,
        updated: 2,
        results: [
          { id: 'v1', status: 'updated' },
          { id: 'v2', status: 'updated' },
          { id: 'v3', status: 'error', error: 'locked' },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.bulkUpdateMetadata('col', {}, {});
      expect(report.updated).toBe(2);
      expect(Array.isArray(report.results)).toBe(true);
      expect(report.results).toHaveLength(3);
    });
  });

  describe('copyVectors', () => {
    it('posts to /collections/{src}/vectors/copy with destination and ids', async () => {
      const serverReply: CopyReport = {
        src: 'hot',
        dst: 'cold',
        requested: 3,
        copied: 2,
        failed: 1,
        results: [
          { id: 'v1', status: 'ok' },
          { id: 'v2', status: 'ok' },
          { id: 'v3', status: 'missing_in_src', error: 'not found' },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.copyVectors('hot', 'cold', ['v1', 'v2', 'v3']);

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/hot/vectors/copy',
        { destination: 'cold', ids: ['v1', 'v2', 'v3'] },
      );
      expect(report.src).toBe('hot');
      expect(report.dst).toBe('cold');
      expect(report.copied).toBe(2);
      expect(report.failed).toBe(1);
    });

    it('returns per-id statuses in results array', async () => {
      const serverReply: CopyReport = {
        src: 'src',
        dst: 'dst',
        requested: 2,
        copied: 1,
        failed: 1,
        results: [
          { id: 'v1', status: 'ok' },
          { id: 'v2', status: 'dst_insert_failed', error: 'dimension mismatch' },
        ],
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const report = await client.copyVectors('src', 'dst', ['v1', 'v2']);
      const statuses = report.results.map((r) => r.status);
      expect(statuses).toEqual(['ok', 'dst_insert_failed']);
    });
  });

  describe('setVectorExpiry', () => {
    it('patches /collections/{c}/vectors/{id}/expiry with expires_at', async () => {
      mockTransport.patch.mockResolvedValue(undefined);

      await client.setVectorExpiry('my-col', 'vec-123', 1893456000000);

      expect(mockTransport.patch).toHaveBeenCalledTimes(1);
      expect(mockTransport.patch).toHaveBeenCalledWith(
        '/collections/my-col/vectors/vec-123/expiry',
        { expires_at: 1893456000000 },
      );
    });

    it('sends null to clear an existing expiry', async () => {
      mockTransport.patch.mockResolvedValue(undefined);

      await client.setVectorExpiry('my-col', 'vec-123', null);

      expect(mockTransport.patch).toHaveBeenCalledWith(
        '/collections/my-col/vectors/vec-123/expiry',
        { expires_at: null },
      );
    });
  });
});

// ---------------------------------------------------------------------------
// CollectionsClient — tier control
// ---------------------------------------------------------------------------

describe('CollectionsClient — tier control (phase13)', () => {
  let mockTransport: MockTransport;
  let client: CollectionsClient;

  beforeEach(() => {
    mockTransport = createMockTransport();
    client = new CollectionsClient({ transport: mockTransport as never });
  });

  describe('reencodeCollection', () => {
    it('posts to /collections/{c}/reencode with target_encoding', async () => {
      const serverReply: ReencodeJob = {
        job_id: 'reencode-my-col-1746000000',
        collection: 'my-col',
        state: 'completed',
        target_encoding: 'sq8',
        progress: 1.0,
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const job = await client.reencodeCollection('my-col', 'sq8');

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/my-col/reencode',
        { target_encoding: 'sq8' },
      );
      expect(job.collection).toBe('my-col');
      expect(job.state).toBe('completed');
      expect(job.target_encoding).toBe('sq8');
      expect(job.progress).toBeCloseTo(1.0);
    });

    it('returns the full ReencodeJob from the server', async () => {
      const serverReply: ReencodeJob = {
        job_id: 'reencode-x-99',
        collection: 'x',
        state: 'running',
        target_encoding: 'binary',
        progress: 0.42,
      };
      mockTransport.post.mockResolvedValue(serverReply);

      const job = await client.reencodeCollection('x', 'binary');
      expect(job).toEqual(serverReply);
    });
  });

  describe('setCollectionTtl', () => {
    it('posts to /collections/{c}/ttl with ttl_secs', async () => {
      mockTransport.post.mockResolvedValue(undefined);

      await client.setCollectionTtl('my-col', 3600);

      expect(mockTransport.post).toHaveBeenCalledTimes(1);
      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/my-col/ttl',
        { ttl_secs: 3600 },
      );
    });

    it('sends null to clear the collection TTL', async () => {
      mockTransport.post.mockResolvedValue(undefined);

      await client.setCollectionTtl('my-col', null);

      expect(mockTransport.post).toHaveBeenCalledWith(
        '/collections/my-col/ttl',
        { ttl_secs: null },
      );
    });
  });
});
