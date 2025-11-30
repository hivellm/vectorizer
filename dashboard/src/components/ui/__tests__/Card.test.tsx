/**
 * Unit tests for Card component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Card from '../Card';

describe('Card', () => {
  it('should render card with children', () => {
    render(
      <Card>
        <p>Card content</p>
      </Card>
    );
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('should render card with title attribute', () => {
    const { container } = render(
      <Card title="Test Title">
        <p>Card content</p>
      </Card>
    );
    expect(container.querySelector('[title="Test Title"]')).toBeInTheDocument();
  });

  it('should apply custom className', () => {
    const { container } = render(
      <Card className="custom-class">
        <p>Card content</p>
      </Card>
    );
    expect(container.firstChild).toHaveClass('custom-class');
  });

});

