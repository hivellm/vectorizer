/**
 * Unit tests for SearchPage component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';
import SearchPage from '../SearchPage';

// Mock hooks
vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: vi.fn().mockResolvedValue([
      { name: 'collection1', dimension: 512, metric: 'cosine' },
    ]),
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [{ name: 'collection1', dimension: 512, metric: 'cosine' }],
    setCollections: vi.fn(),
  }),
}));

vi.mock('@/hooks/useSearchHistory', () => ({
  useSearchHistory: () => ({
    history: [],
    addToHistory: vi.fn(),
    clearHistory: vi.fn(),
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

describe('SearchPage', () => {
  it('should render search page', () => {
    render(
      <Wrapper>
        <SearchPage />
      </Wrapper>
    );
    
    // Check for search input instead of text
    const searchInput = screen.getByPlaceholderText(/search|query/i);
    expect(searchInput).toBeInTheDocument();
  });

  it('should render search input', () => {
    render(
      <Wrapper>
        <SearchPage />
      </Wrapper>
    );
    
    const searchInput = screen.getByPlaceholderText(/search|query/i);
    expect(searchInput).toBeInTheDocument();
  });
});

