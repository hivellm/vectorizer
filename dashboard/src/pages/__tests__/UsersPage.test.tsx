import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import UsersPage from '../UsersPage';

// AuthContext is consumed by the page (currentUser badge + Bearer token).
vi.mock('@/contexts/AuthContext', () => ({
  useAuth: () => ({
    token: 'test-token',
    user: { user_id: '1', username: 'admin', roles: ['Admin'] },
  }),
}));

// The real `ApiClient.get<T>` returns `T` directly (not `{ data: T }`).
// The legacy /auth/users response shape is `{ users: User[] }`.
vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn(async () => ({
      users: [
        {
          user_id: '1',
          username: 'admin',
          roles: ['Admin'],
          created_at: '2026-01-01T00:00:00Z',
          last_login_at: '2026-05-02T12:00:00Z',
        },
        {
          user_id: '2',
          username: 'editor',
          roles: ['User'],
          created_at: '2026-02-01T00:00:00Z',
          last_login_at: '2026-05-01T08:00:00Z',
        },
      ],
    })),
    post: vi.fn(async () => ({})),
    put: vi.fn(async () => ({})),
    delete: vi.fn(async () => ({})),
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

describe('UsersPage', () => {
  it('renders the page heading and the users table headers', async () => {
    render(<MemoryRouter><UsersPage /></MemoryRouter>);
    expect(screen.getByRole('heading', { name: /Users/i })).toBeTruthy();
    // The users table renders only after the async fetch resolves with rows.
    await screen.findByText('editor');
    for (const col of ['Username', 'Roles']) {
      expect(screen.getAllByText(col).length).toBeGreaterThan(0);
    }
  });
});
