/**
 * Unit tests for Sidebar component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/providers/ThemeProvider';
import Sidebar from '../Sidebar';

const Wrapper = ({ children }: { children: React.ReactNode }) => (
  <BrowserRouter>
    <ThemeProvider>
      {children}
    </ThemeProvider>
  </BrowserRouter>
);

describe('Sidebar', () => {
  it('should render sidebar with navigation items', () => {
    render(
      <Wrapper>
        <Sidebar />
      </Wrapper>
    );
    
    // Should contain navigation links
    expect(screen.getByText(/overview/i)).toBeInTheDocument();
    expect(screen.getByText(/collections/i)).toBeInTheDocument();
  });

  it('should render all main navigation items', () => {
    render(
      <Wrapper>
        <Sidebar />
      </Wrapper>
    );
    
    const navItems = [
      /overview/i,
      /collections/i,
      /search/i,
      /vectors/i,
    ];
    
    navItems.forEach((item) => {
      expect(screen.getByText(item)).toBeInTheDocument();
    });
  });

  it('should render Vectorizer branding', () => {
    render(
      <Wrapper>
        <Sidebar />
      </Wrapper>
    );
    
    expect(screen.getByText(/vectorizer/i)).toBeInTheDocument();
  });
});

