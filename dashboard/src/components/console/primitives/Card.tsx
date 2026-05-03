import type { ReactNode } from 'react';

interface CardProps {
  className?: string;
  children: ReactNode;
}
export function Card({ className, children }: CardProps) {
  return <div className={['card', className].filter(Boolean).join(' ')}>{children}</div>;
}

type CardHeadProps =
  | { children: ReactNode; title?: never; sub?: never; right?: never }
  | { children?: never; title: ReactNode; sub?: ReactNode; right?: ReactNode };

export function CardHead(props: CardHeadProps) {
  if ('children' in props && props.children !== undefined) {
    return <div className="card-head">{props.children}</div>;
  }
  return (
    <div className="card-head">
      <div className="title">{props.title}</div>
      {props.sub && <span className="sub">{props.sub}</span>}
      {props.right}
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
