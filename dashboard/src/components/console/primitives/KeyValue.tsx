import type { ReactNode } from 'react';

export function KeyValue({ children }: { children: ReactNode }) {
  return <dl className="kv">{children}</dl>;
}

/**
 * Renders a <dt>/<dd> pair as a fragment so consumers can map() rows as
 * direct children of <KeyValue>. Pass `key` on the JSX element when iterating.
 */
export function KeyValueRow({ term, children }: { term: ReactNode; children: ReactNode }) {
  return (
    <>
      <dt>{term}</dt>
      <dd>{children}</dd>
    </>
  );
}
