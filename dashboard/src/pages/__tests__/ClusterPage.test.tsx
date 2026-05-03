import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import ClusterPage from '../ClusterPage';

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get:  vi.fn(async () => ({ data: { replicas: [] } })),
    post: vi.fn(async () => ({ data: {} })),
  }),
}));

describe('ClusterPage (Replication)', () => {
  it('renders the page heading and four KPIs', () => {
    render(<MemoryRouter><ClusterPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Replication/i })).toBeTruthy();
    expect(screen.getByText(/Master offset/i)).toBeTruthy();
    expect(screen.getByText(/Connected replicas/i)).toBeTruthy();
    expect(screen.getByText(/Max lag/i)).toBeTruthy();
    expect(screen.getByText(/Write concern/i)).toBeTruthy();
  });

  it('renders the replicas table column headers', () => {
    render(<MemoryRouter><ClusterPage /></MemoryRouter>);
    for (const col of ['ID', 'Region', 'Offset', 'Lag', 'Status']) {
      expect(screen.getAllByText(col).length).toBeGreaterThan(0);
    }
  });
});
