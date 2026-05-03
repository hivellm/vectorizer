import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import OverviewPage from '../OverviewPage';

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy', metric: 'cosine' },
      { name: 'code', dimension: 768, vector_count: 8000, status: 'indexing', metric: 'cosine' },
    ],
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy' },
      { name: 'code', dimension: 768, vector_count: 8000, status: 'indexing' },
    ],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

describe('OverviewPage', () => {
  it('renders KPI strip and top collections table', () => {
    render(<MemoryRouter><OverviewPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Overview/i })).toBeTruthy();
    expect(screen.getByText(/Total vectors/i)).toBeTruthy();
    expect(screen.getByText('docs')).toBeTruthy();
    expect(screen.getByText('code')).toBeTruthy();
  });
});
