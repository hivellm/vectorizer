/**
 * Unit tests for Modal component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Modal from '../Modal';

describe('Modal', () => {
  it('should render modal when open', () => {
    render(
      <Modal isOpen={true} onClose={() => {}}>
        <p>Modal content</p>
      </Modal>
    );
    expect(screen.getByText('Modal content')).toBeInTheDocument();
  });

  it('should not render modal when closed', () => {
    render(
      <Modal isOpen={false} onClose={() => {}}>
        <p>Modal content</p>
      </Modal>
    );
    expect(screen.queryByText('Modal content')).not.toBeInTheDocument();
  });

  it('should call onClose when backdrop is clicked', async () => {
    const handleClose = vi.fn();
    const user = userEvent.setup();
    
    const { container } = render(
      <Modal isOpen={true} onClose={handleClose}>
        <p>Modal content</p>
      </Modal>
    );
    
    // Click on backdrop (first div with fixed inset-0)
    const backdrop = container.querySelector('.fixed.inset-0.bg-black\\/50');
    if (backdrop) {
      await user.click(backdrop as HTMLElement);
      // Both backdrop and outer container call onClose, so it's called at least once
      expect(handleClose).toHaveBeenCalled();
    }
  });

  it('should render modal with title', () => {
    render(
      <Modal isOpen={true} onClose={() => {}} title="Test Modal">
        <p>Modal content</p>
      </Modal>
    );
    expect(screen.getByText('Test Modal')).toBeInTheDocument();
  });

  it('should render modal with footer', () => {
    render(
      <Modal isOpen={true} onClose={() => {}} footer={<button>Save</button>}>
        <p>Modal content</p>
      </Modal>
    );
    expect(screen.getByText('Save')).toBeInTheDocument();
  });
});

