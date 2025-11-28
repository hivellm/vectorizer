/**
 * Unit tests for Header component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BrowserRouter } from 'react-router-dom';
import Header from '../Header';

describe('Header', () => {
  it('should render header with menu button', () => {
    const onMenuClick = vi.fn();
    render(
      <BrowserRouter>
        <Header onMenuClick={onMenuClick} />
      </BrowserRouter>
    );
    
    const menuButton = screen.getByRole('button');
    expect(menuButton).toBeInTheDocument();
  });

  it('should call onMenuClick when menu button is clicked', async () => {
    const onMenuClick = vi.fn();
    const user = userEvent.setup();
    
    render(
      <BrowserRouter>
        <Header onMenuClick={onMenuClick} />
      </BrowserRouter>
    );
    const menuButton = screen.getByRole('button');
    
    await user.click(menuButton);
    expect(onMenuClick).toHaveBeenCalledTimes(1);
  });

  it('should render header title', () => {
    const onMenuClick = vi.fn();
    render(
      <BrowserRouter>
        <Header onMenuClick={onMenuClick} />
      </BrowserRouter>
    );
    
    // Header should show page title (defaults to Dashboard)
    expect(screen.getByText(/dashboard|overview/i)).toBeInTheDocument();
  });
});

