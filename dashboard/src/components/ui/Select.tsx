/**
 * Select component — console design language.
 *
 * Public API preserved (`<Select>` + `<SelectOption>`); we still use
 * `react-aria-components` for accessibility, but every Tailwind class
 * is replaced with inline styles or console.css classes.
 */

import * as React from 'react';
import {
  Select as AriaSelect,
  SelectValue,
  Button,
  ListBox,
  ListBoxItem,
  Popover,
} from 'react-aria-components';

const ChevronDownIcon = ({ size = 14 }: { size?: number }) => (
  <svg
    width={size}
    height={size}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
    aria-hidden
  >
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
  </svg>
);

interface SelectProps {
  label?: string;
  value?: string;
  onChange?: (value: string) => void;
  children: React.ReactNode;
  placeholder?: string;
  isDisabled?: boolean;
  disabled?: boolean;
  required?: boolean;
  className?: string;
}

interface SelectOptionProps {
  id: string;
  value: string;
  children: React.ReactNode;
}

const triggerStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  width: '100%',
  background: 'var(--bg-2)',
  border: '1px solid var(--border)',
  color: 'var(--text)',
  fontSize: 13,
  padding: '8px 12px',
  borderRadius: 6,
  outline: 'none',
  textAlign: 'left',
  cursor: 'pointer',
  fontFamily: 'inherit',
};

const popoverStyle: React.CSSProperties = {
  outline: 'none',
  zIndex: 50,
};

const listBoxStyle: React.CSSProperties = {
  minWidth: 200,
  background: 'var(--panel-hi)',
  border: '1px solid var(--border)',
  borderRadius: 'var(--radius)',
  boxShadow: 'var(--shadow-lg)',
  padding: 4,
  maxHeight: 240,
  overflow: 'auto',
};

const itemStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderRadius: 4,
  fontSize: 13,
  color: 'var(--text-1)',
  cursor: 'pointer',
  outline: 'none',
};

export function Select({
  label,
  value,
  onChange,
  children,
  placeholder = 'Select...',
  isDisabled = false,
  disabled = false,
  required = false,
  className = '',
}: SelectProps) {
  const effectiveDisabled = isDisabled || disabled;
  return (
    <div className={`field ${className}`.trim()}>
      {label && (
        <label className="field-label">
          {label}
          {required && <span style={{ color: 'var(--red)', marginLeft: 4 }}>*</span>}
        </label>
      )}
      <AriaSelect
        selectedKey={value || null}
        onSelectionChange={(key) => {
          if (onChange && typeof key === 'string') {
            onChange(key);
          }
        }}
        isDisabled={effectiveDisabled}
      >
        <Button style={triggerStyle}>
          <SelectValue>
            {({ selectedText }) =>
              selectedText || (
                <span style={{ color: 'var(--text-2)' }}>{placeholder}</span>
              )
            }
          </SelectValue>
          <ChevronDownIcon />
        </Button>
        <Popover placement="bottom start" style={popoverStyle}>
          <ListBox style={listBoxStyle}>{children}</ListBox>
        </Popover>
      </AriaSelect>
    </div>
  );
}

export function SelectOption({ id, value, children }: SelectOptionProps) {
  return (
    <ListBoxItem
      id={id}
      textValue={value}
      style={({ isFocused, isSelected }) => ({
        ...itemStyle,
        background:
          isFocused || isSelected ? 'var(--bg-3)' : 'transparent',
      })}
    >
      {children}
    </ListBoxItem>
  );
}

// Export as namespace for easier usage
Select.Option = SelectOption;
