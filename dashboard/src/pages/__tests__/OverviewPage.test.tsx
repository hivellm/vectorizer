import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import OverviewPage from '../OverviewPage';

const { navigateMock } = vi.hoisted(() => ({ navigateMock: vi.fn() }));
vi.mock('react-router-dom', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router-dom')>();
  return { ...actual, useNavigate: () => navigateMock };
});

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 384, vector_count: 1200, status: 'healthy', metric: 'cosine' },
      { name: 'code', dimension: 768, vector_count: 8000, status: 'indexing', metric: 'cosine' },
    ],
  }),
}));

vi.mock('@/hooks/useMetrics', () => ({
  useMetrics: () => ({
    metrics: {
      qps: 0,
      p99Ms: 0,
      cpuPercent: 0,
      memPercent: 0,
      connections: 0,
      cacheHitRate: 0,
      totalVectors: 0,
    },
    loading: false,
    error: null,
  }),
}));

vi.mock('@/hooks/useStats', () => ({
  useStats: () => ({
    stats: {
      status: 'healthy',
      cache: { size: 0, capacity: 0, hits: 0, misses: 0, evictions: 0, hitRate: 0.942 },
    },
    loading: false,
    error: null,
  }),
}));

vi.mock('@/hooks/useEvents', () => ({
  useEvents: () => ({ events: [], loading: false, error: null, available: false }),
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

  it('navigates to a collection when its Top Collections row is clicked', () => {
    navigateMock.mockClear();
    render(<MemoryRouter><OverviewPage /></MemoryRouter>);
    fireEvent.click(screen.getByText('docs'));
    expect(navigateMock).toHaveBeenCalledWith('/collections?name=docs');
  });
});
