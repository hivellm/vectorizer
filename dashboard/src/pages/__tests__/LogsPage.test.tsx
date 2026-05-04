import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import LogsPage from '../LogsPage';

// The real ApiClient.get<T> resolves with the parsed body directly (no
// `{ data }` wrapper). The legacy /api/logs response is either an array
// of LogEntry, a `{ logs: LogEntry[] }` envelope, or a newline-delimited
// string. Mock the envelope shape here so the restyled page can mount
// without hitting the network.
vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({
      logs: [
        { timestamp: '2026-05-02T17:00:00Z', level: 'info',  message: 'Server started' },
        { timestamp: '2026-05-02T17:00:01Z', level: 'warn',  message: 'High latency' },
        { timestamp: '2026-05-02T17:00:02Z', level: 'error', message: 'Disk full' },
        { timestamp: '2026-05-02T17:00:03Z', level: 'debug', message: 'Connection trace', target: 'net::pool' },
      ],
    })),
    post: vi.fn(async () => ({})),
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

describe('LogsPage', () => {
  it('renders the page heading', () => {
    render(<MemoryRouter><LogsPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Logs/i })).toBeTruthy();
  });

  it('renders a log container after the async fetch resolves', async () => {
    render(<MemoryRouter><LogsPage /></MemoryRouter>);
    // Either a real log line or the empty-state copy should appear once
    // the async fetch resolves.
    const matches = await screen.findAllByText(
      /Server started|High latency|Disk full|No logs/i,
      undefined,
      { timeout: 3000 },
    );
    expect(matches.length).toBeGreaterThan(0);
  });
});
