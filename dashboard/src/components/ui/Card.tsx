/**
 * Card component based on Untitled UI - Dark mode support
 */

import { type HTMLAttributes, forwardRef } from 'react';

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  padding?: 'none' | 'sm' | 'md' | 'lg';
}

const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ padding = 'md', className = '', children, ...props }, ref) => {
    const paddingClasses = {
      none: '',
      sm: 'p-4',
      md: 'p-6',
      lg: 'p-8',
    };

    const classes = `bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-800/50 ${paddingClasses[padding]} ${className}`;

    return (
      <div ref={ref} className={classes} {...props}>
        {children}
      </div>
    );
  },
);

Card.displayName = 'Card';

export default Card;
