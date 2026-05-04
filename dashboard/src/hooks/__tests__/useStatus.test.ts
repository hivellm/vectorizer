/**
 * Unit tests for useStatus hook.
 */
import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useStatus } from '../useStatus';

const getMock = vi.fn();
vi.mock('../useApiClient', () => ({
  useApiClient: () => ({ get: getMock }),
}));

describe('useStatus', () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it('normalizes the canonical /status response shape', async () => {
    getMock.mockResolvedValueOnce({
      online: true,
      version: '3.3.0',
      uptime_seconds: 7200,
      collections_count: 5,
    });
    const { result } = renderHook(() => useStatus({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.status.version).toBe('3.3.0'));
    expect(result.current.status.online).toBe(true);
    expect(result.current.status.uptimeSeconds).toBe(7200);
    expect(result.current.status.collectionsCount).toBe(5);
  });

  it('returns empty version string when payload is missing version', async () => {
    getMock.mockResolvedValueOnce({ online: true, uptime_seconds: 0, collections_count: 0 });
    const { result } = renderHook(() => useStatus({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.status.version).toBe('');
  });

  it('returns zero state when payload is empty', async () => {
    getMock.mockResolvedValueOnce({});
    const { result } = renderHook(() => useStatus({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.status.online).toBe(false);
    expect(result.current.status.version).toBe('');
    expect(result.current.status.uptimeSeconds).toBe(0);
  });

  it('surfaces fetch errors', async () => {
    getMock.mockRejectedValueOnce(new Error('network failure'));
    const { result } = renderHook(() => useStatus({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.error).toBe('network failure'));
    expect(result.current.status.version).toBe('');
  });
});
