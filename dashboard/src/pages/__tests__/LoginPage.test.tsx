import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import LoginPage from '../LoginPage';

const loginMock = vi.fn(async () => undefined);

vi.mock('@/contexts/AuthContext', async () => {
  const actual = await vi.importActual<typeof import('@/contexts/AuthContext')>('@/contexts/AuthContext');
  return {
    ...actual,
    useAuth: () => ({
      isAuthenticated: false,
      authRequired: true,
      user: null,
      token: null,
      login: loginMock,
      logout: vi.fn(),
      verifySession: vi.fn(async () => false),
      refreshToken: vi.fn(async () => false),
      isLoading: false,
    }),
  };
});

describe('LoginPage', () => {
  it('renders the brand mark, both fields, and the sign-in button', () => {
    render(<MemoryRouter><LoginPage /></MemoryRouter>);
    expect(screen.getByAltText(/Vectorizer/i)).toBeTruthy();
    expect(screen.getByLabelText(/Username/i)).toBeTruthy();
    expect(screen.getByLabelText(/Password/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /Sign in/i })).toBeTruthy();
  });

  it('activates the console body styling on mount', () => {
    render(<MemoryRouter><LoginPage /></MemoryRouter>);
    expect(document.body.dataset.console).toBe('1');
  });

  it('calls the auth login mutation on submit', async () => {
    render(<MemoryRouter><LoginPage /></MemoryRouter>);
    fireEvent.change(screen.getByLabelText(/Username/i), { target: { value: 'admin' } });
    fireEvent.change(screen.getByLabelText(/Password/i), { target: { value: 'secret' } });
    fireEvent.click(screen.getByRole('button', { name: /Sign in/i }));
    // Allow microtasks to flush
    await Promise.resolve();
    expect(loginMock).toHaveBeenCalledWith('admin', 'secret');
  });
});
