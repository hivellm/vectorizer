/**
 * Unit tests for the SDK schema-evolution + observability surface (phase14):
 * - CollectionsClient: renameCollection, reindexCollection,
 *   snapshotCollectionNative, listCollectionSnapshotsNative,
 *   restoreCollectionSnapshotNative
 * - SearchClient: explainSearch
 * - AdminClient: listSlowQueries, setSlowQueryConfig
 *
 * Verifies wire shape (URL + body) and that server responses decode into
 * the expected model types. Pattern mirrors tests/tier-control.test.ts.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { CollectionsClient } from '../src/client/collections';
import { SearchClient } from '../src/client/search';
import { AdminClient } from '../src/client/admin';
import type {
  ExplainResponse,
  NativeSnapshotInfo,
  ReindexJob,
  SlowQueryConfig,
  SlowQueryEntry,
} from '../src/models/schema-evolution';

// ---------------------------------------------------------------------------
// Shared mock transport helper
// ---------------------------------------------------------------------------

interface MockTransport {
  get: ReturnType<typeof vi.fn>;
  post: ReturnType<typeof vi.fn>;
  put: ReturnType<typeof vi.fn>;
  patch: ReturnType<typeof vi.fn>;
  delete: ReturnType<typeof vi.fn>;
  postFormData: ReturnType<typeof vi.fn>;
}

function createMock(): MockTransport {
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
// CollectionsClient — phase14
// ---------------------------------------------------------------------------

describe('CollectionsClient — schema-evolution (phase14)', () => {
  let mock: MockTransport;
  let client: CollectionsClient;

  beforeEach(() => {
    mock = createMock();
    client = new CollectionsClient({ transport: mock as never });
  });

  // ── renameCollection ──────────────────────────────────────────────────────

  describe('renameCollection', () => {
    it('POSTs to /collections/{name}/rename with new_name in body', async () => {
      mock.post.mockResolvedValue({ old_name: 'docs', new_name: 'docs_v2', alias_retained: 'docs', status: 'ok' });
      await client.renameCollection('docs', 'docs_v2');
      expect(mock.post).toHaveBeenCalledWith(
        '/collections/docs/rename',
        { new_name: 'docs_v2' },
      );
    });

    it('resolves void on success', async () => {
      mock.post.mockResolvedValue({ status: 'ok' });
      await expect(client.renameCollection('col', 'col2')).resolves.toBeUndefined();
    });
  });

  // ── reindexCollection ─────────────────────────────────────────────────────

  describe('reindexCollection', () => {
    const serverResponse: ReindexJob = {
      job_id: 'reindex-docs-1746000000',
      collection: 'docs',
      state: 'completed',
      params: { m: 32, ef_construction: 400, ef_search: 200 },
      progress: 1.0,
    };

    it('POSTs correct body to /collections/{name}/reindex', async () => {
      mock.post.mockResolvedValue(serverResponse);
      await client.reindexCollection('docs', { m: 32, ef_construction: 400, ef_search: 200 });
      expect(mock.post).toHaveBeenCalledWith(
        '/collections/docs/reindex',
        { m: 32, ef_construction: 400, ef_search: 200 },
      );
    });

    it('returns a ReindexJob with state "completed"', async () => {
      mock.post.mockResolvedValue(serverResponse);
      const job = await client.reindexCollection('docs', { m: 32, ef_construction: 400, ef_search: 200 });
      expect(job.state).toBe('completed');
      expect(job.job_id).toContain('reindex');
      expect(job.progress).toBe(1.0);
    });
  });

  // ── snapshotCollectionNative ──────────────────────────────────────────────

  describe('snapshotCollectionNative', () => {
    const snapshotResponse: NativeSnapshotInfo & { status: string } = {
      id: 'snap-abc-123',
      collection: 'docs',
      created_at: '2026-05-02T00:00:00Z',
      size_bytes: 4096,
      status: 'ok',
    };

    it('POSTs empty body to /collections/{name}/snapshot', async () => {
      mock.post.mockResolvedValue(snapshotResponse);
      await client.snapshotCollectionNative('docs');
      expect(mock.post).toHaveBeenCalledWith('/collections/docs/snapshot', {});
    });

    it('returns NativeSnapshotInfo with correct fields', async () => {
      mock.post.mockResolvedValue(snapshotResponse);
      const info = await client.snapshotCollectionNative('docs');
      expect(info.id).toBe('snap-abc-123');
      expect(info.collection).toBe('docs');
      expect(info.size_bytes).toBe(4096);
      expect(info.created_at).toBe('2026-05-02T00:00:00Z');
    });
  });

  // ── listCollectionSnapshotsNative ─────────────────────────────────────────

  describe('listCollectionSnapshotsNative', () => {
    const listResponse = {
      collection: 'docs',
      snapshots: [
        { id: 'snap-1', collection: 'docs', created_at: '2026-05-02T00:00:00Z', size_bytes: 2048 },
        { id: 'snap-2', collection: 'docs', created_at: '2026-05-02T01:00:00Z', size_bytes: 3000 },
      ],
      total: 2,
    };

    it('GETs /collections/{name}/snapshots', async () => {
      mock.get.mockResolvedValue(listResponse);
      await client.listCollectionSnapshotsNative('docs');
      expect(mock.get).toHaveBeenCalledWith('/collections/docs/snapshots');
    });

    it('returns the snapshots array', async () => {
      mock.get.mockResolvedValue(listResponse);
      const snaps = await client.listCollectionSnapshotsNative('docs');
      expect(snaps).toHaveLength(2);
      expect(snaps[0].id).toBe('snap-1');
      expect(snaps[1].size_bytes).toBe(3000);
    });

    it('returns empty array when snapshots missing', async () => {
      mock.get.mockResolvedValue({ collection: 'docs', total: 0 });
      const snaps = await client.listCollectionSnapshotsNative('docs');
      expect(snaps).toEqual([]);
    });
  });

  // ── restoreCollectionSnapshotNative ──────────────────────────────────────

  describe('restoreCollectionSnapshotNative', () => {
    it('POSTs empty body to /collections/{name}/snapshots/{id}/restore', async () => {
      mock.post.mockResolvedValue({ collection: 'docs', snapshot_id: 'snap-1', status: 'restored' });
      await client.restoreCollectionSnapshotNative('docs', 'snap-1');
      expect(mock.post).toHaveBeenCalledWith(
        '/collections/docs/snapshots/snap-1/restore',
        {},
      );
    });

    it('resolves void on success', async () => {
      mock.post.mockResolvedValue({ status: 'restored' });
      await expect(client.restoreCollectionSnapshotNative('docs', 'snap-1')).resolves.toBeUndefined();
    });
  });
});

// ---------------------------------------------------------------------------
// SearchClient — phase14
// ---------------------------------------------------------------------------

describe('SearchClient — explainSearch (phase14)', () => {
  let mock: MockTransport;
  let client: SearchClient;

  beforeEach(() => {
    mock = createMock();
    client = new SearchClient({ transport: mock as never });
  });

  const explainResponse: ExplainResponse = {
    collection: 'docs',
    k: 10,
    results: [{ id: 'vec-1', score: 0.95, payload: null }],
    trace: {
      visited_nodes: 120,
      ef_search: 100,
      hnsw_search_ms: 1.23,
      payload_filter_evals: 0,
      quantization_score_ms: 0.45,
      total_ms: 2.10,
    },
  };

  it('POSTs vector and k to /collections/{name}/explain', async () => {
    mock.post.mockResolvedValue(explainResponse);
    await client.explainSearch('docs', [0.1, 0.2, 0.3], 10);
    expect(mock.post).toHaveBeenCalledWith(
      '/collections/docs/explain',
      { vector: [0.1, 0.2, 0.3], k: 10 },
    );
  });

  it('omits k from body when not provided', async () => {
    mock.post.mockResolvedValue(explainResponse);
    await client.explainSearch('docs', [0.1, 0.2]);
    const [, body] = mock.post.mock.calls[0] as [string, Record<string, unknown>];
    expect(body).not.toHaveProperty('k');
    expect(body['vector']).toEqual([0.1, 0.2]);
  });

  it('returns ExplainResponse with all trace fields', async () => {
    mock.post.mockResolvedValue(explainResponse);
    const resp = await client.explainSearch('docs', [0.1, 0.2, 0.3], 10);
    expect(resp.collection).toBe('docs');
    expect(resp.k).toBe(10);
    expect(resp.results).toHaveLength(1);
    expect(resp.trace.visited_nodes).toBe(120);
    expect(resp.trace.ef_search).toBe(100);
    expect(resp.trace.hnsw_search_ms).toBeCloseTo(1.23);
    expect(resp.trace.payload_filter_evals).toBe(0);
    expect(resp.trace.quantization_score_ms).toBeCloseTo(0.45);
    expect(resp.trace.total_ms).toBeCloseTo(2.10);
  });
});

// ---------------------------------------------------------------------------
// AdminClient — phase14
// ---------------------------------------------------------------------------

describe('AdminClient — slow queries (phase14)', () => {
  let mock: MockTransport;
  let client: AdminClient;

  beforeEach(() => {
    mock = createMock();
    client = new AdminClient({ transport: mock as never });
  });

  // ── listSlowQueries ───────────────────────────────────────────────────────

  describe('listSlowQueries', () => {
    const serverResponse = {
      entries: [
        { timestamp: '2026-05-02T00:01:00Z', collection: 'docs', k: 10, duration_ms: 312.5 },
        { timestamp: '2026-05-02T00:02:00Z', collection: 'logs', k: 20, duration_ms: 800.0 },
      ] satisfies SlowQueryEntry[],
      total: 2,
      config: { threshold_ms: 200, capacity: 1000 } satisfies SlowQueryConfig,
    };

    it('GETs /slow_queries', async () => {
      mock.get.mockResolvedValue(serverResponse);
      await client.listSlowQueries();
      expect(mock.get).toHaveBeenCalledWith('/slow_queries');
    });

    it('returns the entries array', async () => {
      mock.get.mockResolvedValue(serverResponse);
      const entries = await client.listSlowQueries();
      expect(entries).toHaveLength(2);
      expect(entries[0].collection).toBe('docs');
      expect(entries[0].duration_ms).toBe(312.5);
      expect(entries[1].k).toBe(20);
    });

    it('returns empty array when entries missing', async () => {
      mock.get.mockResolvedValue({ total: 0, config: { threshold_ms: 200, capacity: 1000 } });
      const entries = await client.listSlowQueries();
      expect(entries).toEqual([]);
    });
  });

  // ── setSlowQueryConfig ────────────────────────────────────────────────────

  describe('setSlowQueryConfig', () => {
    const configResponse = { threshold_ms: 150, capacity: 500, status: 'ok' };

    it('POSTs correct body to /slow_queries/config', async () => {
      mock.post.mockResolvedValue(configResponse);
      await client.setSlowQueryConfig({ threshold_ms: 150, capacity: 500 });
      expect(mock.post).toHaveBeenCalledWith(
        '/slow_queries/config',
        { threshold_ms: 150, capacity: 500 },
      );
    });

    it('returns SlowQueryConfig without the server-only status field', async () => {
      mock.post.mockResolvedValue(configResponse);
      const result = await client.setSlowQueryConfig({ threshold_ms: 150, capacity: 500 });
      expect(result.threshold_ms).toBe(150);
      expect(result.capacity).toBe(500);
      expect((result as Record<string, unknown>)['status']).toBeUndefined();
    });
  });
});
