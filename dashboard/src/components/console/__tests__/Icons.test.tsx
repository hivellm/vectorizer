import { render } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Icons } from '../Icons';

describe('Icons', () => {
  it('renders every named icon as inline SVG', () => {
    const names: Array<keyof typeof Icons> = [
      'dashboard', 'collections', 'search', 'vectors', 'monitor', 'keys', 'mcp',
      'settings', 'plus', 'zap', 'cpu', 'database', 'bolt', 'layers', 'activity',
      'chevron', 'copy', 'trash', 'bell', 'filter', 'sparkles', 'globe', 'shield',
      'flame', 'panel', 'panel2', 'arrowDown', 'arrowUp', 'check', 'x', 'refresh', 'hex',
    ];
    for (const name of names) {
      const Cmp = Icons[name];
      const { container } = render(<Cmp />);
      const svg = container.querySelector('svg');
      expect(svg, `icon ${name} should render`).toBeTruthy();
      expect(svg!.getAttribute('viewBox')).toBe('0 0 24 24');
    }
  });

  it('respects size prop', () => {
    const { container } = render(<Icons.search size={20} />);
    expect(container.querySelector('svg')!.getAttribute('width')).toBe('20');
    expect(container.querySelector('svg')!.getAttribute('height')).toBe('20');
  });
});
