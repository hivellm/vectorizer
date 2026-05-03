import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import ConfigurationPage from '../ConfigurationPage';

// Mock the API client. The Settings page hits the legacy /config endpoint;
// the rewrite preserves that endpoint exactly. Returning a parsed object lets
// the General/Defaults KeyValue cards derive labels even before the dynamic
// js-yaml import resolves in jsdom/happy-dom.
vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({
      server: { host: '127.0.0.1', port: 15002, data_dir: '/var/lib/vectorizer' },
      collections: {
        defaults: {
          metric: 'cosine',
          embedding: { model: 'bm25' },
          index: { type: 'hnsw' },
          quantization: { type: 'sq' },
        },
      },
      logging: { level: 'info' },
    })),
    post: vi.fn(async () => ({ ok: true })),
    put: vi.fn(async () => ({ ok: true })),
  }),
}));

// Toast is wired through the existing ToastProvider context.
vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  }),
}));

// Skip the Monaco mount in the test environment — it's network/canvas heavy
// and not what this test is verifying.
vi.mock('@/components/ui/CodeEditor', () => ({
  default: () => <div data-testid="code-editor-stub">[CodeEditor]</div>,
}));

describe('ConfigurationPage (Settings)', () => {
  it('renders the page heading and the three Settings cards', async () => {
    render(<MemoryRouter><ConfigurationPage /></MemoryRouter>);
    expect(await screen.findByRole('heading', { name: /Settings/i })).toBeTruthy();
    expect(screen.getByText(/General/i)).toBeTruthy();
    expect(screen.getByText(/Defaults/i)).toBeTruthy();
    expect(screen.getByText(/Raw config/i)).toBeTruthy();
  });

  it('embeds the (stubbed) Monaco editor for raw config editing', async () => {
    render(<MemoryRouter><ConfigurationPage /></MemoryRouter>);
    expect(await screen.findByTestId('code-editor-stub')).toBeTruthy();
  });
});
