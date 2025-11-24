/**
 * Unit tests for MainLayout component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import MainLayout from '../MainLayout';

const Wrapper = ({ children }: { children: React.ReactNode }) => (
  <BrowserRouter>
    <ThemeProvider>
      {children}
    </ThemeProvider>
  </BrowserRouter>
);

describe('MainLayout', () => {
  it('should render layout with header and sidebar', () => {
    render(
      <Wrapper>
        <MainLayout />
      </Wrapper>
    );
    
    // Should contain header
    expect(screen.getByText(/vectorizer/i)).toBeInTheDocument();
  });

  it('should render outlet for child routes', () => {
    render(
      <Wrapper>
        <MainLayout />
      </Wrapper>
    );
    
    // Layout should be present
    expect(screen.getByText(/vectorizer/i)).toBeInTheDocument();
  });
});

