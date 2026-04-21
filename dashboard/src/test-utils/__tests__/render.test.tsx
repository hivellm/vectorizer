/**
 * Self-tests for the `renderWithProviders` helper.
 *
 * These assert that (a) the helper injects both `AuthContext` and
 * `MemoryRouter` so `useAuth` resolves without throwing, and
 * (b) `buildAuthState` round-trips a seeded user / authentication
 * state into the context value children observe.
 */

import { describe, it, expect } from 'vitest';
import { useAuth } from '@/contexts/AuthContext';
import { useLocation } from 'react-router-dom';

import {
  buildAuthState,
  renderWithProviders,
  screen,
} from '@/test-utils/render';

function AuthProbe() {
  const { user, isAuthenticated, authRequired } = useAuth();
  return (
    <div>
      <span data-testid="user-name">{user?.username ?? '<anonymous>'}</span>
      <span data-testid="is-auth">{String(isAuthenticated)}</span>
      <span data-testid="auth-required">{String(authRequired)}</span>
    </div>
  );
}

function RouteProbe() {
  const location = useLocation();
  return <span data-testid="pathname">{location.pathname}</span>;
}

describe('renderWithProviders', () => {
  it('injects AuthContext with an anonymous, auth-not-required default', () => {
    renderWithProviders(<AuthProbe />);
    expect(screen.getByTestId('user-name').textContent).toBe('<anonymous>');
    expect(screen.getByTestId('is-auth').textContent).toBe('true');
    expect(screen.getByTestId('auth-required').textContent).toBe('false');
  });

  it('round-trips a seeded authenticated user through buildAuthState', () => {
    const seeded = buildAuthState({
      user: { user_id: 'u-1', username: 'alice', roles: ['admin'] },
      token: 'tok-abc',
      authRequired: true,
    });
    renderWithProviders(<AuthProbe />, { authState: seeded });
    expect(screen.getByTestId('user-name').textContent).toBe('alice');
    expect(screen.getByTestId('is-auth').textContent).toBe('true');
    expect(screen.getByTestId('auth-required').textContent).toBe('true');
  });

  it('respects the requested route via MemoryRouter', () => {
    renderWithProviders(<RouteProbe />, { route: '/collections/abc' });
    expect(screen.getByTestId('pathname').textContent).toBe('/collections/abc');
  });
});
