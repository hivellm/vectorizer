/**
 * Unit tests for useSandboxHistory.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSandboxHistory, __SANDBOX_HISTORY_INTERNALS } from '../useSandboxHistory';

describe('useSandboxHistory', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('starts with empty history and favorites', () => {
    const { result } = renderHook(() => useSandboxHistory());
    expect(result.current.history).toEqual([]);
    expect(result.current.favorites).toEqual([]);
  });

  it('records a request with a stable id and ISO timestamp', () => {
    const { result } = renderHook(() => useSandboxHistory());

    act(() => {
      result.current.recordRequest({
        method: 'POST',
        path: '/search',
        pathParams: {},
        body: '{"q":"hello"}',
        status: 200,
        timingMs: 42,
      });
    });

    expect(result.current.history).toHaveLength(1);
    const [entry] = result.current.history;
    expect(entry.method).toBe('POST');
    expect(entry.path).toBe('/search');
    expect(entry.body).toBe('{"q":"hello"}');
    expect(entry.status).toBe(200);
    expect(entry.timingMs).toBe(42);
    expect(entry.id).toMatch(/^[a-z0-9]+-[a-z0-9]+$/);
    expect(new Date(entry.ranAt).toString()).not.toBe('Invalid Date');
  });

  it('prepends new entries and prunes to the HISTORY_LIMIT cap', () => {
    const { result } = renderHook(() => useSandboxHistory());

    act(() => {
      for (let i = 0; i < __SANDBOX_HISTORY_INTERNALS.HISTORY_LIMIT + 5; i++) {
        result.current.recordRequest({
          method: 'GET',
          path: `/health?n=${i}`,
          pathParams: {},
          body: '',
        });
      }
    });

    expect(result.current.history).toHaveLength(__SANDBOX_HISTORY_INTERNALS.HISTORY_LIMIT);
    // Most recent entry is at index 0 because recordRequest prepends.
    const mostRecent = result.current.history[0];
    expect(mostRecent.path).toBe(`/health?n=${__SANDBOX_HISTORY_INTERNALS.HISTORY_LIMIT + 4}`);
  });

  it('toggles favorites idempotently by fingerprint', () => {
    const { result } = renderHook(() => useSandboxHistory());

    const entry = {
      method: 'POST',
      path: '/search',
      pathParams: {},
      body: '{"q":"fingerprint"}',
    };

    act(() => result.current.toggleFavorite(entry));
    expect(result.current.favorites).toHaveLength(1);
    expect(result.current.isFavorited('POST', '/search', '{"q":"fingerprint"}')).toBe(true);

    // Fingerprint is method + path + trimmed body — different whitespace still matches.
    expect(result.current.isFavorited('POST', '/search', '  {"q":"fingerprint"}  ')).toBe(true);

    act(() => result.current.toggleFavorite(entry));
    expect(result.current.favorites).toHaveLength(0);
    expect(result.current.isFavorited('POST', '/search', '{"q":"fingerprint"}')).toBe(false);
  });

  it('clears history without touching favorites', () => {
    const { result } = renderHook(() => useSandboxHistory());

    act(() => {
      result.current.recordRequest({
        method: 'GET',
        path: '/a',
        pathParams: {},
        body: '',
      });
      result.current.toggleFavorite({
        method: 'GET',
        path: '/a',
        pathParams: {},
        body: '',
      });
    });

    expect(result.current.history).toHaveLength(1);
    expect(result.current.favorites).toHaveLength(1);

    act(() => result.current.clearHistory());

    expect(result.current.history).toHaveLength(0);
    expect(result.current.favorites).toHaveLength(1);
  });

  it('persists across remounts via localStorage', () => {
    const first = renderHook(() => useSandboxHistory());
    act(() => {
      first.result.current.recordRequest({
        method: 'DELETE',
        path: '/collections/x',
        pathParams: { name: 'x' },
        body: '',
        status: 204,
        timingMs: 7,
      });
    });

    const second = renderHook(() => useSandboxHistory());
    expect(second.result.current.history).toHaveLength(1);
    expect(second.result.current.history[0].path).toBe('/collections/x');
    expect(second.result.current.history[0].status).toBe(204);
  });

  it('removeHistory and removeFavorite drop one entry by id', () => {
    const { result } = renderHook(() => useSandboxHistory());

    act(() => {
      result.current.recordRequest({ method: 'GET', path: '/x', pathParams: {}, body: '' });
      result.current.recordRequest({ method: 'GET', path: '/y', pathParams: {}, body: '' });
      result.current.toggleFavorite({ method: 'GET', path: '/x', pathParams: {}, body: '' });
      result.current.toggleFavorite({ method: 'GET', path: '/y', pathParams: {}, body: '' });
    });

    expect(result.current.history).toHaveLength(2);
    expect(result.current.favorites).toHaveLength(2);

    const firstHistoryId = result.current.history[0].id;
    const firstFavoriteId = result.current.favorites[0].id;

    act(() => {
      result.current.removeHistory(firstHistoryId);
      result.current.removeFavorite(firstFavoriteId);
    });

    expect(result.current.history).toHaveLength(1);
    expect(result.current.favorites).toHaveLength(1);
    expect(result.current.history.find((h) => h.id === firstHistoryId)).toBeUndefined();
    expect(result.current.favorites.find((f) => f.id === firstFavoriteId)).toBeUndefined();
  });
});
