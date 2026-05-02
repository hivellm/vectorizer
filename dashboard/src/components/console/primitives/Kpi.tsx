import type { ReactNode } from 'react';
import { Sparkline } from './Sparkline';

type Accent = 'teal' | 'magenta' | 'amber' | 'none';

interface DeltaProps {
  tone: 'up' | 'down' | 'neutral';
  text: string;
}

interface KpiProps {
  label: ReactNode;
  value: ReactNode;
  unit?: string;
  delta?: DeltaProps;
  accent?: Accent;
  spark?: { data: number[]; color?: string };
}

export function Kpi({ label, value, unit, delta, accent = 'none', spark }: KpiProps) {
  const cls = ['kpi', accent !== 'none' ? `accent-${accent}` : ''].filter(Boolean).join(' ');
  return (
    <div className={cls}>
      <div className="label">{label}</div>
      <div className="value tnum">
        {value}
        {unit && <span className="unit">{unit}</span>}
      </div>
      {delta && <div className={`delta ${delta.tone}`}>{delta.text}</div>}
      {spark && (
        <div className="spark">
          <Sparkline data={spark.data} color={spark.color ?? 'var(--teal)'} />
        </div>
      )}
    </div>
  );
}
