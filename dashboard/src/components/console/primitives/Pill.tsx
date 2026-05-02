import type { ReactNode } from 'react';

export type PillTone = 'teal' | 'magenta' | 'amber' | 'green' | 'red' | 'muted' | 'default';

interface PillProps {
  tone?: PillTone;
  className?: string;
  live?: boolean;
  children: ReactNode;
}

export function Pill({ tone = 'default', className, live, children }: PillProps) {
  const cls = [
    'pill',
    tone !== 'default' ? tone : '',
    live ? 'live' : '',
    className ?? '',
  ]
    .filter(Boolean)
    .join(' ');
  return <span className={cls}>{children}</span>;
}
