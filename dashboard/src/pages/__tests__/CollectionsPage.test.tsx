/**
 * Unit tests for CollectionsPage component
 */

import { describe, it, expect, vi } from 'vitest';
import { renderWithProviders, screen } from '@/test-utils/render';
import CollectionsPage from '../CollectionsPage';

// Mock hooks
vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: vi.fn().mockResolvedValue([]),
    createCollection: vi.fn(),
    deleteCollection: vi.fn(),
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

describe('CollectionsPage', () => {
  it('should render collections page', () => {
    renderWithProviders(<CollectionsPage />, { route: '/collections' });

    // Use getAllByText since "collections" appears multiple times
    const collectionsElements = screen.getAllByText(/collections/i);
    expect(collectionsElements.length).toBeGreaterThan(0);
  });

  it('should render create collection button', () => {
    renderWithProviders(<CollectionsPage />, { route: '/collections' });

    // Use getAllByText since "create" might appear multiple times
    const createElements = screen.getAllByText(/create|new/i);
    expect(createElements.length).toBeGreaterThan(0);
  });
});
