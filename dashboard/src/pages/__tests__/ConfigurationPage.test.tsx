/**
 * Unit tests for ConfigurationPage component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';
import ConfigurationPage from '../ConfigurationPage';

// Mock hooks
vi.mock('@/hooks/useApiClient', () => ({
  useApiClient: () => ({
    get: vi.fn().mockResolvedValue({}),
    post: vi.fn().mockResolvedValue({}),
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

describe('ConfigurationPage', () => {
  it('should render configuration page', async () => {
    render(
      <Wrapper>
        <ConfigurationPage />
      </Wrapper>
    );
    
    // Wait for page to load
    await waitFor(() => {
      expect(screen.getByText(/configuration|settings|general|advanced|file watcher/i)).toBeInTheDocument();
    });
  });

  it('should render configuration tabs', async () => {
    render(
      <Wrapper>
        <ConfigurationPage />
      </Wrapper>
    );
    
    // Wait for tabs to load - use getAllByText since tabs might appear multiple times
    await waitFor(() => {
      const tabs = screen.getAllByText(/general|advanced|file watcher/i);
      expect(tabs.length).toBeGreaterThan(0);
    }, { timeout: 3000 });
  });
});

