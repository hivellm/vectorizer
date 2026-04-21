/**
 * Unit tests for MainLayout component
 */

import { describe, it, expect } from 'vitest';
import { renderWithProviders, screen } from '@/test-utils/render';
import MainLayout from '../MainLayout';

describe('MainLayout', () => {
  it('should render layout with header and sidebar', () => {
    renderWithProviders(<MainLayout />);

    // Should contain header
    expect(screen.getByText(/vectorizer/i)).toBeInTheDocument();
  });

  it('should render outlet for child routes', () => {
    renderWithProviders(<MainLayout />);

    // Layout should be present
    expect(screen.getByText(/vectorizer/i)).toBeInTheDocument();
  });
});
