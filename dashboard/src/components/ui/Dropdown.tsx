/**
 * Dropdown component — console design language.
 *
 * Public API preserved (`<Dropdown>`, `<DropdownItem>`, `<DropdownSection>`,
 * `<DropdownSeparator>`); we still use `react-aria-components` for
 * accessibility, but every Tailwind class is replaced with inline
 * styles or console.css classes (`.btn`).
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

interface DropdownProps {
  children: React.ReactNode;
  label?: string;
  icon?: React.ReactNode;
  variant?: 'button' | 'icon' | 'avatar';
  placement?:
    | 'top'
    | 'bottom'
    | 'left'
    | 'right'
    | 'top start'
    | 'top end'
    | 'bottom start'
    | 'bottom end';
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

const popoverStyle: React.CSSProperties = {
  outline: 'none',
  zIndex: 50,
};

const menuStyle: React.CSSProperties = {
  minWidth: 200,
  background: 'var(--panel-hi)',
  border: '1px solid var(--border)',
  borderRadius: 'var(--radius)',
  boxShadow: 'var(--shadow-lg)',
  padding: 4,
};

const itemBaseStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '6px 10px',
  borderRadius: 4,
  fontSize: 13,
  color: 'var(--text-1)',
  cursor: 'pointer',
  outline: 'none',
};

const iconBtnStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: 6,
  background: 'transparent',
  border: '1px solid transparent',
  borderRadius: 6,
  color: 'var(--text-2)',
  cursor: 'pointer',
};

export function Dropdown({
  children,
  label,
  icon,
  variant = 'button',
  placement = 'bottom start',
}: DropdownProps) {
  const triggerContent = () => {
    if ((variant === 'icon' || variant === 'avatar') && icon) {
      return icon;
    }
    return (
      <>
        {label}
        <ChevronDownIcon />
      </>
    );
  };

  return (
    <MenuTrigger>
      <AriaButton
        className={variant === 'button' ? 'btn' : undefined}
        style={
          variant === 'button'
            ? { display: 'inline-flex', alignItems: 'center', gap: 8 }
            : iconBtnStyle
        }
      >
        {triggerContent()}
      </AriaButton>
      <Popover placement={placement} style={popoverStyle}>
        <Menu style={menuStyle}>{children}</Menu>
      </Popover>
    </MenuTrigger>
  );
}

export function DropdownItem({
  id,
  children,
  icon,
  addon,
  isDisabled,
  onAction,
}: DropdownItemProps) {
  return (
    <MenuItem
      id={id}
      isDisabled={isDisabled}
      onAction={() => {
        if (onAction && !isDisabled) {
          onAction();
        }
      }}
      style={({ isFocused, isDisabled: di }) => ({
        ...itemBaseStyle,
        background: isFocused ? 'var(--bg-3)' : 'transparent',
        opacity: di ? 0.5 : 1,
        cursor: di ? 'not-allowed' : 'pointer',
      })}
    >
      {icon && (
        <span
          style={{ width: 16, height: 16, display: 'inline-flex', alignItems: 'center' }}
        >
          {icon}
        </span>
      )}
      <span style={{ flex: 1 }}>{children}</span>
      {addon && <span style={{ fontSize: 11, color: 'var(--text-2)' }}>{addon}</span>}
    </MenuItem>
  );
}

export function DropdownSection({ children, title }: DropdownSectionProps) {
  return (
    <Section>
      {title && (
        <AriaHeader
          style={{
            padding: '6px 10px',
            fontSize: 10,
            fontWeight: 600,
            color: 'var(--text-2)',
            textTransform: 'uppercase',
            letterSpacing: '0.04em',
          }}
        >
          {title}
        </AriaHeader>
      )}
      {children}
    </Section>
  );
}

export function DropdownSeparator() {
  return (
    <Separator
      style={{
        height: 1,
        background: 'var(--border)',
        margin: '4px 0',
        border: 0,
      }}
    />
  );
}

// Export as namespace for easier usage
Dropdown.Item = DropdownItem;
Dropdown.Section = DropdownSection;
Dropdown.Separator = DropdownSeparator;
