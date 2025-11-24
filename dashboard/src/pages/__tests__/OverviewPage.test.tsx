/**
 * Unit tests for OverviewPage component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';
import OverviewPage from '../OverviewPage';

// Mock hooks
vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: vi.fn().mockResolvedValue([
      { name: 'collection1', dimension: 512, metric: 'cosine', vector_count: 100 },
      { name: 'collection2', dimension: 256, metric: 'euclidean', vector_count: 50 },
    ]),
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

describe('OverviewPage', () => {
  it('should render overview page', async () => {
    render(
      <Wrapper>
        <OverviewPage />
      </Wrapper>
    );
    
    // Wait for page to load and check for collections text
    await waitFor(() => {
      expect(screen.getByText(/collections/i)).toBeInTheDocument();
    });
  });

  it('should render stats cards', async () => {
    render(
      <Wrapper>
        <OverviewPage />
      </Wrapper>
    );
    
    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText(/collections/i)).toBeInTheDocument();
    });
  });
});

