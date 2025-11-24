/**
 * Unit tests for Input component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Input } from '../Input';

describe('Input', () => {
    it('should render input with placeholder', () => {
        render(<Input placeholder="Enter text" />);
        expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
    });

    it('should handle value changes', async () => {
        const handleChange = vi.fn();
        const user = userEvent.setup();

        render(<Input onChange={handleChange} />);
        const input = screen.getByRole('textbox');

        await user.type(input, 'test');
        expect(handleChange).toHaveBeenCalled();
    });

    it('should be disabled when disabled prop is true', () => {
        render(<Input disabled />);
        expect(screen.getByRole('textbox')).toBeDisabled();
    });

    it('should show error message when error prop is provided', () => {
        render(<Input error="This field is required" />);
        expect(screen.getByText('This field is required')).toBeInTheDocument();
    });

    it('should show label when label prop is provided', () => {
        render(<Input label="Username" />);
        expect(screen.getByText('Username')).toBeInTheDocument();
    });

    it('should apply custom className', () => {
        const { container } = render(<Input className="custom-class" />);
        expect(container.querySelector('input')).toHaveClass('custom-class');
    });
});

