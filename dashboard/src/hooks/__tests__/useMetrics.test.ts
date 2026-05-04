/**
 * Unit tests for useMetrics hook.
 */

import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useMetrics } from '../useMetrics';

const getMock = vi.fn();

vi.mock('../useApiClient', () => ({
  useApiClient: () => ({ get: getMock }),
}));

describe('useMetrics', () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it('returns zeros initially when payload is empty', async () => {
    getMock.mockResolvedValueOnce({});
    const { result } = renderHook(() => useMetrics({ intervalMs: 0 }));
    expect(result.current.metrics.qps).toBe(0);
    expect(result.current.metrics.cpuPercent).toBe(0);
    expect(result.current.metrics.totalVectors).toBe(0);
  });

  it('normalizes the backend payload (canonical snake_case fields)', async () => {
    getMock.mockResolvedValueOnce({
      qps: 1234,
      p99: 2.8,
      cpu_percent: 42,
      memory_percent: 61.5,
      connections: 17,
      cache_hit_rate: 0.93,
      total_vectors: 587_963,
    });
    const { result } = renderHook(() => useMetrics({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.metrics.qps).toBe(1234));
    expect(result.current.metrics.p99Ms).toBe(2.8);
    expect(result.current.metrics.cpuPercent).toBe(42);
    expect(result.current.metrics.memPercent).toBe(61.5);
    expect(result.current.metrics.connections).toBe(17);
    expect(result.current.metrics.cacheHitRate).toBeCloseTo(0.93);
    expect(result.current.metrics.totalVectors).toBe(587_963);
  });

  it('reads cache.hit_rate from the /health-shaped nested cache object', async () => {
    getMock.mockResolvedValueOnce({
      total_vectors: 12,
      cache: { hits: 100, misses: 5, hit_rate: 0.952 },
    });
    const { result } = renderHook(() => useMetrics({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.metrics.totalVectors).toBe(12));
    expect(result.current.metrics.cacheHitRate).toBeCloseTo(0.952);
  });

  it('surfaces fetch errors', async () => {
    getMock.mockRejectedValueOnce(new Error('boom'));
    const { result } = renderHook(() => useMetrics({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.error).toBe('boom'));
  });
});
