import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Modal } from '../primitives/Modal';

describe('Modal', () => {
  it('renders nothing when closed', () => {
    render(
      <Modal open={false} onClose={() => {}}>
        x
      </Modal>,
    );
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('closes on Escape', () => {
    const close = vi.fn();
    render(
      <Modal open onClose={close}>
        x
      </Modal>,
    );
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(close).toHaveBeenCalled();
  });

  it('closes on overlay click', () => {
    const close = vi.fn();
    render(
      <Modal open onClose={close}>
        x
      </Modal>,
    );
    fireEvent.click(screen.getByRole('dialog'));
    expect(close).toHaveBeenCalled();
  });

  it('does NOT close on panel click', () => {
    const close = vi.fn();
    render(
      <Modal open onClose={close} title="t">
        body
      </Modal>,
    );
    fireEvent.click(screen.getByText('body'));
    expect(close).not.toHaveBeenCalled();
  });
});
