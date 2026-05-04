import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import ConnectionsPage from '../ConnectionsPage';

// `useConnections` is a localStorage-backed hook that manages a list of
// user-saved Vectorizer server connections (profiles). The real shape:
//   { id, name, host, port, type, auth?: { token? }, status, active? }
vi.mock('@/hooks/useConnections', () => ({
  useConnections: () => ({
    connections: [
      {
        id: 'conn-local',
        name: 'Local server',
        host: 'localhost',
        port: 15002,
        type: 'local' as const,
        status: 'online' as const,
      },
      {
        id: 'conn-remote',
        name: 'Production cluster',
        host: '10.0.0.1',
        port: 15002,
        type: 'remote' as const,
        auth: { token: 'secret' },
        status: 'offline' as const,
      },
    ],
    activeConnectionId: 'conn-local',
    activeConnection: null,
    loading: false,
    addConnection: vi.fn(() => 'conn-new'),
    updateConnection: vi.fn(),
    removeConnection: vi.fn(),
    checkConnectionHealth: vi.fn(async () => 'online' as const),
    checkAllConnectionsHealth: vi.fn(async () => {}),
    setActiveConnection: vi.fn(),
  }),
}));

// The page uses `useToastContext` from the ToastProvider barrel.
vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

describe('ConnectionsPage', () => {
  it('renders the page heading and the connections table', () => {
    render(<MemoryRouter><ConnectionsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Connections/i })).toBeTruthy();
    // Saved connections appear as rows in the table. (Local server also
    // shows up in the "Active" KPI tile, so allow >=1 match.)
    expect(screen.getAllByText('Local server').length).toBeGreaterThan(0);
    expect(screen.getByText('Production cluster')).toBeTruthy();
    // The restyle renders connections in a real <Tbl> with column headers.
    for (const col of ['Name', 'Endpoint', 'Status']) {
      expect(screen.getAllByText(col).length).toBeGreaterThan(0);
    }
  });
});
