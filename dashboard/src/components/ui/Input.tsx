/**
 * Input component — console design language.
 *
 * Public API preserved from the previous (Tailwind) implementation:
 *   { label, error, helperText, ...InputHTMLAttributes }
 */

import { forwardRef, useId } from 'react';
import type { InputHTMLAttributes, ReactNode } from 'react';

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: ReactNode;
  error?: ReactNode;
  helperText?: ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, helperText, className, id, ...props }, ref) => {
    const generatedId = useId();
    const inputId = id || generatedId;

    const cls = ['input', error ? 'input-error' : '', className ?? '']
      .filter(Boolean)
      .join(' ');

    if (label || error || helperText) {
      return (
        <div className="field" style={{ width: '100%' }}>
          {label && (
            <label className="field-label" htmlFor={inputId}>
              {label}
              {props.required && (
                <span style={{ color: 'var(--red)', marginLeft: 4 }}>*</span>
              )}
            </label>
          )}
          <input ref={ref} id={inputId} className={cls} {...props} />
          {error && (
            <div style={{ fontSize: 11, color: 'var(--red)', marginTop: 4 }}>{error}</div>
          )}
          {helperText && !error && (
            <div style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 4 }}>{helperText}</div>
          )}
        </div>
      );
    }

    return <input ref={ref} id={inputId} className={cls} {...props} />;
  }
);

Input.displayName = 'Input';
