import { useEffect, useMemo, useState } from 'react';
import { Icons } from './Icons';

interface CmdItem {
  label: string;
  to: string;
  icon: keyof typeof Icons;
  hint?: string;
  section: string;
}

const ITEMS: CmdItem[] = [
  { section: 'Navigation', label: 'Go to Overview',     to: '/overview',     icon: 'dashboard',   hint: 'G O' },
  { section: 'Navigation', label: 'Go to Collections',  to: '/collections',  icon: 'collections', hint: 'G C' },
  { section: 'Navigation', label: 'Go to Search',       to: '/search',       icon: 'search',      hint: 'G S' },
  { section: 'Navigation', label: 'Go to Vectors',      to: '/vectors',      icon: 'vectors',     hint: 'G V' },
  { section: 'Navigation', label: 'Go to Monitoring',   to: '/monitoring',   icon: 'activity',    hint: 'G M' },
  { section: 'Navigation', label: 'Go to API Keys',     to: '/api-keys',     icon: 'keys',        hint: 'G K' },
  { section: 'Navigation', label: 'Go to MCP Tools',    to: '/mcp-tools',    icon: 'mcp' },
  { section: 'Navigation', label: 'Go to Settings',     to: '/configuration',icon: 'settings' },
];

interface Props {
  open: boolean;
  onClose: () => void;
  onNavigate: (to: string) => void;
}

export function CommandPalette({ open, onClose, onNavigate }: Props) {
  const [q, setQ] = useState('');
  const [active, setActive] = useState(0);

  useEffect(() => { if (open) { setQ(''); setActive(0); } }, [open]);

  const flat = useMemo(
    () => ITEMS.filter((it) => it.label.toLowerCase().includes(q.toLowerCase())),
    [q],
  );

  if (!open) return null;

  const go = (it: CmdItem) => {
    onNavigate(it.to);
    onClose();
  };

  return (
    <div className="cmd-overlay" onClick={onClose} role="dialog" aria-modal aria-label="Command palette">
      <div className="cmd-panel" onClick={(e) => e.stopPropagation()}>
        <input
          className="cmd-input"
          placeholder="Search or type a command…"
          autoFocus
          role="combobox"
          aria-expanded
          aria-controls="cmdk-list"
          aria-activedescendant={flat[active] ? `cmdk-${flat[active].to}` : undefined}
          value={q}
          onChange={(e) => { setQ(e.target.value); setActive(0); }}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') { setActive((a) => Math.min(a + 1, flat.length - 1)); e.preventDefault(); }
            if (e.key === 'ArrowUp')   { setActive((a) => Math.max(a - 1, 0)); e.preventDefault(); }
            if (e.key === 'Enter' && flat[active]) go(flat[active]);
            if (e.key === 'Escape')   onClose();
          }}
        />
        <div className="cmd-list" role="listbox" id="cmdk-list">
          {Object.entries(
            flat.reduce<Record<string, CmdItem[]>>((acc, it) => {
              (acc[it.section] ??= []).push(it);
              return acc;
            }, {}),
          ).map(([section, items]) => (
            <div key={section}>
              <div className="cmd-section">{section}</div>
              {items.map((it) => {
                const idx = flat.indexOf(it);
                const Icon = Icons[it.icon];
                return (
                  <div
                    key={it.to}
                    id={`cmdk-${it.to}`}
                    role="option"
                    aria-selected={idx === active}
                    className={`cmd-row ${idx === active ? 'active' : ''}`}
                    onClick={() => go(it)}
                    onMouseEnter={() => setActive(idx)}
                  >
                    <Icon className="icon" />
                    <span>{it.label}</span>
                    {it.hint && <span className="hint">{it.hint}</span>}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
