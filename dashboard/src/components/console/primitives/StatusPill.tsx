import { Pill, type PillTone } from './Pill';

const MAP: Record<string, { tone: PillTone; label: string }> = {
  healthy: { tone: 'green', label: 'healthy' },
  'in-sync': { tone: 'green', label: 'in-sync' },
  ok: { tone: 'green', label: 'ok' },
  indexing: { tone: 'amber', label: 'indexing' },
  'catching-up': { tone: 'amber', label: 'catching up' },
  warning: { tone: 'amber', label: 'warning' },
  error: { tone: 'red', label: 'error' },
};

export function StatusPill({ status }: { status: string }) {
  const m = MAP[status] ?? { tone: 'muted' as PillTone, label: status };
  return (
    <Pill tone={m.tone}>
      <span className={`dot ${m.tone === 'muted' ? '' : m.tone}`} />
      {m.label}
    </Pill>
  );
}
