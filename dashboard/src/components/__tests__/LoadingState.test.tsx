/**
 * Unit tests for LoadingState component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import LoadingState from '../LoadingState';

describe('LoadingState', () => {
  it('should render loading message', () => {
    render(<LoadingState message="Loading data..." />);
    expect(screen.getByText('Loading data...')).toBeInTheDocument();
  });

  it('should render default message when no message provided', () => {
    render(<LoadingState />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('should render spinner', () => {
    const { container } = render(<LoadingState />);
    const spinner = container.querySelector('svg');
    expect(spinner).toBeInTheDocument();
  });
});

