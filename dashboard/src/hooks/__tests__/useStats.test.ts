/**
 * Unit tests for useStats hook.
 */
import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useStats } from '../useStats';

const getMock = vi.fn();
vi.mock('../useApiClient', () => ({
  useApiClient: () => ({ get: getMock }),
}));

describe('useStats', () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it('normalizes /health cache shape', async () => {
    getMock.mockResolvedValueOnce({
      status: 'healthy',
      cache: {
        size: 7000,
        capacity: 10000,
        hits: 4_210_000,
        misses: 258_000,
        evictions: 1204,
        hit_rate: 0.942,
      },
    });
    const { result } = renderHook(() => useStats({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.stats.cache.hits).toBe(4_210_000));
    expect(result.current.stats.cache.hitRate).toBeCloseTo(0.942);
    expect(result.current.stats.status).toBe('healthy');
  });

  it('returns zero cache when payload is empty', async () => {
    getMock.mockResolvedValueOnce({});
    const { result } = renderHook(() => useStats({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.stats.cache.size).toBe(0);
  });

  it('picks up optional WAL fields when the backend grows them', async () => {
    getMock.mockResolvedValueOnce({
      status: 'healthy',
      cache: { size: 1, capacity: 1, hits: 1, misses: 0, evictions: 0, hit_rate: 1 },
      wal: { sequence: 8_811_998, size_bytes: 297_795_584, last_checkpoint: '2026-05-02T12:00:00Z' },
    });
    const { result } = renderHook(() => useStats({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.stats.walSequence).toBe(8_811_998));
    expect(result.current.stats.walSizeBytes).toBe(297_795_584);
    expect(result.current.stats.walLastCheckpointAt).toBe('2026-05-02T12:00:00Z');
  });

  it('surfaces fetch errors', async () => {
    getMock.mockRejectedValueOnce(new Error('boom'));
    const { result } = renderHook(() => useStats({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.error).toBe('boom'));
  });

  it('extracts version from /health when present', async () => {
    getMock.mockResolvedValueOnce({
      status: 'healthy',
      version: '3.2.1',
      cache: { size: 0, capacity: 0, hits: 0, misses: 0, evictions: 0, hit_rate: 0 },
    });
    const { result } = renderHook(() => useStats({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.stats.version).toBe('3.2.1'));
  });

  it('leaves version undefined when /health does not emit it', async () => {
    getMock.mockResolvedValueOnce({
      status: 'healthy',
      cache: { size: 0, capacity: 0, hits: 0, misses: 0, evictions: 0, hit_rate: 0 },
    });
    const { result } = renderHook(() => useStats({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.stats.version).toBeUndefined();
  });
});
