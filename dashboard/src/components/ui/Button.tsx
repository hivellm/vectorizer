/**
 * Button — console design.
 *
 * Maps the legacy variant/size API onto the console `.btn` class
 * composition (see `console.css`). The `forwardRef` signature and
 * the `variant`/`size`/`isLoading` prop names are preserved so any
 * downstream consumer (e.g. FileBrowser) keeps working.
 */

import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  isLoading?: boolean;
  leftIcon?: ReactNode;
  rightIcon?: ReactNode;
  fullWidth?: boolean;
}

const VARIANT_TO_CLASS: Record<NonNullable<ButtonProps['variant']>, string> = {
  primary: 'btn primary',
  secondary: 'btn',
  outline: 'btn',
  ghost: 'btn ghost',
  danger: 'btn magenta',
};

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      isLoading = false,
      leftIcon,
      rightIcon,
      fullWidth,
      className = '',
      children,
      disabled,
      type,
      style,
      ...rest
    },
    ref,
  ) => {
    const cls = [
      VARIANT_TO_CLASS[variant],
      size === 'sm' ? 'sm' : '',
      className,
    ]
      .filter(Boolean)
      .join(' ');

    const composedStyle = fullWidth
      ? { width: '100%', justifyContent: 'center', ...(style ?? {}) }
      : style;

    return (
      <button
        ref={ref}
        type={type ?? 'button'}
        className={cls}
        disabled={disabled || isLoading}
        style={composedStyle}
        {...rest}
      >
        {isLoading && (
          <span
            className="spinner"
            style={{ width: 12, height: 12, borderWidth: 1.5 }}
            aria-hidden
          />
        )}
        {leftIcon}
        {children}
        {rightIcon}
      </button>
    );
  },
);

Button.displayName = 'Button';

export default Button;
