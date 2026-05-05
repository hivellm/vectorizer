/**
 * Table — console design.
 *
 * Wrappers around the console `.tbl` styles. Preserves the legacy
 * named-export API (`Table`, `TableHeader`, `TableBody`, `TableRow`,
 * `TableHead`, `TableCell`) so existing consumers keep working.
 */

import type { ReactNode } from 'react';
import { Tbl } from '@/components/console';

interface TableProps {
  children: ReactNode;
  className?: string;
}

export function Table({ children, className }: TableProps) {
  return (
    <div style={{ overflowX: 'auto' }}>
      <Tbl className={className}>{children}</Tbl>
    </div>
  );
}

interface TableHeaderProps {
  children: ReactNode;
  className?: string;
}

export function TableHeader({ children, className }: TableHeaderProps) {
  return <thead className={className}>{children}</thead>;
}

interface TableBodyProps {
  children: ReactNode;
  className?: string;
}

export function TableBody({ children, className }: TableBodyProps) {
  return <tbody className={className}>{children}</tbody>;
}

interface TableRowProps {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
}

export function TableRow({ children, className, onClick }: TableRowProps) {
  return (
    <tr
      className={className}
      onClick={onClick}
      style={onClick ? { cursor: 'pointer' } : undefined}
    >
      {children}
    </tr>
  );
}

interface TableHeadProps {
  children: ReactNode;
  className?: string;
  align?: 'left' | 'center' | 'right';
}

export function TableHead({ children, className, align = 'left' }: TableHeadProps) {
  return (
    <th className={className} style={{ textAlign: align }}>
      {children}
    </th>
  );
}

interface TableCellProps {
  children: ReactNode;
  className?: string;
  align?: 'left' | 'center' | 'right';
}

export function TableCell({ children, className, align = 'left' }: TableCellProps) {
  return (
    <td className={className} style={{ textAlign: align }}>
      {children}
    </td>
  );
}
