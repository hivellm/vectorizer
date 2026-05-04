/**
 * Checkbox component — console design language.
 *
 * Public API preserved from the previous (Tailwind) implementation:
 *   { id, checked, onChange: (b: boolean) => void, label, disabled }
 */

import { useId } from 'react';
import type { ReactNode } from 'react';

interface CheckboxProps {
  id?: string;
  checked?: boolean;
  onChange?: (checked: boolean) => void;
  label?: ReactNode;
  disabled?: boolean;
}

function Checkbox({ id, checked, onChange, label, disabled = false }: CheckboxProps) {
  const generatedId = useId();
  const checkboxId = id || generatedId;
  return (
    <label
      htmlFor={checkboxId}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 8,
        cursor: disabled ? 'not-allowed' : 'pointer',
        userSelect: 'none',
        opacity: disabled ? 0.5 : 1,
      }}
    >
      <input
        id={checkboxId}
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange?.(e.target.checked)}
        disabled={disabled}
        style={{
          width: 14,
          height: 14,
          accentColor: 'var(--teal)',
          margin: 0,
        }}
      />
      {label && (
        <span style={{ fontSize: 13, color: 'var(--text-1)', lineHeight: 1 }}>{label}</span>
      )}
    </label>
  );
}

export default Checkbox;
