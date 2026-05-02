import { Fragment } from 'react';
import { Icons } from './Icons';

interface Props {
  crumbs: string[];
  onOpenCmd: () => void;
  onRefresh?: () => void;
}

export function ConsoleTopbar({ crumbs, onOpenCmd, onRefresh }: Props) {
  return (
    <div className="topbar">
      <div className="crumbs">
        {crumbs.map((c, i) => (
          <Fragment key={i}>
            {i > 0 && <span className="sep">›</span>}
            <span className={i === crumbs.length - 1 ? 'now' : undefined}>{c}</span>
          </Fragment>
        ))}
      </div>
      <div className="env-switch">
        <span className="dot" />
        <span>production</span>
        <Icons.chevron size={12} />
      </div>
      <button type="button" className="cmdk" onClick={onOpenCmd}>
        <Icons.search size={13} />
        <span>Search collections, vectors, keys…</span>
        <span className="kbd">⌘K</span>
      </button>
      <button type="button" className="icon-btn" title="Notifications">
        <Icons.bell />
      </button>
      <button type="button" className="icon-btn" title="Refresh" onClick={onRefresh}>
        <Icons.refresh />
      </button>
    </div>
  );
}
