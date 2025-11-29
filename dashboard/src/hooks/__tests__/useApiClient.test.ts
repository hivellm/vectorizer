/**
 * Unit tests for useApiClient hook
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useApiClient } from '../useApiClient';

// Mock the ApiClient class
vi.mock('@/lib/api-client', () => {
  class MockApiClient {
    baseUrl: string;
    get = vi.fn();
    post = vi.fn();
    put = vi.fn();
    delete = vi.fn();

    constructor(baseUrl: string) {
      this.baseUrl = baseUrl;
    }
  }

  return {
    ApiClient: MockApiClient,
  };
});

describe('useApiClient', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create ApiClient instance', () => {
    const { result } = renderHook(() => useApiClient());

    expect(result.current).toBeDefined();
    expect(result.current).toHaveProperty('baseUrl');
    expect(result.current).toHaveProperty('get');
    expect(result.current).toHaveProperty('post');
    expect(result.current).toHaveProperty('put');
    expect(result.current).toHaveProperty('delete');
  });

  it('should return the same instance on re-render (memoization)', () => {
    const { result, rerender } = renderHook(() => useApiClient());
    const firstInstance = result.current;

    rerender();
    const secondInstance = result.current;

    expect(firstInstance).toBe(secondInstance);
  });

  it('should create ApiClient with baseUrl property', () => {
    const { result } = renderHook(() => useApiClient());
    
    // baseUrl should be a string (either empty or the dev URL)
    expect(typeof result.current.baseUrl).toBe('string');
  });

  it('should have all required API methods', () => {
    const { result } = renderHook(() => useApiClient());
    
    expect(typeof result.current.get).toBe('function');
    expect(typeof result.current.post).toBe('function');
    expect(typeof result.current.put).toBe('function');
    expect(typeof result.current.delete).toBe('function');
  });
});

