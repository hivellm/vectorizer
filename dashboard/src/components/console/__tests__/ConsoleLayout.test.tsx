import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { describe, it, expect } from 'vitest';
import { AuthProvider } from '@/contexts/AuthContext';
import { ConsoleLayout } from '../ConsoleLayout';

const Page = () => <div data-testid="page">PAGE</div>;

describe('ConsoleLayout', () => {
  it('renders sidebar, topbar and outlet', () => {
    render(
      <MemoryRouter initialEntries={['/overview']}>
        <AuthProvider>
          <Routes>
            <Route element={<ConsoleLayout />}>
              <Route path="/overview" element={<Page />} />
            </Route>
          </Routes>
        </AuthProvider>
      </MemoryRouter>,
    );
    // "Vectorizer" appears in both the sidebar logo and the topbar crumbs;
    // confirm at least one is rendered to prove the shell mounted.
    expect(screen.getAllByText('Vectorizer').length).toBeGreaterThan(0);
    expect(screen.getByText(/Search collections/)).toBeTruthy();
    expect(screen.getByTestId('page')).toBeTruthy();
    expect(document.body.dataset.console).toBe('1');
  });

  it('opens command palette on ⌘K', () => {
    render(
      <MemoryRouter initialEntries={['/overview']}>
        <AuthProvider>
          <Routes>
            <Route element={<ConsoleLayout />}>
              <Route path="/overview" element={<Page />} />
            </Route>
          </Routes>
        </AuthProvider>
      </MemoryRouter>,
    );
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    expect(screen.getByPlaceholderText(/Search or type a command/)).toBeTruthy();
  });
});
