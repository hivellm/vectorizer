import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import ClusterPage from '../ClusterPage';

const getMock = vi.fn();

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: getMock,
    post: vi.fn(async () => ({ data: {} })),
  }),
}));

describe('ClusterPage (Replication)', () => {
  it('renders the page heading and four KPIs', () => {
    getMock.mockResolvedValueOnce({ replicas: [] });
    render(<MemoryRouter><ClusterPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Replication/i })).toBeTruthy();
    expect(screen.getByText(/Master offset/i)).toBeTruthy();
    expect(screen.getByText(/Connected replicas/i)).toBeTruthy();
    expect(screen.getByText(/Max lag/i)).toBeTruthy();
    expect(screen.getByText(/Write concern/i)).toBeTruthy();
  });

  it('shows the empty state when no replicas are returned', async () => {
    getMock.mockResolvedValueOnce({ replicas: [] });
    render(<MemoryRouter><ClusterPage /></MemoryRouter>);
    await waitFor(() =>
      expect(screen.getByText(/No replicas to display/i)).toBeTruthy(),
    );
  });

  it('renders the replicas table column headers when replicas are returned', async () => {
    getMock.mockResolvedValueOnce({
      replicas: [
        { id: 'replica-eu', region: 'eu-west-1', offset: 1, lag: 0, status: 'in-sync' },
      ],
    });
    render(<MemoryRouter><ClusterPage /></MemoryRouter>);
    await waitFor(() => expect(screen.getByText('replica-eu')).toBeTruthy());
    for (const col of ['ID', 'Region', 'Offset', 'Lag', 'Status']) {
      expect(screen.getAllByText(col).length).toBeGreaterThan(0);
    }
  });

  it('renders the endpoint-not-exposed pill on fetch failure', async () => {
    getMock.mockRejectedValueOnce(new Error('boom'));
    render(<MemoryRouter><ClusterPage /></MemoryRouter>);
    await waitFor(() =>
      expect(
        screen.getByText(/Replication endpoint not yet exposed/i),
      ).toBeTruthy(),
    );
  });
});
