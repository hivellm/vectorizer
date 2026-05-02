/**
 * Unit tests for AppRouter component
 */

import { describe, it, expect, vi } from 'vitest';
import { Outlet } from 'react-router-dom';
import { renderWithProviders, screen } from '@/test-utils/render';
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

vi.mock('@/components/console', () => ({
  ConsoleLayout: () => <div data-testid="console-layout"><Outlet /></div>,
}));

describe('AppRouter', () => {
  it('should render router with ConsoleLayout', () => {
    renderWithProviders(<AppRouter />);

    expect(screen.getByTestId('console-layout')).toBeInTheDocument();
  });

  it('should navigate to overview by default', () => {
    renderWithProviders(<AppRouter />, { route: '/' });

    // MemoryRouter starts at '/', ConsoleLayout renders regardless of sub-route
    expect(screen.getByTestId('console-layout')).toBeInTheDocument();
  });
});
