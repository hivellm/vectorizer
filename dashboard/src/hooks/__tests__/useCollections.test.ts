/**
 * Unit tests for useCollections hook
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useCollections } from '../useCollections';
import { useApiClient } from '../useApiClient';

// Mock useApiClient
vi.mock('../useApiClient');

describe('useCollections', () => {
  const mockApi = {
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
  };

  beforeEach(() => {
    vi.mocked(useApiClient).mockReturnValue(mockApi as any);
  });

  it('should list collections', async () => {
    const mockCollections = [
      { name: 'collection1', dimension: 512, metric: 'cosine' },
      { name: 'collection2', dimension: 256, metric: 'euclidean' },
    ];

    mockApi.get.mockResolvedValue(mockCollections);

    const { result } = renderHook(() => useCollections());

    const collections = await result.current.listCollections();

    expect(mockApi.get).toHaveBeenCalledWith('/collections');
    expect(collections).toEqual(mockCollections);
  });

  it('should get collection by name', async () => {
    const mockCollection = {
      name: 'test-collection',
      dimension: 512,
      metric: 'cosine',
    };

    mockApi.get.mockResolvedValue(mockCollection);

    const { result } = renderHook(() => useCollections());

    const collection = await result.current.getCollection('test-collection');

    expect(mockApi.get).toHaveBeenCalledWith('/collections/test-collection');
    expect(collection).toEqual(mockCollection);
  });

  it('should create collection', async () => {
    const mockRequest = {
      name: 'new-collection',
      dimension: 512,
      metric: 'cosine' as const,
    };

    const mockResponse = {
      name: 'new-collection',
      dimension: 512,
      metric: 'cosine',
    };

    mockApi.post.mockResolvedValue(mockResponse);

    const { result } = renderHook(() => useCollections());

    const collection = await result.current.createCollection(mockRequest);

    expect(mockApi.post).toHaveBeenCalledWith('/collections', mockRequest);
    expect(collection).toEqual(mockResponse);
  });

  it('should delete collection', async () => {
    mockApi.delete.mockResolvedValue(undefined);

    const { result } = renderHook(() => useCollections());

    await result.current.deleteCollection('test-collection');

    expect(mockApi.delete).toHaveBeenCalledWith('/collections/test-collection');
  });
});

