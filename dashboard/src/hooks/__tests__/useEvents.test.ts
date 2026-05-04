/**
 * Unit tests for useEvents hook.
 */
import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useEvents } from '../useEvents';

const getMock = vi.fn();
vi.mock('../useApiClient', () => ({
  useApiClient: () => ({ get: getMock }),
}));

describe('useEvents', () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it('hydrates events from a populated response', async () => {
    getMock.mockResolvedValueOnce({
      events: [
        { ts: 'now', level: 'ok', msg: 'first' },
        { ts: 'just now', level: 'warn', msg: 'second' },
      ],
    });
    const { result } = renderHook(() => useEvents({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.events.length).toBe(2));
    expect(result.current.available).toBe(true);
  });

  it('accepts a bare array payload', async () => {
    getMock.mockResolvedValueOnce([{ ts: 'now', level: 'info', msg: 'hello' }]);
    const { result } = renderHook(() => useEvents({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.events.length).toBe(1));
    expect(result.current.events[0].msg).toBe('hello');
  });

  it('marks unavailable on 404', async () => {
    getMock.mockRejectedValueOnce(new Error('HTTP 404: Not Found'));
    const { result } = renderHook(() => useEvents({ intervalMs: 0 }));
    await waitFor(() => expect(result.current.available).toBe(false));
    expect(result.current.events.length).toBe(0);
  });
});
