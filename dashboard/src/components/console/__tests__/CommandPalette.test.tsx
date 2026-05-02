import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi } from 'vitest';
import { CommandPalette } from '../CommandPalette';

const setup = (open = true) => {
  const onClose = vi.fn();
  const navigate = vi.fn();
  render(
    <MemoryRouter>
      <CommandPalette open={open} onClose={onClose} onNavigate={navigate} />
    </MemoryRouter>,
  );
  return { onClose, navigate };
};

describe('CommandPalette', () => {
  it('renders nothing when closed', () => {
    setup(false);
    expect(screen.queryByPlaceholderText(/Search or type a command/)).toBeNull();
  });

  it('navigates on Enter', () => {
    const { navigate } = setup();
    const input = screen.getByPlaceholderText(/Search or type a command/) as HTMLInputElement;
    fireEvent.change(input, { target: { value: 'Overview' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(navigate).toHaveBeenCalledWith('/overview');
  });

  it('closes on Escape', () => {
    const { onClose } = setup();
    fireEvent.keyDown(screen.getByPlaceholderText(/Search or type a command/), { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });
});
