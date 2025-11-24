/**
 * Dropdown component based on Untitled UI
 * Uses React Aria Components
 */

import * as React from 'react';
import {
  Button as AriaButton,
  Menu,
  MenuItem,
  MenuTrigger,
  Popover,
  Separator,
  Section,
  Header as AriaHeader,
} from 'react-aria-components';
// ChevronDown icon - using SVG inline since it might not be available
const ChevronDownIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
  </svg>
);

interface DropdownProps {
  children: React.ReactNode;
  label?: string;
  icon?: React.ReactNode;
  variant?: 'button' | 'icon' | 'avatar';
  placement?: 'top' | 'bottom' | 'left' | 'right' | 'top start' | 'top end' | 'bottom start' | 'bottom end';
}

interface DropdownItemProps {
  id: string;
  children: React.ReactNode;
  icon?: React.ReactNode;
  addon?: React.ReactNode;
  isDisabled?: boolean;
  onAction?: () => void;
}

interface DropdownSectionProps {
  children: React.ReactNode;
  title?: string;
}

const DropdownContext = React.createContext<{
  variant?: 'button' | 'icon' | 'avatar';
}>({});

export function Dropdown({ children, label, icon, variant = 'button', placement = 'bottom start' }: DropdownProps) {
  const triggerContent = () => {
    if (variant === 'icon' && icon) {
      return icon;
    }
    if (variant === 'avatar' && icon) {
      return icon;
    }
    return (
      <>
        {label}
        <ChevronDownIcon className="w-4 h-4 ml-2" />
      </>
    );
  };

  return (
    <DropdownContext.Provider value={{ variant }}>
      <MenuTrigger>
        <AriaButton
          className={`
            inline-flex items-center justify-center font-medium rounded-lg transition-colors
            focus:outline-none focus:ring-2 focus:ring-offset-2 dark:focus:ring-offset-neutral-900
            ${variant === 'button'
              ? 'px-4 py-2 text-sm bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-neutral-100 hover:bg-neutral-50 dark:hover:bg-neutral-800 focus:ring-neutral-500'
              : variant === 'icon'
              ? 'p-2 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 focus:ring-neutral-500 rounded-md'
              : 'p-2 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 focus:ring-neutral-500 rounded-md'
            }
          `}
        >
          {triggerContent()}
        </AriaButton>
        <Popover
          placement={placement}
          className="outline-none z-50"
        >
          <Menu
            className="min-w-[200px] bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg shadow-lg p-1"
          >
            {children}
          </Menu>
        </Popover>
      </MenuTrigger>
    </DropdownContext.Provider>
  );
}

export function DropdownItem({ id, children, icon, addon, isDisabled, onAction }: DropdownItemProps) {
  return (
    <MenuItem
      id={id}
      isDisabled={isDisabled}
      onAction={() => {
        if (onAction && !isDisabled) {
          onAction();
        }
      }}
      className={`
        flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium
        text-neutral-700 dark:text-neutral-300
        cursor-pointer
        hover:bg-neutral-100 dark:hover:bg-neutral-800
        focus:bg-neutral-100 dark:focus:bg-neutral-800
        disabled:opacity-50 disabled:cursor-not-allowed
        outline-none
      `}
    >
      {icon && <span className="w-4 h-4 flex items-center justify-center">{icon}</span>}
      <span className="flex-1">{children}</span>
      {addon && <span className="text-xs text-neutral-500 dark:text-neutral-400">{addon}</span>}
    </MenuItem>
  );
}

export function DropdownSection({ children, title }: DropdownSectionProps) {
  return (
    <Section>
      {title && (
        <AriaHeader className="px-3 py-2 text-xs font-semibold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
          {title}
        </AriaHeader>
      )}
      {children}
    </Section>
  );
}

export function DropdownSeparator() {
  return (
    <Separator className="h-px bg-neutral-200 dark:bg-neutral-700 my-1" />
  );
}

// Export as namespace for easier usage
Dropdown.Item = DropdownItem;
Dropdown.Section = DropdownSection;
Dropdown.Separator = DropdownSeparator;

