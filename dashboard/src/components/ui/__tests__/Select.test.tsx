/**
 * Unit tests for Select component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Select, SelectOption } from '../Select';

describe('Select', () => {
  it('should render select with placeholder', () => {
    render(
      <Select placeholder="Choose option">
        <SelectOption id="1" value="option1">Option 1</SelectOption>
      </Select>
    );
    expect(screen.getByText('Choose option')).toBeInTheDocument();
  });

  it('should render select with label', () => {
    render(
      <Select label="Select an option">
        <SelectOption id="1" value="option1">Option 1</SelectOption>
      </Select>
    );
    expect(screen.getByText('Select an option')).toBeInTheDocument();
  });

  it('should be disabled when isDisabled is true', () => {
    render(
      <Select isDisabled label="Test">
        <SelectOption id="1" value="option1">Option 1</SelectOption>
      </Select>
    );
    const button = screen.getByRole('button');
    // React Aria Components may use disabled attribute instead
    expect(button).toHaveAttribute('disabled');
  });

  it('should apply custom className', () => {
    const { container } = render(
      <Select className="custom-select">
        <SelectOption id="1" value="option1">Option 1</SelectOption>
      </Select>
    );
    expect(container.firstChild).toHaveClass('custom-select');
  });
});

