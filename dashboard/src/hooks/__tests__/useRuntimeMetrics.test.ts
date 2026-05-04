/**
 * Unit tests for useRuntimeMetrics hook.
 *
 * Covers:
 *  - snake_case → camelCase normalisation of the /metrics/runtime response
 *  - loading state on first call
 *  - error state on rejected promise
 *  - qpsHistory ring-buffer accumulation
 */
import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useRuntimeMetrics } from '../useRuntimeMetrics';

const getMock = vi.fn();
vi.mock('../useApiClient', () => ({
  useApiClient: () => ({ get: getMock }),
}));

/** Canonical snake_case shape returned by the Rust handler. */
const BACKEND_PAYLOAD = {
  cpu_percent: 42.5,
  memory_rss_bytes: 512_000_000,
  memory_total_bytes: 8_000_000_000,
  memory_percent: 6.4,
  active_connections: 17,
  uptime_seconds: 3600,
  qps_window_60s: 120.5,
  error_rate_5xx_60s: 0.002,
  throughput_by_route: [
    { route: '/api/search', qps: 80.0, p50_ms: 1.2, p99_ms: 4.8 },
    { route: '/api/collections', qps: 40.5, p50_ms: 0.8, p99_ms: 2.1 },
  ],
};

describe('useRuntimeMetrics', () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it('normalises snake_case keys from the backend into camelCase fields', async () => {
    getMock.mockResolvedValueOnce(BACKEND_PAYLOAD);

    const { result } = renderHook(() => useRuntimeMetrics());

    await waitFor(() => expect(result.current.loading).toBe(false));

    const m = result.current.metrics;
    expect(m.cpuPercent).toBeCloseTo(42.5);
    expect(m.memoryRssBytes).toBe(512_000_000);
    expect(m.memoryTotalBytes).toBe(8_000_000_000);
    expect(m.memoryPercent).toBeCloseTo(6.4);
    expect(m.activeConnections).toBe(17);
    expect(m.uptimeSeconds).toBe(3600);
    expect(m.qpsWindow60s).toBeCloseTo(120.5);
    expect(m.errorRate5xx60s).toBeCloseTo(0.002);
    expect(m.throughputByRoute).toHaveLength(2);
    expect(m.throughputByRoute[0].route).toBe('/api/search');
    expect(m.throughputByRoute[0].qps).toBeCloseTo(80.0);
    expect(m.throughputByRoute[0].p50Ms).toBeCloseTo(1.2);
    expect(m.throughputByRoute[0].p99Ms).toBeCloseTo(4.8);
    expect(m.throughputByRoute[1].route).toBe('/api/collections');
  });

  it('is in loading state before the first response resolves', async () => {
    // Return a promise that never resolves so we can check the loading=true state.
    getMock.mockReturnValueOnce(new Promise(() => {}));

    const { result } = renderHook(() => useRuntimeMetrics());

    expect(result.current.loading).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it('sets error and clears metrics when the fetch rejects', async () => {
    getMock.mockRejectedValueOnce(new Error('network failure'));

    const { result } = renderHook(() => useRuntimeMetrics());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.error).toBeInstanceOf(Error);
    expect(result.current.error?.message).toBe('network failure');
    // Metrics should remain at zero defaults
    expect(result.current.metrics.cpuPercent).toBe(0);
    expect(result.current.metrics.qpsWindow60s).toBe(0);
  });

  it('seeds qpsHistory from the REST fallback before any WS frame arrives', async () => {
    // Phase29 — without a WsDashboardProvider mounted, the hook has
    // no WS source. The REST one-shot still feeds qpsHistory[0] so
    // the MonitoringPage Sparkline has something to render before
    // the socket negotiates. Live updates beyond this seed flow via
    // WS pushes (covered by the provider tests, not here).
    getMock.mockResolvedValueOnce({ ...BACKEND_PAYLOAD, qps_window_60s: 10 });

    const { result } = renderHook(() => useRuntimeMetrics());

    await waitFor(() => expect(result.current.qpsHistory).toHaveLength(1));
    expect(result.current.qpsHistory[0]).toBe(10);
  });

  it('accepts a nested { data: ... } envelope from the ApiClient', async () => {
    getMock.mockResolvedValueOnce({ data: BACKEND_PAYLOAD });

    const { result } = renderHook(() => useRuntimeMetrics());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.metrics.cpuPercent).toBeCloseTo(42.5);
    expect(result.current.error).toBeNull();
  });

  it('returns zero-valued metrics when payload is not an object', async () => {
    getMock.mockResolvedValueOnce(null);

    const { result } = renderHook(() => useRuntimeMetrics());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.metrics.cpuPercent).toBe(0);
    expect(result.current.metrics.throughputByRoute).toHaveLength(0);
  });
});
