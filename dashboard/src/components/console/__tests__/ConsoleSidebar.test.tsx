import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi } from 'vitest';
import { ConsoleSidebar } from '../ConsoleSidebar';

// Mock useOptionalAuth so the sidebar renders its footer (avatar + info)
// in tests without requiring an AuthProvider in the tree. Username is
// intentionally empty to exercise the "VZ" avatar fallback.
vi.mock('@/contexts/AuthContext', async () => {
  const actual = await vi.importActual<typeof import('@/contexts/AuthContext')>(
    '@/contexts/AuthContext',
  );
  return {
    ...actual,
    useOptionalAuth: () => ({
      user: { user_id: 'u1', username: '', roles: ['User'] },
      token: null,
      isAuthenticated: true,
      isLoading: false,
      authRequired: false,
      login: async () => {},
      logout: async () => {},
      verifySession: async () => true,
      refreshToken: async () => true,
    }),
  };
});

const renderAt = (path: string) =>
  render(
    <MemoryRouter initialEntries={[path]}>
      <ConsoleSidebar collapsed={false} onToggleCollapsed={() => {}} />
    </MemoryRouter>,
  );

describe('ConsoleSidebar', () => {
  it('renders all primary navigation links', () => {
    renderAt('/overview');
    for (const label of [
      'Overview', 'Collections', 'Search', 'Vectors', 'Monitoring',
      'Replication', 'API Keys', 'MCP Tools', 'Settings',
    ]) {
      expect(screen.getByText(label)).toBeTruthy();
    }
  });

  it('marks the active route', () => {
    renderAt('/collections');
    const item = screen.getByText('Collections').closest('a, [role="link"], div');
    expect(item?.className).toContain('active');
  });

  it('hides labels when collapsed', () => {
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={true} onToggleCollapsed={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.queryByText('Overview')).toBeNull();
  });

  it('calls onToggleCollapsed', () => {
    let toggled = 0;
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={false} onToggleCollapsed={() => { toggled++; }} />
      </MemoryRouter>,
    );
    fireEvent.click(screen.getByRole('button', { name: /collapse|expand sidebar/i }));
    expect(toggled).toBe(1);
  });

  it('falls back to "VZ" when username is empty', () => {
    renderAt('/overview');
    expect(screen.getByText('VZ')).toBeTruthy();
  });

  it('renders the version pill from the version prop', () => {
    render(
      <MemoryRouter>
        <ConsoleSidebar
          collapsed={false}
          onToggleCollapsed={() => {}}
          version="3.2.1"
        />
      </MemoryRouter>,
    );
    expect(screen.getByText('3.2.1')).toBeTruthy();
  });

  it('shows em-dash when version is missing', () => {
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={false} onToggleCollapsed={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.getByText('—')).toBeTruthy();
  });
});
