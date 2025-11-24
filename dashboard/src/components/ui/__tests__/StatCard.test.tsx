/**
 * Unit tests for StatCard component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import StatCard from '../StatCard';

describe('StatCard', () => {
  it('should render stat card with title and value', () => {
    render(<StatCard title="Total Collections" value="10" />);
    expect(screen.getByText('Total Collections')).toBeInTheDocument();
    expect(screen.getByText('10')).toBeInTheDocument();
  });

  it('should render stat card with icon', () => {
    const Icon = () => <svg data-testid="icon" />;
    render(<StatCard title="Test" value="100" icon={<Icon />} />);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('should render stat card with subtitle', () => {
    render(
      <StatCard
        title="Test"
        value="100"
        subtitle="Test subtitle"
      />
    );
    expect(screen.getByText('Test subtitle')).toBeInTheDocument();
  });

  it('should render stat card with trend', () => {
    render(
      <StatCard
        title="Test"
        value="100"
        trend={{ value: 10, isPositive: true }}
      />
    );
    // Trend shows percentage, so look for the % sign
    expect(screen.getByText('10%')).toBeInTheDocument();
  });

  it('should render positive trend with up arrow', () => {
    render(
      <StatCard
        title="Test"
        value="100"
        trend={{ value: 10, isPositive: true }}
      />
    );
    expect(screen.getByText('↑')).toBeInTheDocument();
  });

  it('should render negative trend with down arrow', () => {
    render(
      <StatCard
        title="Test"
        value="100"
        trend={{ value: 5, isPositive: false }}
      />
    );
    expect(screen.getByText('↓')).toBeInTheDocument();
  });
});

