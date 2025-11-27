/**
 * Unit tests for useSearchHistory hook
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSearchHistory } from '../useSearchHistory';

describe('useSearchHistory', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
  });

  it('should initialize with empty history', () => {
    const { result } = renderHook(() => useSearchHistory());
    expect(result.current.history).toEqual([]);
  });

  it('should add search to history', () => {
    const { result } = renderHook(() => useSearchHistory());

    act(() => {
      result.current.addToHistory({
        query: 'test query',
        collection: 'collection1',
        type: 'text',
        limit: 10,
      });
    });

    expect(result.current.history.length).toBe(1);
    expect(result.current.history[0].query).toBe('test query');
    expect(result.current.history[0].collection).toBe('collection1');
  });

  it('should limit history size', () => {
    const { result } = renderHook(() => useSearchHistory());

    act(() => {
      // Add more than max items (MAX_HISTORY_ITEMS is 20)
      for (let i = 0; i < 25; i++) {
        result.current.addToHistory({
          query: `query ${i}`,
          collection: 'collection1',
          type: 'text',
          limit: 10,
        });
      }
    });

    // History should be limited to MAX_HISTORY_ITEMS (20)
    expect(result.current.history.length).toBeLessThanOrEqual(20);
  });

  it('should clear history', () => {
    const { result } = renderHook(() => useSearchHistory());

    act(() => {
      result.current.addToHistory({
        query: 'test query',
        collection: 'collection1',
        type: 'text',
        limit: 10,
      });
    });

    expect(result.current.history.length).toBe(1);

    act(() => {
      result.current.clearHistory();
    });

    expect(result.current.history.length).toBe(0);
  });

  it('should remove search from history', () => {
    const { result } = renderHook(() => useSearchHistory());

    act(() => {
      result.current.addToHistory({
        query: 'query1',
        collection: 'collection1',
        type: 'text',
        limit: 10,
      });
      result.current.addToHistory({
        query: 'query2',
        collection: 'collection2',
        type: 'text',
        limit: 10,
      });
    });

    expect(result.current.history.length).toBe(2);
    // New items are added at the beginning, so query2 is first
    expect(result.current.history[0].query).toBe('query2');
    expect(result.current.history[1].query).toBe('query1');

    // Remove the first item (query2)
    const firstId = result.current.history[0].id;

    act(() => {
      result.current.removeFromHistory(firstId);
    });

    expect(result.current.history.length).toBe(1);
    // After removing query2, only query1 remains
    expect(result.current.history[0].query).toBe('query1');
  });
});

