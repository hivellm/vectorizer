/**
 * Unit tests for LoadingSpinner component
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import LoadingSpinner from '../LoadingSpinner';

describe('LoadingSpinner', () => {
  it('should render spinner', () => {
    const { container } = render(<LoadingSpinner />);
    const spinner = container.querySelector('.spinner');
    expect(spinner).toBeInTheDocument();
  });

  it('should apply size dimensions correctly', () => {
    const { rerender, container } = render(<LoadingSpinner size="sm" />);
    let spinner = container.querySelector('.spinner') as HTMLElement | null;
    expect(spinner?.style.width).toBe('14px');
    expect(spinner?.style.height).toBe('14px');

    rerender(<LoadingSpinner size="md" />);
    spinner = container.querySelector('.spinner') as HTMLElement | null;
    expect(spinner?.style.width).toBe('20px');
    expect(spinner?.style.height).toBe('20px');

    rerender(<LoadingSpinner size="lg" />);
    spinner = container.querySelector('.spinner') as HTMLElement | null;
    expect(spinner?.style.width).toBe('32px');
    expect(spinner?.style.height).toBe('32px');
  });

  it('should apply custom className', () => {
    const { container } = render(<LoadingSpinner className="custom-class" />);
    expect(container.firstChild).toHaveClass('custom-class');
  });
});
