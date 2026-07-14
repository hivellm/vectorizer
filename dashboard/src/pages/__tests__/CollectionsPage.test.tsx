import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import CollectionsPage from '../CollectionsPage';

// The page uses `useToastContext` (Reindex/Copy feedback) and the create/delete
// modals; stub the toast barrel so it renders without a ToastProvider.
vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

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
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy', metric: 'cosine' },
      { name: 'code', dimension: 768, vector_count: 8000, status: 'indexing', metric: 'cosine' },
    ],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

describe('CollectionsPage', () => {
  it('renders the list and selects the first item by default', () => {
    render(<MemoryRouter><CollectionsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Collections/i })).toBeTruthy();
    // List entry
    expect(screen.getAllByText('docs').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('code')).toBeTruthy();
    // Detail pane shows the first collection (the heading slot in the detail card)
    expect(screen.getAllByText('docs').length).toBeGreaterThanOrEqual(2);
  });

  it('switches detail when clicking a different row', () => {
    render(<MemoryRouter><CollectionsPage /></MemoryRouter>);
    fireEvent.click(screen.getByText('code'));
    expect(screen.getAllByText('code').length).toBeGreaterThanOrEqual(2);
  });
});
