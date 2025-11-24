/**
 * Unit tests for Checkbox component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Checkbox from '../Checkbox';

describe('Checkbox', () => {
  it('should render checkbox with label', () => {
    render(<Checkbox label="Test checkbox" />);
    expect(screen.getByLabelText('Test checkbox')).toBeInTheDocument();
  });

  it('should be checked when checked prop is true', () => {
    render(<Checkbox label="Test" checked />);
    expect(screen.getByRole('checkbox')).toBeChecked();
  });

  it('should be unchecked when checked prop is false', () => {
    render(<Checkbox label="Test" checked={false} />);
    expect(screen.getByRole('checkbox')).not.toBeChecked();
  });

  it('should handle change events', async () => {
    const handleChange = vi.fn();
    const user = userEvent.setup();
    
    render(<Checkbox label="Test" onChange={handleChange} />);
    const checkbox = screen.getByRole('checkbox');
    
    await user.click(checkbox);
    expect(handleChange).toHaveBeenCalledTimes(1);
  });

  it('should be disabled when disabled prop is true', () => {
    render(<Checkbox label="Test" disabled />);
    expect(screen.getByRole('checkbox')).toBeDisabled();
  });

  it('should render without label', () => {
    render(<Checkbox checked={false} onChange={vi.fn()} />);
    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeInTheDocument();
  });
});

