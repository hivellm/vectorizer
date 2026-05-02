import type { HTMLAttributes, TableHTMLAttributes, TdHTMLAttributes, ThHTMLAttributes, ReactNode } from 'react';

export function Tbl({ children, ...rest }: TableHTMLAttributes<HTMLTableElement>) {
  return <table className="tbl" {...rest}>{children}</table>;
}

export function Th({ children, ...rest }: ThHTMLAttributes<HTMLTableCellElement>) {
  return <th {...rest}>{children}</th>;
}

export function Td({ children, ...rest }: TdHTMLAttributes<HTMLTableCellElement>) {
  return <td {...rest}>{children}</td>;
}

export interface RowAttrs extends HTMLAttributes<HTMLTableRowElement> {
  active?: boolean;
}
export function Tr({ active, className, children, ...rest }: RowAttrs & { children: ReactNode }) {
  return (
    <tr className={[active ? 'active' : '', className ?? ''].filter(Boolean).join(' ')} {...rest}>
      {children}
    </tr>
  );
}
