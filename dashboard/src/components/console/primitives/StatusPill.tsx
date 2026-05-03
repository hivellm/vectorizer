import { Pill, type PillTone } from './Pill';

const MAP = {
  healthy:        { tone: 'green',  label: 'healthy' },
  'in-sync':      { tone: 'green',  label: 'in-sync' },
  ok:             { tone: 'green',  label: 'ok' },
  indexing:       { tone: 'amber',  label: 'indexing' },
  'catching-up':  { tone: 'amber',  label: 'catching up' },
  warning:        { tone: 'amber',  label: 'warning' },
  error:          { tone: 'red',    label: 'error' },
} as const satisfies Record<string, { tone: PillTone; label: string }>;

export type KnownStatus = keyof typeof MAP;

export function StatusPill({ status }: { status: string }) {
  const m = (MAP as Record<string, { tone: PillTone; label: string }>)[status]
    ?? { tone: 'muted', label: status };
  return (
    <Pill tone={m.tone}>
      <span className={`dot ${m.tone === 'muted' ? '' : m.tone}`} />
      {m.label}
    </Pill>
  );
}
