import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import SearchPage from '../SearchPage';

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy' },
    ],
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [{ name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy' }],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    post: vi.fn(async () => ({ data: { results: [] } })),
    get:  vi.fn(async () => ({ data: [] })),
  }),
}));

describe('SearchPage', () => {
  it('renders the four search-type tabs', () => {
    render(<MemoryRouter><SearchPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Search Playground/i })).toBeTruthy();
    for (const label of ['Intelligent', 'Semantic', 'Contextual', 'Multi-collection']) {
      expect(screen.getByText(label)).toBeTruthy();
    }
  });

  it('switches the request preview when clicking a different tab', () => {
    render(<MemoryRouter><SearchPage /></MemoryRouter>);
    // Default = intelligent
    expect(screen.getAllByText(/intelligent_search/i).length).toBeGreaterThanOrEqual(1);
    fireEvent.click(screen.getByText('Semantic'));
    expect(screen.getAllByText(/semantic_search/i).length).toBeGreaterThanOrEqual(1);
  });
});
