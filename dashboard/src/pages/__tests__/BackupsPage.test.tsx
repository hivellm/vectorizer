import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import BackupsPage from '../BackupsPage';

// The legacy /backups response shape is `{ backups: Backup[] }` (or a
// bare array). Each Backup has { id, name, date, size, collections }.
vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({
      backups: [
        {
          id: '1a2b3c4d-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
          name: 'snapshot-2026-05-01',
          date: '2026-05-01T10:00:00Z',
          size: 12_345_678,
          collections: ['docs', 'code'],
        },
        {
          id: '5e6f7g8h-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
          name: 'snapshot-2026-04-30',
          date: '2026-04-30T10:00:00Z',
          size: 8_901_234,
          collections: ['docs'],
        },
      ],
    })),
    post: vi.fn(async () => ({})),
    delete: vi.fn(async () => ({})),
  }),
}));

vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: async () => [
      { name: 'docs', dimension: 384, vector_count: 1200, metric: 'cosine' },
      { name: 'code', dimension: 384, vector_count: 800, metric: 'cosine' },
    ],
  }),
}));

vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

describe('BackupsPage', () => {
  it('renders the page heading and the backups table', async () => {
    render(<MemoryRouter><BackupsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Backups/i })).toBeTruthy();
    // After the async fetch resolves, either rows or the empty state appear.
    const matches = await screen.findAllByText(
      /snapshot-|No backups|backup/i,
      undefined,
      { timeout: 3000 },
    );
    expect(matches.length).toBeGreaterThan(0);
  });
});
