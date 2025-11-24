/**
 * Unit tests for FileWatcherPage component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';
import FileWatcherPage from '../FileWatcherPage';

// Mock hooks
vi.mock('@/hooks/useFileWatcher', () => ({
  useFileWatcher: () => ({
    getStatus: vi.fn().mockResolvedValue({
      enabled: true,
      running: false,
      watch_paths: [],
    }),
    getMetrics: vi.fn().mockResolvedValue({
      timing: { uptime_seconds: 0 },
      files: { total_files_processed: 0 },
      status: { is_healthy: true },
    }),
  }),
}));

const Wrapper = ({ children }: { children: React.ReactNode }) => (
  <BrowserRouter>
    <ThemeProvider>
      <ToastProvider>
        {children}
      </ToastProvider>
    </ThemeProvider>
  </BrowserRouter>
);

describe('FileWatcherPage', () => {
  it('should render file watcher page', async () => {
    render(
      <Wrapper>
        <FileWatcherPage />
      </Wrapper>
    );
    
    // Wait for page to load
    await waitFor(() => {
      expect(screen.getByText(/file watcher|status/i)).toBeInTheDocument();
    });
  });

  it('should render file watcher content', async () => {
    render(
      <Wrapper>
        <FileWatcherPage />
      </Wrapper>
    );
    
    // Wait for page to load - check for any file watcher content
    await waitFor(() => {
      const content = screen.queryByText(/file watcher|enabled|disabled|metrics/i);
      expect(content).toBeInTheDocument();
    }, { timeout: 3000 });
  });
});

