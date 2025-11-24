/**
 * Unit tests for Toast component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ToastContainer } from '../Toast';
import type { Toast } from '../Toast';

describe('ToastContainer', () => {
    it('should render nothing when toasts array is empty', () => {
        const { container } = render(<ToastContainer toasts={[]} onClose={vi.fn()} />);
        expect(container.firstChild).toBeNull();
    });

    it('should render toast with message', () => {
        const toasts: Toast[] = [{
            id: '1',
            message: 'Test message',
            type: 'info',
        }];
        const onClose = vi.fn();

        render(<ToastContainer toasts={toasts} onClose={onClose} />);
        expect(screen.getByText('Test message')).toBeInTheDocument();
    });

    it('should render multiple toasts', () => {
        const toasts: Toast[] = [
            { id: '1', message: 'First', type: 'success' },
            { id: '2', message: 'Second', type: 'error' },
        ];
        const onClose = vi.fn();

        render(<ToastContainer toasts={toasts} onClose={onClose} />);
        expect(screen.getByText('First')).toBeInTheDocument();
        expect(screen.getByText('Second')).toBeInTheDocument();
    });

    it('should call onClose when close button is clicked', async () => {
        const toasts: Toast[] = [{
            id: '1',
            message: 'Test',
            type: 'info',
        }];
        const onClose = vi.fn();
        const user = userEvent.setup();

        render(<ToastContainer toasts={toasts} onClose={onClose} />);

        // Find close button by its SVG icon (XMarkIcon)
        const closeButton = screen.getByRole('button', { hidden: false });

        await user.click(closeButton);

        // The onClose is called after 300ms timeout, so we need to wait
        await new Promise(resolve => setTimeout(resolve, 350));

        expect(onClose).toHaveBeenCalledWith('1');
    });
});

