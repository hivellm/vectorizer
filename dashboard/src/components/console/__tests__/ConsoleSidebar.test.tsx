import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect } from 'vitest';
import { ConsoleSidebar } from '../ConsoleSidebar';

const renderAt = (path: string) =>
  render(
    <MemoryRouter initialEntries={[path]}>
      <ConsoleSidebar collapsed={false} onToggleCollapsed={() => {}} />
    </MemoryRouter>,
  );

describe('ConsoleSidebar', () => {
  it('renders all primary navigation links', () => {
    renderAt('/overview');
    for (const label of [
      'Overview', 'Collections', 'Search', 'Vectors', 'Monitoring',
      'Replication', 'API Keys', 'MCP Tools', 'Settings',
    ]) {
      expect(screen.getByText(label)).toBeTruthy();
    }
  });

  it('marks the active route', () => {
    renderAt('/collections');
    const item = screen.getByText('Collections').closest('a, [role="link"], div');
    expect(item?.className).toContain('active');
  });

  it('hides labels when collapsed', () => {
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={true} onToggleCollapsed={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.queryByText('Overview')).toBeNull();
  });

  it('calls onToggleCollapsed', () => {
    let toggled = 0;
    render(
      <MemoryRouter>
        <ConsoleSidebar collapsed={false} onToggleCollapsed={() => { toggled++; }} />
      </MemoryRouter>,
    );
    fireEvent.click(screen.getByText(/collapse sidebar/i));
    expect(toggled).toBe(1);
  });
});
