import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import VectorsPage from '../VectorsPage';

// The page uses `useToastContext` (Copy/Delete feedback); stub the toast barrel
// so it renders without a ToastProvider.
vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

// The Insert modal (rendered but closed) pulls useAuth via the API client; it
// is not under test here, so stub it out.
vi.mock('@/components/modals/FileUploadModal', () => ({
  default: () => null,
}));

const SAMPLE = [
  { id: 'vec_aaa', text: 'first vector text payload', dimension: 4, vector: [0.5, -0.3, 0.1, -0.7] },
  { id: 'vec_bbb', text: 'second vector text payload', dimension: 4, vector: [0.2, 0.4, -0.1, 0.6] },
];

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 4, vector_count: 2, status: 'healthy' },
    ],
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [{ name: 'docs', dimension: 4, vector_count: 2, status: 'healthy' }],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
    setError: vi.fn(),
  }),
}));

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({ data: { vectors: SAMPLE } })),
    post: vi.fn(async () => ({ data: {} })),
  }),
}));

describe('VectorsPage', () => {
  it('renders the page heading and a collection dropdown', () => {
    render(<MemoryRouter><VectorsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Vector Browser|Vectors/i })).toBeTruthy();
    expect(screen.getByLabelText(/Collection/i)).toBeTruthy();
  });

  it('renders the embedding histogram svg with bars when a vector is selected', async () => {
    const { container } = render(<MemoryRouter><VectorsPage /></MemoryRouter>);
    // Wait for async fetch + first vector to appear. The id appears in both
    // the table row and the detail header (because the first vector is the
    // default selection), so use *AllBy*.
    const matches = await screen.findAllByText('vec_aaa');
    expect(matches.length).toBeGreaterThan(0);
    // Click the table-cell occurrence (the <td class="id">) to re-select.
    const cell = matches.find((el) => el.tagName === 'TD');
    fireEvent.click(cell ?? matches[0]);
    // The embedding viz contains <rect> bars
    const rects = container.querySelectorAll('svg rect');
    expect(rects.length).toBeGreaterThan(0);
  });
});
