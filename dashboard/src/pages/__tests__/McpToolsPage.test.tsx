import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import McpToolsPage from '../McpToolsPage';

const getMock = vi.fn();

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: getMock,
    post: vi.fn(async () => ({ data: {} })),
  }),
}));

describe('McpToolsPage', () => {
  it('renders the page heading and the two KPIs', () => {
    getMock.mockResolvedValueOnce({ tools: [] });
    render(<MemoryRouter><McpToolsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /MCP Tools/i })).toBeTruthy();
    expect(screen.getByText(/Tool calls observed/i)).toBeTruthy();
    expect(screen.getByText(/Errors/i)).toBeTruthy();
  });

  it('shows an empty-state message when the endpoint returns no tools', async () => {
    getMock.mockResolvedValueOnce({ tools: [] });
    render(<MemoryRouter><McpToolsPage /></MemoryRouter>);
    await waitFor(() =>
      expect(screen.getByText(/No MCP tool stats to display/i)).toBeTruthy(),
    );
  });

  it('renders the table headers when live tools are returned', async () => {
    getMock.mockResolvedValueOnce({
      tools: [{ name: 'search_vectors', calls: 100, p99: 2.8, status: 'ok' }],
    });
    render(<MemoryRouter><McpToolsPage /></MemoryRouter>);
    await waitFor(() => expect(screen.getByText('search_vectors')).toBeTruthy());
    for (const col of ['Tool', 'Calls', 'p99', 'Status']) {
      expect(screen.getByText(col)).toBeTruthy();
    }
  });

  it('shows the endpoint-not-exposed pill on fetch failure', async () => {
    getMock.mockRejectedValueOnce(new Error('boom'));
    render(<MemoryRouter><McpToolsPage /></MemoryRouter>);
    await waitFor(() =>
      expect(
        screen.getByText(/MCP capability stats endpoint not yet exposed/i),
      ).toBeTruthy(),
    );
  });
});
