import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import McpToolsPage from '../McpToolsPage';

vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({ data: { tools: [] } })),
    post: vi.fn(async () => ({ data: {} })),
  }),
}));

describe('McpToolsPage', () => {
  it('renders the page heading and the three KPI cards', () => {
    render(<MemoryRouter><McpToolsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /MCP Tools/i })).toBeTruthy();
    expect(screen.getByText(/Active connections/i)).toBeTruthy();
    expect(screen.getByText(/Tool calls today/i)).toBeTruthy();
    expect(screen.getByText(/Errors/i)).toBeTruthy();
  });

  it('renders the tools table column headers', () => {
    render(<MemoryRouter><McpToolsPage /></MemoryRouter>);
    for (const col of ['Tool', 'Calls', 'p99', 'Status']) {
      expect(screen.getByText(col)).toBeTruthy();
    }
  });
});
