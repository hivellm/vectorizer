/**
 * Test-only render helper + auth-state fake.
 *
 * Every page under `src/pages/` now renders inside an `AuthProvider`
 * (phase8_gate-data-routes-when-auth-enabled / F7), and its `useAuth`
 * hook throws `"useAuth must be used within an AuthProvider"` when
 * rendered bare. `renderWithProviders` wraps the subject in the
 * production provider stack — `AuthContext` via an injected auth
 * value (so tests do not need a live server to resolve auth state)
 * plus `MemoryRouter`, `ThemeProvider`, `ToastProvider`.
 *
 * Page-level tests should call `renderWithProviders(<Page />, {route})`
 * instead of `render(...)` directly.
 */

import { ReactElement, ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { render, RenderOptions, RenderResult } from '@testing-library/react';

import AuthContext, { User } from '@/contexts/AuthContext';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';

/**
 * Partial shape for `buildAuthState`. Any field not specified falls
 * back to a default that matches an unauthenticated session against
 * a server with `auth.enabled: false` (so `isAuthenticated` is `true`
 * by the same "auth not required → consider user authenticated" rule
 * the real `AuthProvider` applies).
 */
export interface AuthStateInput {
  user?: User | null;
  token?: string | null;
  isAuthenticated?: boolean;
  isLoading?: boolean;
  authRequired?: boolean;
}

/** The full value the real `AuthContext` provides, with test-friendly defaults. */
export type FakeAuthState = {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  authRequired: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  verifySession: () => Promise<boolean>;
  refreshToken: () => Promise<boolean>;
};

/**
 * Build a full `AuthContext` value for tests. The method bodies
 * are intentionally inert — tests should never exercise network
 * calls through these helpers. Event handlers that fire `login` /
 * `logout` during cleanup resolve successfully so the test does not
 * reject an unawaited promise.
 */
export function buildAuthState(partial: AuthStateInput = {}): FakeAuthState {
  const authRequired = partial.authRequired ?? false;
  const user = partial.user ?? null;
  const token = partial.token ?? null;
  const isAuthenticated =
    partial.isAuthenticated ?? (!authRequired || (!!token && !!user));
  return {
    user,
    token,
    isAuthenticated,
    isLoading: partial.isLoading ?? false,
    authRequired,
    login: async () => {
      /* inert in tests — assert auth behaviour through seeded state */
    },
    logout: async () => {
      /* inert in tests — assert auth behaviour through seeded state */
    },
    verifySession: async () => true,
    refreshToken: async () => true,
  };
}

export interface RenderWithProvidersOptions extends Omit<RenderOptions, 'wrapper'> {
  /** Initial route for the in-memory router. Defaults to `/`. */
  route?: string;
  /** Seed the auth context. Defaults to `buildAuthState()` (anonymous, auth not required). */
  authState?: AuthStateInput | FakeAuthState;
}

/**
 * Render `ui` inside the full production provider stack with an
 * injectable auth state. Returns the standard `@testing-library/react`
 * result so callers can destructure as usual.
 */
export function renderWithProviders(
  ui: ReactElement,
  options: RenderWithProvidersOptions = {}
): RenderResult {
  const { route = '/', authState, ...rtlOptions } = options;
  const resolvedAuth: FakeAuthState =
    authState && 'login' in authState
      ? (authState as FakeAuthState)
      : buildAuthState(authState as AuthStateInput | undefined);

  const Wrapper = ({ children }: { children: ReactNode }) => (
    <AuthContext.Provider value={resolvedAuth}>
      <MemoryRouter initialEntries={[route]}>
        <ThemeProvider>
          <ToastProvider>{children}</ToastProvider>
        </ThemeProvider>
      </MemoryRouter>
    </AuthContext.Provider>
  );

  return render(ui, { wrapper: Wrapper, ...rtlOptions });
}

// Re-exports so call sites can `import { screen, ... } from '@/test-utils/render'`
// and keep all their testing imports in a single line.
export { screen, fireEvent, waitFor, within } from '@testing-library/react';
