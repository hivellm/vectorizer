/**
 * Phase25 §7 — RuntimeMetrics + extended Stats / Collection wire shapes.
 *
 * Verifies that AdminClient.getRuntimeMetrics() targets
 * `GET /metrics/runtime`, that the new Stats fields default cleanly on
 * older servers, and that Collection.vector_count_history rounds-trips.
 * Pattern mirrors tests/cluster-auth-admin.test.ts.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AdminClient } from '../src/client/admin';
import type {
  RuntimeMetrics,
  Stats,
} from '../src/models/admin';
import type { Collection, VectorCountSample } from '../src/models/collection';

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

describe('AdminClient.getRuntimeMetrics (phase25 §7.2)', () => {
  let mock: MockTransport;
  let client: AdminClient;

  beforeEach(() => {
    mock = createMock();
    client = new AdminClient({ transport: mock as never });
  });

  it('GETs /metrics/runtime', async () => {
    mock.get.mockResolvedValue({} as RuntimeMetrics);
    await client.getRuntimeMetrics();
    expect(mock.get).toHaveBeenCalledWith('/metrics/runtime');
  });

  it('decodes a full snapshot including throughput_by_route and wal', async () => {
    const wire: RuntimeMetrics = {
      cpu_percent: 12.4,
      memory_rss_bytes: 124_857_600,
      memory_total_bytes: 17_179_869_184,
      memory_percent: 0.73,
      active_connections: 8,
      uptime_seconds: 3712,
      qps_window_60s: 142.3,
      error_rate_5xx_60s: 0.001,
      throughput_by_route: [
        { route: '/insert_texts', qps: 12.0, p50_ms: 8.2, p99_ms: 41.0 },
      ],
      wal: {
        current_seq: 482919,
        size_bytes: 12_582_912,
        last_checkpoint_at: 1_714_828_800,
        last_checkpoint_seq: 482_800,
      },
    };
    mock.get.mockResolvedValue(wire);
    const m = await client.getRuntimeMetrics();
    expect(m.cpu_percent).toBe(12.4);
    expect(m.active_connections).toBe(8);
    expect(m.throughput_by_route?.[0]?.route).toBe('/insert_texts');
    expect(m.throughput_by_route?.[0]?.p99_ms).toBe(41.0);
    expect(m.wal?.current_seq).toBe(482919);
    expect(m.wal?.last_checkpoint_seq).toBe(482_800);
  });

  it('tolerates a partial payload from a server that omits routes/wal', async () => {
    const wire: RuntimeMetrics = {
      cpu_percent: 1.0,
      memory_total_bytes: 8_000_000_000,
    };
    mock.get.mockResolvedValue(wire);
    const m = await client.getRuntimeMetrics();
    expect(m.cpu_percent).toBe(1.0);
    expect(m.throughput_by_route).toBeUndefined();
    expect(m.wal).toBeUndefined();
  });
});

describe('Stats — phase25 §5 quantization fields (TS shape)', () => {
  it('decodes default_quantization + compression_ratio when present', () => {
    const wire: Stats = {
      collections: 3,
      total_vectors: 12_000,
      uptime_seconds: 60,
      version: '3.4.0',
      default_quantization: 'sq-8bit',
      compression_ratio: 4.0,
    };
    expect(wire.default_quantization).toBe('sq-8bit');
    expect(wire.compression_ratio).toBe(4.0);
  });

  it('treats the new fields as optional for older servers', () => {
    const wire: Stats = {
      collections: 0,
      total_vectors: 0,
      uptime_seconds: 0,
      version: '3.3.0',
    };
    expect(wire.default_quantization).toBeUndefined();
    expect(wire.compression_ratio).toBeUndefined();
  });
});

describe('Collection.vector_count_history (phase25 §6)', () => {
  it('round-trips a typed sample list', () => {
    const samples: VectorCountSample[] = [
      { at: 1_714_828_740, count: 482_900 },
      { at: 1_714_828_800, count: 482_919 },
    ];
    const c: Collection = {
      name: 'docs',
      dimension: 768,
      vector_count: 482_919,
      vector_count_history: samples,
    };
    expect(c.vector_count_history).toHaveLength(2);
    expect(c.vector_count_history?.[0]?.count).toBe(482_900);
    expect(c.vector_count_history?.[1]?.at).toBe(1_714_828_800);
  });

  it('is optional so older payloads decode unchanged', () => {
    const c: Collection = { name: 'older', dimension: 384 };
    expect(c.vector_count_history).toBeUndefined();
  });
});
