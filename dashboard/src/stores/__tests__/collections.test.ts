/**
 * Unit tests for collections store (Zustand)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { useCollectionsStore } from '../collections';
import type { Collection } from '@/hooks/useCollections';

describe('useCollectionsStore', () => {
  beforeEach(() => {
    // Reset store before each test
    useCollectionsStore.setState({
      collections: [],
      loading: false,
      error: null,
    });
  });

  it('should have initial state', () => {
    const state = useCollectionsStore.getState();
    expect(state.collections).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBe(null);
  });

  it('should set collections', () => {
    const collections: Collection[] = [
      {
        name: 'test-collection',
        dimension: 512,
        metric: 'cosine',
        vector_count: 100,
      },
    ];

    useCollectionsStore.getState().setCollections(collections);
    const state = useCollectionsStore.getState();

    expect(state.collections).toEqual(collections);
    expect(state.collections.length).toBe(1);
  });

  it('should set loading state', () => {
    useCollectionsStore.getState().setLoading(true);
    expect(useCollectionsStore.getState().loading).toBe(true);

    useCollectionsStore.getState().setLoading(false);
    expect(useCollectionsStore.getState().loading).toBe(false);
  });

  it('should set error state', () => {
    const error = 'Test error';
    useCollectionsStore.getState().setError(error);
    expect(useCollectionsStore.getState().error).toBe(error);

    useCollectionsStore.getState().setError(null);
    expect(useCollectionsStore.getState().error).toBe(null);
  });

  it('should add collection', () => {
    const collection: Collection = {
      name: 'test-collection',
      dimension: 512,
      metric: 'cosine',
      vector_count: 100,
    };

    useCollectionsStore.getState().addCollection(collection);
    const state = useCollectionsStore.getState();

    expect(state.collections.length).toBe(1);
    expect(state.collections[0].name).toBe('test-collection');
  });

  it('should clear error', () => {
    useCollectionsStore.getState().setError('Test error');
    expect(useCollectionsStore.getState().error).toBe('Test error');

    useCollectionsStore.getState().clearError();
    expect(useCollectionsStore.getState().error).toBe(null);
  });

  it('should remove collection', () => {
    const collections: Collection[] = [
      {
        name: 'test-collection',
        dimension: 512,
        metric: 'cosine',
      },
    ];

    useCollectionsStore.getState().setCollections(collections);
    useCollectionsStore.getState().removeCollection('test-collection');

    const state = useCollectionsStore.getState();
    expect(state.collections.length).toBe(0);
  });
});

