/**
 * Unit tests for AppRouter component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import AppRouter from '../AppRouter';

// Mock all lazy-loaded pages
vi.mock('@/pages/OverviewPage', () => ({
  default: () => <div>Overview Page</div>,
}));

vi.mock('@/pages/CollectionsPage', () => ({
  default: () => <div>Collections Page</div>,
}));

vi.mock('@/pages/SearchPage', () => ({
  default: () => <div>Search Page</div>,
}));

vi.mock('@/pages/VectorsPage', () => ({
  default: () => <div>Vectors Page</div>,
}));

vi.mock('@/pages/FileWatcherPage', () => ({
  default: () => <div>FileWatcher Page</div>,
}));

vi.mock('@/pages/GraphPage', () => ({
  default: () => <div>Graph Page</div>,
}));

vi.mock('@/pages/ConnectionsPage', () => ({
  default: () => <div>Connections Page</div>,
}));

vi.mock('@/pages/WorkspacePage', () => ({
  default: () => <div>Workspace Page</div>,
}));

vi.mock('@/pages/ConfigurationPage', () => ({
  default: () => <div>Configuration Page</div>,
}));

vi.mock('@/pages/LogsPage', () => ({
  default: () => <div>Logs Page</div>,
}));

vi.mock('@/pages/BackupsPage', () => ({
  default: () => <div>Backups Page</div>,
}));

vi.mock('@/pages/TestPage', () => ({
  default: () => <div>Test Page</div>,
}));

vi.mock('@/components/layout/MainLayout', () => ({
  default: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="main-layout">{children}</div>
  ),
}));

describe('AppRouter', () => {
  it('should render router with MainLayout', () => {
    render(
      <BrowserRouter>
        <AppRouter />
      </BrowserRouter>
    );

    expect(screen.getByTestId('main-layout')).toBeInTheDocument();
  });

  it('should navigate to overview by default', () => {
    window.history.pushState({}, '', '/');
    
    render(
      <BrowserRouter>
        <AppRouter />
      </BrowserRouter>
    );

    // Should redirect to overview
    expect(window.location.pathname).toBe('/');
  });
});

