/**
 * Card — console design.
 *
 * Thin wrapper that forwards `className`/HTML props onto a `.card`
 * container. Replaces the previous Tailwind-styled implementation.
 * The legacy `padding` prop is kept for API compatibility but is a
 * no-op (the console design owns padding via `.card`).
 */

import { forwardRef, type HTMLAttributes } from 'react';

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  /** Legacy padding prop — kept for API compatibility, currently unused. */
  padding?: 'none' | 'sm' | 'md' | 'lg';
}

const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ padding: _padding, className, children, ...rest }, ref) => {
    void _padding;
    const cls = ['card', className].filter(Boolean).join(' ');
    return (
      <div ref={ref} className={cls} {...rest}>
        {children}
      </div>
    );
  },
);

Card.displayName = 'Card';

export default Card;
