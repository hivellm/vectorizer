import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import ApiKeysPage from '../ApiKeysPage';

// Mock auth context (used transitively by useApiKeys / useApiClient call sites).
vi.mock('@/contexts/AuthContext', () => ({
  useAuth: () => ({ token: 'test-token', user: null }),
}));

// ApiKeysPage uses `useToastContext`; stub the barrel so it renders without a
// ToastProvider.
vi.mock('@/providers/ToastProvider', () => ({
  useToastContext: () => ({
    show: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

// Mock the API client used by ApiKeysPage to fetch the keys list.
// The legacy /auth/keys response shape is { keys: ApiKey[] }.
vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({
      keys: [
        {
          id: '1',
          name: 'production-server',
          key_prefix: 'vk_live_8a2c',
          permissions: ['read', 'write'],
          role: 'ReadWrite',
          calls: 4_812_330,
          last_used_at: '12s ago',
          created_at: '2026-01-12',
        },
        {
          id: '2',
          name: 'ci-cd-pipeline',
          key_prefix: 'vk_live_f04b',
          permissions: ['admin'],
          role: 'Admin',
          calls: 92_104,
          last_used_at: '4m ago',
          created_at: '2026-02-08',
        },
      ],
    })),
    post: vi.fn(async () => ({})),
    delete: vi.fn(async () => ({})),
  }),
}));

// Mock the createApiKey hook surface.
vi.mock('@/hooks/useApiKeys', () => ({
  useApiKeys: () => ({
    createApiKey: vi.fn(async () => ({})),
    loading: false,
    error: null,
  }),
}));

describe('ApiKeysPage', () => {
  it('renders the page heading and the keys table column headers', async () => {
    render(<MemoryRouter><ApiKeysPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /API Keys/i })).toBeTruthy();
    // The keys table renders only after the async fetch resolves with rows.
    await screen.findByText('production-server');
    for (const col of ['Name', 'Key', 'Role', 'Created']) {
      expect(screen.getAllByText(col).length).toBeGreaterThan(0);
    }
  });

  it('renders the permission matrix card', () => {
    render(<MemoryRouter><ApiKeysPage /></MemoryRouter>);
    expect(screen.getByText(/Permission matrix/i)).toBeTruthy();
    // The matrix lists role variants as pills
    expect(screen.getAllByText('Admin').length).toBeGreaterThan(0);
    expect(screen.getByText('ReadOnly')).toBeTruthy();
  });

  it('renders a row per fetched key', async () => {
    render(<MemoryRouter><ApiKeysPage /></MemoryRouter>);
    expect(await screen.findByText('production-server')).toBeTruthy();
    expect(await screen.findByText('ci-cd-pipeline')).toBeTruthy();
  });
});
