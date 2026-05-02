import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { ConsoleTopbar } from '../ConsoleTopbar';

describe('ConsoleTopbar', () => {
  it('renders crumbs with the last segment as current', () => {
    render(<ConsoleTopbar crumbs={['Vectorizer', 'Collections']} onOpenCmd={() => {}} />);
    expect(screen.getByText('Vectorizer')).toBeTruthy();
    expect(screen.getByText('Collections').className).toContain('now');
  });

  it('opens command palette on click', () => {
    let opened = 0;
    render(<ConsoleTopbar crumbs={['x']} onOpenCmd={() => { opened++; }} />);
    fireEvent.click(screen.getByText(/Search collections/));
    expect(opened).toBe(1);
  });
});
