/**
 * Unit tests for CollectionsPage component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { ToastProvider } from '@/providers/ToastProvider';
import CollectionsPage from '../CollectionsPage';

// Mock hooks
vi.mock('@/hooks/useCollections', () => ({
  useCollections: () => ({
    listCollections: vi.fn().mockResolvedValue([]),
    createCollection: vi.fn(),
    deleteCollection: vi.fn(),
  }),
}));

vi.mock('@/stores/collections', () => ({
  useCollectionsStore: () => ({
    collections: [],
    loading: false,
    error: null,
    setCollections: vi.fn(),
    setLoading: vi.fn(),
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

describe('CollectionsPage', () => {
  it('should render collections page', () => {
    render(
      <Wrapper>
        <CollectionsPage />
      </Wrapper>
    );
    
    // Use getAllByText since "collections" appears multiple times
    const collectionsElements = screen.getAllByText(/collections/i);
    expect(collectionsElements.length).toBeGreaterThan(0);
  });

  it('should render create collection button', () => {
    render(
      <Wrapper>
        <CollectionsPage />
      </Wrapper>
    );
    
    // Use getAllByText since "create" might appear multiple times
    const createElements = screen.getAllByText(/create|new/i);
    expect(createElements.length).toBeGreaterThan(0);
  });
});

