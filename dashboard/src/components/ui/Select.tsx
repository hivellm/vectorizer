/**
 * Select component based on Untitled UI
 * Uses React Aria Components
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

// ChevronDown icon
const ChevronDownIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
  className?: string;
}

interface SelectOptionProps {
  id: string;
  value: string;
  children: React.ReactNode;
}

export function Select({
  label,
  value,
  onChange,
  children,
  placeholder = 'Select...',
  isDisabled = false,
  className = '',
}: SelectProps) {
  return (
    <div className={`flex flex-col gap-1 ${className}`}>
      {label && (
        <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
          {label}
        </label>
      )}
      <AriaSelect
        selectedKey={value || null}
        onSelectionChange={(key) => {
          if (onChange && typeof key === 'string') {
            onChange(key);
          }
        }}
        isDisabled={isDisabled}
      >
        <Button
          className={`
            w-full px-3 py-2 text-left text-sm
            bg-white dark:bg-neutral-900
            border border-neutral-300 dark:border-neutral-800
            rounded-lg
            text-neutral-900 dark:text-white
            focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:focus:ring-offset-neutral-900
            disabled:opacity-50 disabled:cursor-not-allowed
            flex items-center justify-between
          `}
        >
          <SelectValue className="text-neutral-900 dark:text-white">
            {({ selectedText }) => selectedText || <span className="text-neutral-500 dark:text-neutral-400">{placeholder}</span>}
          </SelectValue>
          <ChevronDownIcon className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
        </Button>
        <Popover
          className="outline-none z-50"
          placement="bottom start"
        >
          <ListBox
            className="min-w-[200px] bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg shadow-lg p-1 max-h-60 overflow-auto"
          >
            {children}
          </ListBox>
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
      className={`
        px-3 py-2 rounded-md text-sm
        text-neutral-700 dark:text-neutral-300
        cursor-pointer
        hover:bg-neutral-100 dark:hover:bg-neutral-800
        focus:bg-neutral-100 dark:focus:bg-neutral-800
        selected:bg-neutral-100 dark:selected:bg-neutral-800
        outline-none
      `}
    >
      {children}
    </ListBoxItem>
  );
}

// Export as namespace for easier usage
Select.Option = SelectOption;

