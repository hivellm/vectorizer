import type { ReactNode } from 'react';

interface CardProps {
  className?: string;
  children: ReactNode;
}
export function Card({ className, children }: CardProps) {
  return <div className={['card', className].filter(Boolean).join(' ')}>{children}</div>;
}

interface CardHeadProps {
  title?: ReactNode;
  sub?: ReactNode;
  right?: ReactNode;
  children?: ReactNode;
}
export function CardHead({ title, sub, right, children }: CardHeadProps) {
  if (children) return <div className="card-head">{children}</div>;
  return (
    <div className="card-head">
      <div className="title">{title}</div>
      {sub && <span className="sub">{sub}</span>}
      {right}
    </div>
  );
}

interface CardBodyProps {
  tight?: boolean;
  className?: string;
  children: ReactNode;
}
export function CardBody({ tight, className, children }: CardBodyProps) {
  return (
    <div className={['card-body', tight ? 'tight' : '', className ?? ''].filter(Boolean).join(' ')}>
      {children}
    </div>
  );
}
